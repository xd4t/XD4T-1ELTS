use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{
    AutoRefillStatus, DailyQuest, DashboardState, DictationErrorPair, DictationResultInput,
    DictationSentence, DictationSubmitResult, ErrorDrillQuestion, ErrorDrillResultInput,
    ErrorDrillSingleResult, ErrorDrillSubmitResult, ErrorFixQuestion, ErrorLog, Essay,
    ForgottenVocabCard, GradeResult, MicroCheckAnswerInput, MicroCheckItem, MicroCheckSubmitResult,
    ParaphrasePair, PracticeScore, QuizAnswerInput, QuizQuestion, QuizSubmitResult,
    RefillPackResult, RoadmapItem, Settings, SettingsPublic, SkillBand, SpeakingCue,
    SpeakingSession, StudyDay, TopicPackDrill, TopicPackPayload, VocabCard, VocabQuizStatus,
    WritingPrompt,
};
use crate::roadmap_parser::{infer_quest_action_hint, parse_roadmap_markdown, ParsedQuest};
use crate::seed_data::{
    all_dictation_seeds, all_paraphrase_seeds_65, all_vocab_seeds_65, all_vocab_seeds_80,
    ERROR_LOG_SEEDS_65, ERROR_LOG_SEEDS_80, ROADMAP_SEEDS_65, ROADMAP_SEEDS_80, SPEAKING_SEEDS,
};
use crate::seed_data_v170::{
    ERROR_DRILL_SEEDS_65, ERROR_DRILL_SEEDS_80, IELTS_TOPICS, PARAPHRASE_SEEDS_80,
    WRITING_PROMPT_SEEDS,
};
use crate::srs::apply_sm2;

pub fn default_writing_checklist() -> Vec<String> {
    vec![
        "Chủ ngữ số ít (-s/-es/has)".to_string(),
        "Danh từ đếm được (a/an/the/-s)".to_string(),
        "chance/change, choose/choice, effect/affect".to_string(),
        "Tính từ/từ loại (harmful, In addition)".to_string(),
        "Không comma splice / câu cụt".to_string(),
        "Đúng khung, kết bài không thêm ý mới".to_string(),
    ]
}

pub fn default_writing_checklist_80() -> Vec<String> {
    vec![
        "Trả lời đủ mọi vế đề & lập trường nhất quán (TR/TA 7.5+)".to_string(),
        "Mạch ý tự nhiên (tránh lạm dụng Firstly/Moreover cứng nhắc)".to_string(),
        "Collocations C1 tự nhiên, tránh lặp từ & văn nói".to_string(),
        "Hedging học thuật (tend to, arguably, to a large extent)".to_string(),
        "Đa dạng câu phức (mệnh đề quan hệ rút gọn, phân từ, đảo ngữ nhẹ)".to_string(),
        "Sạch lỗi nhỏ (mạo từ, giới từ, thì, dấu câu) để đạt GRA 7.5+".to_string(),
    ]
}

pub fn default_writing_focus_notes() -> Vec<String> {
    vec![
        "Nhóm 1: Subject-verb agreement (everybody knows), countable nouns needing a/an/the or -s (humans/people), articles.".to_string(),
        "Nhóm 2: Confused words (change/chance, choose/choice, effect/affect) and word family (harmful vs harmfully, In addition vs In additionally).".to_string(),
        "Nhóm 3: Comma splices (joining 2 independent clauses with a comma) and sentence fragments.".to_string(),
    ]
}

pub fn default_writing_focus_notes_80() -> Vec<String> {
    vec![
        "Over-generalization & thiếu hedging: thay khẳng định tuyệt đối bằng tend to, arguably, to a considerable extent.".to_string(),
        "Mechanical linkers (Firstly/Moreover/In conclusion): dùng referencing, substitution và lexical cohesion tự nhiên.".to_string(),
        "Imprecise collocations, wordiness & subtle GRA slips (mạo từ, giới từ, thì) kéo tụt điểm 7.5–8.0.".to_string(),
    ]
}

pub fn default_skill_targets() -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("Listening".to_string(), 7.5);
    map.insert("Reading".to_string(), 7.0);
    map.insert("Speaking".to_string(), 6.5);
    map.insert("Writing".to_string(), 6.0);
    map
}

pub fn default_skill_targets_80() -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("Listening".to_string(), 8.5);
    map.insert("Reading".to_string(), 8.5);
    map.insert("Speaking".to_string(), 7.5);
    map.insert("Writing".to_string(), 7.5);
    map
}

pub fn default_phase_labels() -> BTreeMap<i64, String> {
    let mut map = BTreeMap::new();
    map.insert(1, "GĐ1 · Diệt lỗi nền & 5 khung T2 (T1–6)".to_string());
    map.insert(2, "GĐ2 · Tăng tốc & 6 cấu trúc nâng cao (T7–14)".to_string());
    map.insert(3, "GĐ3 · Vùng đệm 7.0 & Forecast thực chiến (T15–20)".to_string());
    map.insert(4, "GĐ4 · Nước rút phòng thi (T21–24)".to_string());
    map
}

pub fn default_phase_labels_80() -> BTreeMap<i64, String> {
    let mut map = BTreeMap::new();
    map.insert(1, "GĐ1 · Phá trần 6.5 lên 7.0+ (Tuần 1–6)".to_string());
    map.insert(2, "GĐ2 · Nâng tầm C1–C2 & Phản biện Band 7.5–8.0 (Tuần 7–14)".to_string());
    map.insert(3, "GĐ3 · Chốt chuẩn Band 8.0 & 12 chủ đề trừu tượng (Tuần 15–20)".to_string());
    map.insert(4, "GĐ4 · Nước rút Peak Performance phòng thi (Tuần 21–24)".to_string());
    map
}

pub fn normalize_theme(theme: &str) -> String {
    match theme.trim() {
        "dark" | "mono-dark" => "mono-dark".to_string(),
        "light" | "mono-light" => "mono-light".to_string(),
        "pastel-female" => "pastel-female".to_string(),
        "pastel-male" => "pastel-male".to_string(),
        _ => "pastel-male".to_string(),
    }
}

pub fn normalize_tts_accent(accent: &str) -> String {
    if accent.trim().eq_ignore_ascii_case("en-us") {
        "en-US".to_string()
    } else {
        "en-GB".to_string()
    }
}

pub fn data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA environment variable not set".to_string())?;
    let dir = base.join("xd4t-ielts65");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {e}"))?;
    Ok(dir)
}

pub fn db_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("ielts65.db"))
}

pub fn open_and_init_db() -> Result<Connection, String> {
    let dir = data_dir()?;
    let db_path = dir.join("ielts65.db");
    let seed_path = dir.join("api_key.seed");
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open SQLite: {e}"))?;
    init_schema_and_seed(&conn, Some(&seed_path))?;
    Ok(conn)
}

pub fn mask_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.len() <= 8 {
        return "••••••••".to_string();
    }
    let prefix: String = trimmed.chars().take(3).collect();
    let suffix: String = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}••••{suffix}")
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({table})");
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return false;
    };
    let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(1)) else {
        return false;
    };
    for row in rows.flatten() {
        if row.eq_ignore_ascii_case(column) {
            return true;
        }
    }
    false
}

fn ensure_column_exists(conn: &Connection, table: &str, column: &str, alter_sql: &str) -> Result<bool, String> {
    if column_exists(conn, table, column) {
        return Ok(false);
    }
    conn.execute(alter_sql, [])
        .map_err(|e| format!("Migration error ({table}.{column}): {e}"))?;
    Ok(true)
}

pub fn init_schema_and_seed(conn: &Connection, seed_path: Option<&Path>) -> Result<(), String> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            theme TEXT NOT NULL DEFAULT 'pastel-male',
            gemini_api_key TEXT NOT NULL DEFAULT '',
            gemini_model TEXT NOT NULL DEFAULT 'gemini-3-flash-preview',
            exam_date TEXT NOT NULL DEFAULT '2027-05-15',
            reminder_time TEXT NOT NULL DEFAULT '21:00',
            autostart INTEGER NOT NULL DEFAULT 0,
            target_band REAL NOT NULL DEFAULT 7.0,
            onboarding_done INTEGER NOT NULL DEFAULT 0,
            tts_accent TEXT NOT NULL DEFAULT 'en-GB'
        );

        CREATE TABLE IF NOT EXISTS daily_quests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            track TEXT NOT NULL DEFAULT 'ielts',
            title TEXT NOT NULL,
            meta TEXT NOT NULL,
            minutes INTEGER NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            tier TEXT NOT NULL DEFAULT 'core',
            action_hint TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS study_days (
            date TEXT PRIMARY KEY,
            track TEXT NOT NULL DEFAULT 'ielts',
            minutes INTEGER NOT NULL DEFAULT 0,
            quests_done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS vocab_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track TEXT NOT NULL DEFAULT 'ielts',
            topic TEXT NOT NULL,
            word TEXT NOT NULL,
            meaning TEXT NOT NULL,
            example TEXT NOT NULL,
            ease REAL NOT NULL DEFAULT 2.5,
            interval_days INTEGER NOT NULL DEFAULT 0,
            reps INTEGER NOT NULL DEFAULT 0,
            due_date TEXT NOT NULL,
            lapses INTEGER NOT NULL DEFAULT 0,
            hidden INTEGER NOT NULL DEFAULT 0,
            is_user_card INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS essays (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mode TEXT NOT NULL,
            prompt TEXT NOT NULL,
            content TEXT NOT NULL,
            word_count INTEGER NOT NULL,
            ta REAL NOT NULL,
            cc REAL NOT NULL,
            lr REAL NOT NULL,
            gra REAL NOT NULL,
            overall REAL NOT NULL,
            feedback_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS error_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category TEXT NOT NULL,
            wrong_text TEXT NOT NULL,
            right_text TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS practice_scores (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            skill TEXT NOT NULL,
            source TEXT NOT NULL,
            score REAL NOT NULL,
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS roadmap_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track TEXT NOT NULL DEFAULT 'ielts',
            phase INTEGER NOT NULL,
            week_label TEXT NOT NULL,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS vocab_quizzes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            taken_at TEXT NOT NULL,
            mode TEXT NOT NULL,
            total_questions INTEGER NOT NULL,
            correct_count INTEGER NOT NULL,
            forgotten_words TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS paraphrase_pairs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_term TEXT NOT NULL,
            passage_paraphrase TEXT NOT NULL,
            source_type TEXT NOT NULL DEFAULT '',
            trap_note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dictation_sentences (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            level TEXT NOT NULL,
            section TEXT NOT NULL,
            trap_focus TEXT NOT NULL,
            sentence TEXT NOT NULL,
            meaning_vi TEXT NOT NULL,
            practiced_count INTEGER NOT NULL DEFAULT 0,
            last_accuracy INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS speaking_bank (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            week_num INTEGER NOT NULL,
            level TEXT NOT NULL,
            topic TEXT NOT NULL,
            cue_title TEXT NOT NULL,
            cue_bullets_json TEXT NOT NULL,
            part3_questions_json TEXT NOT NULL,
            target_phrases_json TEXT NOT NULL,
            used_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS speaking_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at TEXT NOT NULL,
            cue_id INTEGER,
            cue_title TEXT NOT NULL,
            track TEXT NOT NULL,
            mode TEXT NOT NULL,
            wpm_start INTEGER NOT NULL DEFAULT 0,
            wpm_end INTEGER NOT NULL DEFAULT 0,
            fillers_start INTEGER NOT NULL DEFAULT 0,
            fillers_end INTEGER NOT NULL DEFAULT 0,
            fc_band REAL NOT NULL DEFAULT 0.0,
            lr_band REAL NOT NULL DEFAULT 0.0,
            gra_band REAL NOT NULL DEFAULT 0.0,
            p_band REAL NOT NULL DEFAULT 0.0,
            overall_band REAL NOT NULL DEFAULT 0.0,
            progress_note TEXT NOT NULL DEFAULT '',
            transcript TEXT NOT NULL DEFAULT '',
            feedback_json TEXT NOT NULL DEFAULT '{}'
        );

        CREATE TABLE IF NOT EXISTS dictation_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at TEXT NOT NULL,
            total_sentences INTEGER NOT NULL,
            avg_accuracy INTEGER NOT NULL,
            wrong_words_json TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS error_drills (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            error_log_id INTEGER NOT NULL,
            drill_mode TEXT NOT NULL,
            context_tag TEXT NOT NULL DEFAULT 'general',
            stem TEXT NOT NULL,
            options_json TEXT NOT NULL DEFAULT '[]',
            answer TEXT NOT NULL,
            rationale_vi TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'seed'
        );

        CREATE TABLE IF NOT EXISTS writing_prompts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            level TEXT NOT NULL,
            task_type TEXT NOT NULL,
            topic TEXT NOT NULL,
            title TEXT NOT NULL,
            prompt_text TEXT NOT NULL,
            outline_hint TEXT NOT NULL DEFAULT '',
            target_vocab_json TEXT NOT NULL DEFAULT '[]',
            source TEXT NOT NULL DEFAULT 'seed',
            practiced_count INTEGER NOT NULL DEFAULT 0,
            chart_spec_json TEXT NOT NULL DEFAULT ''
        );
        "#,
    )
    .map_err(|e| format!("Schema init error: {e}"))?;

    conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
        .map_err(|e| format!("Settings seed error: {e}"))?;

    // Safe PRAGMA table_info-checked migrations (v1.0..v1.5 -> v1.6.0)
    ensure_column_exists(
        conn,
        "settings",
        "writing_checklist_json",
        "ALTER TABLE settings ADD COLUMN writing_checklist_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "writing_focus_json",
        "ALTER TABLE settings ADD COLUMN writing_focus_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "skill_targets_json",
        "ALTER TABLE settings ADD COLUMN skill_targets_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "phase_labels_json",
        "ALTER TABLE settings ADD COLUMN phase_labels_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "daily_quests_template_json",
        "ALTER TABLE settings ADD COLUMN daily_quests_template_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "tts_accent",
        "ALTER TABLE settings ADD COLUMN tts_accent TEXT NOT NULL DEFAULT 'en-GB'",
    )?;

    let added_onboarding = ensure_column_exists(
        conn,
        "settings",
        "onboarding_done",
        "ALTER TABLE settings ADD COLUMN onboarding_done INTEGER NOT NULL DEFAULT 0",
    )?;
    if added_onboarding {
        let has_legacy_data: bool = conn
            .query_row(
                "SELECT (length(trim(gemini_api_key)) > 0)
                     OR (theme IN ('dark', 'light'))
                     OR ((SELECT COUNT(*) FROM study_days) > 0)
                     OR ((SELECT COUNT(*) FROM essays) > 0)
                 FROM settings WHERE id = 1",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            != 0;
        if has_legacy_data {
            let _ = conn.execute("UPDATE settings SET onboarding_done = 1 WHERE id = 1", []);
        }
    }

    ensure_column_exists(
        conn,
        "vocab_cards",
        "lapses",
        "ALTER TABLE vocab_cards ADD COLUMN lapses INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column_exists(
        conn,
        "vocab_cards",
        "hidden",
        "ALTER TABLE vocab_cards ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column_exists(
        conn,
        "vocab_cards",
        "is_user_card",
        "ALTER TABLE vocab_cards ADD COLUMN is_user_card INTEGER NOT NULL DEFAULT 0",
    )?;

    ensure_column_exists(
        conn,
        "daily_quests",
        "tier",
        "ALTER TABLE daily_quests ADD COLUMN tier TEXT NOT NULL DEFAULT 'core'",
    )?;
    ensure_column_exists(
        conn,
        "daily_quests",
        "action_hint",
        "ALTER TABLE daily_quests ADD COLUMN action_hint TEXT NOT NULL DEFAULT ''",
    )?;

    // v1.7.0 additive migrations (100% backward-compatible)
    ensure_column_exists(
        conn,
        "vocab_cards",
        "source",
        "ALTER TABLE vocab_cards ADD COLUMN source TEXT NOT NULL DEFAULT 'seed'",
    )?;
    ensure_column_exists(
        conn,
        "dictation_sentences",
        "source",
        "ALTER TABLE dictation_sentences ADD COLUMN source TEXT NOT NULL DEFAULT 'seed'",
    )?;
    ensure_column_exists(
        conn,
        "paraphrase_pairs",
        "level",
        "ALTER TABLE paraphrase_pairs ADD COLUMN level TEXT NOT NULL DEFAULT '6.5'",
    )?;
    ensure_column_exists(
        conn,
        "paraphrase_pairs",
        "source",
        "ALTER TABLE paraphrase_pairs ADD COLUMN source TEXT NOT NULL DEFAULT 'seed'",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "drill_mode",
        "ALTER TABLE error_logs ADD COLUMN drill_mode TEXT NOT NULL DEFAULT 'GRAMMAR_MCQ'",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "status",
        "ALTER TABLE error_logs ADD COLUMN status TEXT NOT NULL DEFAULT 'active'",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "streak_distinct",
        "ALTER TABLE error_logs ADD COLUMN streak_distinct INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "last_drill_id",
        "ALTER TABLE error_logs ADD COLUMN last_drill_id INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "last_context_tag",
        "ALTER TABLE error_logs ADD COLUMN last_context_tag TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column_exists(
        conn,
        "error_logs",
        "source",
        "ALTER TABLE error_logs ADD COLUMN source TEXT NOT NULL DEFAULT 'user'",
    )?;
    ensure_column_exists(
        conn,
        "writing_prompts",
        "chart_spec_json",
        "ALTER TABLE writing_prompts ADD COLUMN chart_spec_json TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "auto_refill_enabled",
        "ALTER TABLE settings ADD COLUMN auto_refill_enabled INTEGER NOT NULL DEFAULT 1",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "last_ai_refill_at",
        "ALTER TABLE settings ADD COLUMN last_ai_refill_at TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column_exists(
        conn,
        "settings",
        "last_ai_refill_topic",
        "ALTER TABLE settings ADD COLUMN last_ai_refill_topic TEXT NOT NULL DEFAULT ''",
    )?;

    // Backward-compatible theme migration
    let _ = conn.execute(
        "UPDATE settings SET theme = 'mono-dark' WHERE id = 1 AND theme = 'dark'",
        [],
    );
    let _ = conn.execute(
        "UPDATE settings SET theme = 'mono-light' WHERE id = 1 AND theme = 'light'",
        [],
    );

    if let Some(path) = seed_path {
        import_seed_key_if_needed(conn, path)?;
    }

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let is_80 = target_band >= 7.5;

    let vocab_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocab_cards", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if vocab_count == 0 {
        let initial_vocab = if is_80 {
            all_vocab_seeds_80()
        } else {
            all_vocab_seeds_65()
        };
        for v in initial_vocab {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, source)
                 VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 'seed')",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    } else {
        let extra_vocab = if is_80 {
            crate::seed_data_v170::VOCAB_SEEDS_80_EXTRA
        } else {
            crate::seed_data_v170::VOCAB_SEEDS_65_EXTRA
        };
        for v in extra_vocab {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, source)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 'seed'
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE lower(word) = lower(?2))",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let roadmap_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM roadmap_items", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if roadmap_count == 0 {
        for item in ROADMAP_SEEDS_65 {
            conn.execute(
                "INSERT INTO roadmap_items (track, phase, week_label, title, done)
                 VALUES ('ielts', ?1, ?2, ?3, 0)",
                params![item.phase, item.week_label, item.title],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let score_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM practice_scores", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if score_count == 0 {
        let initial_scores = [
            ("Listening", "Test đầu vào (02/11/2026)", 5.5, "Mục tiêu 7.5 · Sàn 7.0"),
            ("Reading", "Test đầu vào (02/11/2026)", 5.0, "Mục tiêu 7.0 · Sàn 6.5"),
            ("Speaking", "Test đầu vào (02/11/2026)", 5.5, "Mục tiêu 6.5 · Sàn 6.0"),
            ("Writing", "Test đầu vào (02/11/2026)", 4.5, "Mục tiêu 6.0 · Sàn 5.5"),
        ];
        for (skill, src, score, note) in initial_scores {
            conn.execute(
                "INSERT INTO practice_scores (skill, source, score, note, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![skill, src, score, note, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let err_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM error_logs", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if err_count == 0 {
        let seeds = if is_80 { ERROR_LOG_SEEDS_80 } else { ERROR_LOG_SEEDS_65 };
        let src = if is_80 { "seed_80" } else { "seed_65" };
        for e in seeds {
            conn.execute(
                "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at, drill_mode, status, streak_distinct, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', 0, ?7)",
                params![e.category, e.wrong_text, e.right_text, e.count, today, e.drill_mode, src],
            )
            .map_err(|e| e.to_string())?;
        }
    } else {
        // Backfill drill_mode & source for upgraded DBs matching seed errors
        for e in ERROR_LOG_SEEDS_65 {
            let _ = conn.execute(
                "UPDATE error_logs SET drill_mode = ?1, source = 'seed_65'
                 WHERE wrong_text = ?2 AND right_text = ?3 AND COALESCE(source, 'user') = 'user'",
                params![e.drill_mode, e.wrong_text, e.right_text],
            );
        }
        for e in ERROR_LOG_SEEDS_80 {
            let _ = conn.execute(
                "UPDATE error_logs SET drill_mode = ?1, source = 'seed_80'
                 WHERE wrong_text = ?2 AND right_text = ?3 AND COALESCE(source, 'user') = 'user'",
                params![e.drill_mode, e.wrong_text, e.right_text],
            );
        }
    }
    ensure_seed_error_drills(conn)?;

    // v1.6.0 + v1.7.0 seeds via WHERE NOT EXISTS (safe for migrated DBs)
    for p in all_paraphrase_seeds_65() {
        conn.execute(
            "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at, level, source)
             SELECT ?1, ?2, ?3, ?4, ?5, '6.5', 'seed'
             WHERE NOT EXISTS (SELECT 1 FROM paraphrase_pairs WHERE lower(question_term) = lower(?1) AND lower(passage_paraphrase) = lower(?2))",
            params![p.question_term, p.passage_paraphrase, p.source_type, p.trap_note, today],
        )
        .map_err(|e| e.to_string())?;
    }
    for p in PARAPHRASE_SEEDS_80 {
        conn.execute(
            "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at, level, source)
             SELECT ?1, ?2, ?3, ?4, ?5, '8.0', 'seed'
             WHERE NOT EXISTS (SELECT 1 FROM paraphrase_pairs WHERE lower(question_term) = lower(?1) AND lower(passage_paraphrase) = lower(?2))",
            params![p.question_term, p.passage_paraphrase, p.source_type, p.trap_note, today],
        )
        .map_err(|e| e.to_string())?;
    }

    for d in all_dictation_seeds() {
        conn.execute(
            "INSERT INTO dictation_sentences (level, section, trap_focus, sentence, meaning_vi, source)
             SELECT ?1, ?2, ?3, ?4, ?5, 'seed'
             WHERE NOT EXISTS (SELECT 1 FROM dictation_sentences WHERE lower(sentence) = lower(?4))",
            params![d.level, d.section, d.trap_focus, d.sentence, d.meaning_vi],
        )
        .map_err(|e| e.to_string())?;
    }

    for wp in WRITING_PROMPT_SEEDS {
        let tv_json = serde_json::to_string(&wp.target_vocab).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO writing_prompts (level, task_type, topic, title, prompt_text, outline_hint, target_vocab_json, source, practiced_count, chart_spec_json)
             SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'seed', 0, ?8
             WHERE NOT EXISTS (SELECT 1 FROM writing_prompts WHERE level = ?1 AND title = ?4)",
            params![wp.level, wp.task_type, wp.topic, wp.title, wp.prompt_text, wp.outline_hint, tv_json, wp.chart_spec_json],
        )
        .map_err(|e| e.to_string())?;
        if !wp.chart_spec_json.is_empty() {
            let _ = conn.execute(
                "UPDATE writing_prompts SET chart_spec_json = ?1 WHERE level = ?2 AND title = ?3 AND trim(chart_spec_json) = ''",
                params![wp.chart_spec_json, wp.level, wp.title],
            );
        }
    }

    for s in SPEAKING_SEEDS {
        let bullets_json = serde_json::to_string(&s.bullets).unwrap_or_else(|_| "[]".to_string());
        let part3_json = serde_json::to_string(&s.part3).unwrap_or_else(|_| "[]".to_string());
        let phrases_json = serde_json::to_string(&s.phrases).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO speaking_bank (week_num, level, topic, cue_title, cue_bullets_json, part3_questions_json, target_phrases_json)
             SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7
             WHERE NOT EXISTS (SELECT 1 FROM speaking_bank WHERE week_num = ?1 AND level = ?2 AND cue_title = ?4)",
            params![s.week_num, s.level, s.topic, s.cue_title, bullets_json, part3_json, phrases_json],
        )
        .map_err(|e| e.to_string())?;
    }

    ensure_today_quests(conn, &today)?;
    Ok(())
}

fn ensure_seed_error_drills(conn: &Connection) -> Result<(), String> {
    for drill in ERROR_DRILL_SEEDS_65.iter().chain(ERROR_DRILL_SEEDS_80.iter()) {
        let error_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM error_logs WHERE wrong_text = ?1 LIMIT 1",
                params![drill.seed_wrong_text],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(eid) = error_id else {
            continue;
        };
        let opts_json = serde_json::to_string(&drill.options).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO error_drills (error_log_id, drill_mode, context_tag, stem, options_json, answer, rationale_vi, source)
             SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'seed'
             WHERE NOT EXISTS (SELECT 1 FROM error_drills WHERE error_log_id = ?1 AND stem = ?4)",
            params![
                eid,
                drill.drill_mode,
                drill.context_tag,
                drill.stem,
                opts_json,
                drill.answer,
                drill.rationale_vi
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn import_seed_key_if_needed(conn: &Connection, seed_path: &Path) -> Result<(), String> {
    if !seed_path.exists() {
        return Ok(());
    }
    let current_key: String = conn
        .query_row("SELECT gemini_api_key FROM settings WHERE id = 1", [], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;
    if !current_key.trim().is_empty() {
        return Ok(());
    }
    let raw = fs::read_to_string(seed_path).map_err(|e| format!("Failed reading seed: {e}"))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    conn.execute("BEGIN IMMEDIATE", [])
        .map_err(|e| e.to_string())?;
    let update_res = conn.execute(
        "UPDATE settings SET gemini_api_key = ?1 WHERE id = 1",
        params![trimmed],
    );
    if let Err(e) = update_res {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("Failed saving seed key to SQLite: {e}"));
    }
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(seed_path);
    Ok(())
}

pub struct QuestSeed {
    pub title: &'static str,
    pub meta: &'static str,
    pub minutes: i64,
    pub tier: &'static str,
    pub action_hint: &'static str,
}

pub fn default_quests_for_weekday(weekday: Weekday, is_80: bool) -> Vec<QuestSeed> {
    if is_80 {
        match weekday {
            Weekday::Mon => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 2 · Cụm C1–C2 & Hedging", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Reading P3 (C1): 1 Passage bấm giờ 16p → nạp 2 cặp Bảng Paraphrase", meta: "Đích 11–13/13 câu · Săn đồng nghĩa C1", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Làm thêm 1 Passage Reading + Sentence Mining 2 câu vào SRS", meta: "Bonus · Nâng vốn cụm học thuật C1–C2", minutes: 18, tier: "bonus", action_hint: "paraphrase" },
            ],
            Weekday::Tue => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 3 · Cụm C1–C2 & Hedging", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Phòng Luyện Nói 20p: 1 Cue Card tuần + Part 3 trừu tượng & 5 cụm C1–C2", meta: "Thi thật hoặc 4-3-2 · Chấm Gemini Audio/Prompt", minutes: 20, tier: "core", action_hint: "speaking" },
                QuestSeed { title: "Nghe chép câu Dictation thêm 5 câu bẫy nối âm & học thuật Part 4", meta: "Bonus · Luyện tai 100% offline", minutes: 12, tier: "bonus", action_hint: "dictation" },
            ],
            Weekday::Wed => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 4 · Cụm C1–C2 & Hedging", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Listening (Sec 3/4 — 14p) + Nghe chép Dictation 3 câu (6p)", meta: "Đích 9–10/10 câu · Bắt bẫy đổi ý", minutes: 20, tier: "core", action_hint: "dictation" },
                QuestSeed { title: "Chép chính tả lại đoạn nghe sai trên Study4", meta: "Bonus · Phân tích transcript Part 3–4", minutes: 15, tier: "bonus", action_hint: "study4" },
            ],
            Weekday::Thu => vec![
                QuestSeed { title: "Quiz Giữa Tuần & Chữa 🔥 Từ cứng đầu (10p)", meta: "Thứ 5 · Rà C1–C2 & viết câu mới cho Leech", minutes: 10, tier: "core", action_hint: "quiz" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Writing: Rà Top 3 lỗi → Viết 1 đoạn Body C1 & Chấm AI (18p)", meta: "Hedging + Counter-argument chuẩn 7.5–8.0", minutes: 18, tier: "core", action_hint: "writing" },
                QuestSeed { title: "Viết trọn bài Writing Task 2 (40p)", meta: "Bonus · Bấm giờ chuẩn phòng thi", minutes: 40, tier: "bonus", action_hint: "writing" },
            ],
            Weekday::Fri => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 6 · Cụm C1–C2 & Hedging", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Reading dạng khó (T/F/NG P3, Heading — 16p) → Sentence Mining 2 câu", meta: "Đích 36–39/40 · Nạp cụm C1–C2 vào SRS", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Ôn toàn bộ Bảng Paraphrase Reading trong tuần", meta: "Bonus · Rà bẫy suy luận Not Given", minutes: 10, tier: "bonus", action_hint: "paraphrase" },
            ],
            Weekday::Sat => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 7 · Cụm C1–C2 & Hedging", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Listening (Map/Diagram hoặc Sec 2+3 — 16p) → nhập điểm & lưu lỗi", meta: "Mục tiêu 36–39/40 câu · Sạch lỗi -s/-ed", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Full Mock Listening + Reading bấm giờ (120p) trên Study4", meta: "Bonus · Kiểm tra sức bền 36–39/40", minutes: 120, tier: "bonus", action_hint: "study4" },
            ],
            Weekday::Sun => vec![
                QuestSeed { title: "Quiz Tổng Kết Tuần: Từ vựng C1 + Sửa lỗi (10p)", meta: "Chủ Nhật · Gõ sửa lỗi & kiểm tra trí nhớ", minutes: 10, tier: "core", action_hint: "quiz" },
                QuestSeed { title: "Writing: Rà Top 3 lỗi → Viết Task 1 hoặc Mở+Kết Task 2 & Chấm AI (20p)", meta: "Kiểm tra lặp Top 3 · chuẩn Band 7.5–8.0", minutes: 20, tier: "core", action_hint: "writing" },
                QuestSeed { title: "Phòng Luyện Nói: Cue Card thứ 2 của tuần (Part 3 trừu tượng)", meta: "Bonus · Giữ phản xạ C1–C2 cuối tuần", minutes: 18, tier: "bonus", action_hint: "speaking" },
            ],
        }
    } else {
        match weekday {
            Weekday::Mon => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 2 · Giữ chuỗi & xử lý từ cứng đầu", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Reading: 1 Passage bấm giờ 16p → nạp 2 cặp Bảng Paraphrase", meta: "Đích 30–34/40 · Săn từ đồng nghĩa thật", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Làm thêm 1 Passage Reading + Sentence Mining 2 câu vào SRS", meta: "Bonus · Tăng tốc đọc hiểu", minutes: 18, tier: "bonus", action_hint: "paraphrase" },
            ],
            Weekday::Tue => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 3 · Giữ chuỗi & xử lý từ cứng đầu", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Phòng Luyện Nói 20p: 1 Cue Card tuần (A.R.E.A) + Chấm AI/Prompt", meta: "Thi thật 2:00+Part 3 hoặc Phản xạ 4-3-2", minutes: 20, tier: "core", action_hint: "speaking" },
                QuestSeed { title: "Nghe chép câu Dictation thêm 5 câu bẫy nối âm & -s/-ed", meta: "Bonus · Luyện tinh tai offline", minutes: 12, tier: "bonus", action_hint: "dictation" },
            ],
            Weekday::Wed => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 4 · Giữ chuỗi & xử lý từ cứng đầu", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Listening (Sec 3/4 — 14p) + Nghe chép Dictation 3 câu (6p)", meta: "Bắt âm cuối -s/-ed & từ đổi ý", minutes: 20, tier: "core", action_hint: "dictation" },
                QuestSeed { title: "Chép chính tả lại đoạn nghe sai trên Study4", meta: "Bonus · Soi transcript tránh bẫy", minutes: 15, tier: "bonus", action_hint: "study4" },
            ],
            Weekday::Thu => vec![
                QuestSeed { title: "Quiz Giữa Tuần & Chữa 🔥 Từ cứng đầu (10p)", meta: "Thứ 5 · Kiểm tra trí nhớ & viết câu mới cho Leech", minutes: 10, tier: "core", action_hint: "quiz" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Writing: Rà Top 3 lỗi → Viết 1 đoạn Body & Chấm AI (18p)", meta: "Chống lặp lỗi cũ · mục tiêu ≥ 6.0–6.5", minutes: 18, tier: "core", action_hint: "writing" },
                QuestSeed { title: "Viết trọn bài Writing Task 2 (40p)", meta: "Bonus · Bấm giờ chuẩn phòng thi", minutes: 40, tier: "bonus", action_hint: "writing" },
            ],
            Weekday::Fri => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 6 · Giữ chuỗi & xử lý từ cứng đầu", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Reading dạng khó (T/F/NG, Heading — 16p) → Sentence Mining 2 câu", meta: "Đích 30–34/40 · Nạp từ mới vào SRS", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Ôn toàn bộ Bảng Paraphrase Reading trong tuần", meta: "Bonus · Nhớ bẫy từng dạng bài", minutes: 10, tier: "bonus", action_hint: "paraphrase" },
            ],
            Weekday::Sat => vec![
                QuestSeed { title: "Ôn SRS 8p (Audio-First / Cloze)", meta: "Thứ 7 · Giữ chuỗi & xử lý từ cứng đầu", minutes: 8, tier: "core", action_hint: "srs" },
                QuestSeed { title: "Micro-Check 2p (5 câu rà từ/lỗi/paraphrase)", meta: "Tổng hợp 3 nguồn chống quên", minutes: 2, tier: "core", action_hint: "micro_check" },
                QuestSeed { title: "Study4 Listening (Map/Diagram hoặc Sec 2+3 — 16p) → nhập điểm & lưu lỗi chính tả", meta: "Đích 30–34/40 câu", minutes: 16, tier: "core", action_hint: "study4" },
                QuestSeed { title: "Full Mock Listening + Reading bấm giờ (120p) trên Study4", meta: "Bonus · Rèn sức bền phòng thi", minutes: 120, tier: "bonus", action_hint: "study4" },
            ],
            Weekday::Sun => vec![
                QuestSeed { title: "Quiz Tổng Kết Tuần: Từ vựng + Sửa lỗi (10p)", meta: "Chủ Nhật · Gõ sửa lỗi Sổ lỗi + rà từ", minutes: 10, tier: "core", action_hint: "quiz" },
                QuestSeed { title: "Writing: Rà Top 3 lỗi → Viết Task 1 hoặc Mở+Kết Task 2 & Chấm AI (20p)", meta: "Kiểm tra cảnh báo lặp Top 3 lỗi", minutes: 20, tier: "core", action_hint: "writing" },
                QuestSeed { title: "Phòng Luyện Nói: Cue Card thứ 2 của tuần", meta: "Bonus · Tự tin trước tuần mới", minutes: 18, tier: "bonus", action_hint: "speaking" },
            ],
        }
    }
}

pub fn ensure_today_quests(conn: &Connection, today: &str) -> Result<(), String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts'",
            params![today],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count == 0 {
        let tmpl_json: String = conn
            .query_row(
                "SELECT COALESCE(daily_quests_template_json, '[]') FROM settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "[]".to_string());
        let custom_quests: Vec<ParsedQuest> = serde_json::from_str(&tmpl_json).unwrap_or_default();

        if !custom_quests.is_empty() {
            for q in custom_quests {
                let tier = if q.tier == "bonus" { "bonus" } else { "core" };
                let hint = if q.action_hint.trim().is_empty() {
                    infer_quest_action_hint(&q.title)
                } else {
                    q.action_hint.clone()
                };
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0, ?5, ?6)",
                    params![today, q.title, q.meta, q.minutes, tier, hint],
                )
                .map_err(|e| e.to_string())?;
            }
        } else {
            let target_band: f64 = conn
                .query_row(
                    "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(7.0);
            let is_80 = target_band >= 7.5;
            let weekday = NaiveDate::parse_from_str(today, "%Y-%m-%d")
                .map(|d| d.weekday())
                .unwrap_or_else(|_| Local::now().date_naive().weekday());
            let defaults = default_quests_for_weekday(weekday, is_80);
            for q in defaults {
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0, ?5, ?6)",
                    params![today, q.title, q.meta, q.minutes, q.tier, q.action_hint],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

pub fn get_settings_raw(conn: &Connection) -> Result<Settings, String> {
    conn.query_row(
        "SELECT theme, gemini_api_key, gemini_model, exam_date, reminder_time, autostart, target_band,
                onboarding_done, writing_checklist_json, writing_focus_json, skill_targets_json, phase_labels_json,
                COALESCE(tts_accent, 'en-GB'),
                COALESCE(auto_refill_enabled, 1),
                COALESCE(last_ai_refill_at, ''),
                COALESCE(last_ai_refill_topic, '')
         FROM settings WHERE id = 1",
        [],
        |r| {
            let raw_theme: String = r.get(0)?;
            let chk_json: String = r.get(8)?;
            let foc_json: String = r.get(9)?;
            let tgt_json: String = r.get(10)?;
            let ph_json: String = r.get(11)?;
            let raw_accent: String = r.get(12)?;

            let mut writing_checklist: Vec<String> =
                serde_json::from_str(&chk_json).unwrap_or_default();
            if writing_checklist.is_empty() {
                writing_checklist = default_writing_checklist();
            }
            let mut writing_focus_notes: Vec<String> =
                serde_json::from_str(&foc_json).unwrap_or_default();
            if writing_focus_notes.is_empty() {
                writing_focus_notes = default_writing_focus_notes();
            }
            let mut skill_targets: BTreeMap<String, f64> =
                serde_json::from_str(&tgt_json).unwrap_or_default();
            if skill_targets.is_empty() {
                skill_targets = default_skill_targets();
            }
            let mut phase_labels: BTreeMap<i64, String> =
                serde_json::from_str(&ph_json).unwrap_or_default();
            if phase_labels.is_empty() {
                phase_labels = default_phase_labels();
            }

            Ok(Settings {
                theme: normalize_theme(&raw_theme),
                gemini_api_key: r.get(1)?,
                gemini_model: r.get(2)?,
                exam_date: r.get(3)?,
                reminder_time: r.get(4)?,
                autostart: r.get::<_, i64>(5)? != 0,
                target_band: r.get(6)?,
                onboarding_done: r.get::<_, i64>(7)? != 0,
                tts_accent: normalize_tts_accent(&raw_accent),
                writing_checklist,
                writing_focus_notes,
                skill_targets,
                phase_labels,
                auto_refill_enabled: r.get::<_, i64>(13)? != 0,
                last_ai_refill_at: r.get(14)?,
                last_ai_refill_topic: r.get(15)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn get_settings_public(conn: &Connection) -> Result<SettingsPublic, String> {
    let s = get_settings_raw(conn)?;
    Ok(SettingsPublic {
        theme: s.theme,
        has_api_key: !s.gemini_api_key.trim().is_empty(),
        api_key_masked: mask_key(&s.gemini_api_key),
        gemini_model: s.gemini_model,
        exam_date: s.exam_date,
        reminder_time: s.reminder_time,
        autostart: s.autostart,
        target_band: s.target_band,
        onboarding_done: s.onboarding_done,
        tts_accent: s.tts_accent,
        writing_checklist: s.writing_checklist,
        writing_focus_notes: s.writing_focus_notes,
        phase_labels: s.phase_labels,
        auto_refill_enabled: s.auto_refill_enabled,
        last_ai_refill_at: s.last_ai_refill_at,
        last_ai_refill_topic: s.last_ai_refill_topic,
    })
}

pub fn clear_gemini_api_key(conn: &Connection) -> Result<SettingsPublic, String> {
    conn.execute("UPDATE settings SET gemini_api_key = '' WHERE id = 1", [])
        .map_err(|e| e.to_string())?;
    get_settings_public(conn)
}

pub fn update_settings(
    conn: &Connection,
    theme: Option<String>,
    gemini_api_key: Option<String>,
    gemini_model: Option<String>,
    exam_date: Option<String>,
    reminder_time: Option<String>,
    autostart: Option<bool>,
    target_band: Option<f64>,
    onboarding_done: Option<bool>,
    tts_accent: Option<String>,
    auto_refill_enabled: Option<bool>,
) -> Result<SettingsPublic, String> {
    let cur = get_settings_raw(conn)?;
    let new_theme = theme
        .map(|t| normalize_theme(&t))
        .unwrap_or(cur.theme);
    let new_key = match gemini_api_key {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        Some(_) => cur.gemini_api_key,
        None => cur.gemini_api_key,
    };
    let new_model = gemini_model
        .filter(|m| !m.trim().is_empty())
        .unwrap_or(cur.gemini_model);
    let new_exam = exam_date
        .filter(|d| !d.trim().is_empty())
        .unwrap_or(cur.exam_date);
    let new_rem = reminder_time
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(cur.reminder_time);
    let new_auto = autostart.unwrap_or(cur.autostart);
    let new_target = target_band.unwrap_or(cur.target_band);
    let new_onboarding = onboarding_done.unwrap_or(cur.onboarding_done);
    let new_accent = tts_accent
        .map(|a| normalize_tts_accent(&a))
        .unwrap_or(cur.tts_accent);
    let new_auto_refill = auto_refill_enabled.unwrap_or(cur.auto_refill_enabled);

    conn.execute(
        "UPDATE settings
         SET theme = ?1, gemini_api_key = ?2, gemini_model = ?3, exam_date = ?4,
             reminder_time = ?5, autostart = ?6, target_band = ?7, onboarding_done = ?8, tts_accent = ?9,
             auto_refill_enabled = ?10
         WHERE id = 1",
        params![
            new_theme,
            new_key,
            new_model,
            new_exam,
            new_rem,
            if new_auto { 1 } else { 0 },
            new_target,
            if new_onboarding { 1 } else { 0 },
            new_accent,
            if new_auto_refill { 1 } else { 0 }
        ],
    )
    .map_err(|e| e.to_string())?;

    get_settings_public(conn)
}

pub fn toggle_quest(conn: &Connection, quest_id: i64) -> Result<(), String> {
    let (date, minutes, done, tier): (String, i64, i64, String) = conn
        .query_row(
            "SELECT date, minutes, done, COALESCE(tier, 'core') FROM daily_quests WHERE id = ?1",
            params![quest_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| e.to_string())?;

    let new_done = if done == 0 { 1 } else { 0 };
    conn.execute(
        "UPDATE daily_quests SET done = ?1 WHERE id = ?2",
        params![new_done, quest_id],
    )
    .map_err(|e| e.to_string())?;

    // Only Core quests contribute to streak & core progress
    if tier != "bonus" {
        let delta_min = if new_done == 1 { minutes } else { -minutes };
        let delta_q = if new_done == 1 { 1 } else { -1 };
        adjust_study_day(conn, &date, delta_min, delta_q)?;
    }
    Ok(())
}

pub fn auto_tick_quest_by_hint(conn: &Connection, today: &str, hint: &str) -> Result<bool, String> {
    let updated = conn
        .execute(
            "UPDATE daily_quests SET done = 1
             WHERE date = ?1 AND track = 'ielts' AND done = 0 AND COALESCE(tier, 'core') = 'core' AND action_hint = ?2",
            params![today, hint],
        )
        .map_err(|e| e.to_string())?;
    if updated > 0 {
        let quests_done_today: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts' AND done = 1 AND COALESCE(tier, 'core') = 'core'",
                params![today],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let existing: Option<i64> = conn
            .query_row("SELECT minutes FROM study_days WHERE date = ?1", params![today], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        if existing.is_some() {
            conn.execute(
                "UPDATE study_days SET quests_done = ?1 WHERE date = ?2",
                params![quests_done_today, today],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "INSERT INTO study_days (date, track, minutes, quests_done) VALUES (?1, 'ielts', 10, ?2)",
                params![today, quests_done_today],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(updated > 0)
}

pub fn adjust_study_day(
    conn: &Connection,
    date: &str,
    delta_minutes: i64,
    delta_quests: i64,
) -> Result<(), String> {
    let existing: Option<(i64, i64)> = conn
        .query_row(
            "SELECT minutes, quests_done FROM study_days WHERE date = ?1",
            params![date],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match existing {
        Some((m, q)) => {
            let nm = (m + delta_minutes).max(0);
            let nq = (q + delta_quests).max(0);
            conn.execute(
                "UPDATE study_days SET minutes = ?1, quests_done = ?2 WHERE date = ?3",
                params![nm, nq, date],
            )
            .map_err(|e| e.to_string())?;
        }
        None => {
            let nm = delta_minutes.max(0);
            let nq = delta_quests.max(0);
            conn.execute(
                "INSERT INTO study_days (date, track, minutes, quests_done) VALUES (?1, 'ielts', ?2, ?3)",
                params![date, nm, nq],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn calculate_streak(conn: &Connection, today: NaiveDate) -> Result<i64, String> {
    let mut streak = 0_i64;
    let mut check_date = today;

    let today_str = today.format("%Y-%m-%d").to_string();
    let today_active: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CASE WHEN minutes >= 10 OR quests_done >= 1 THEN 1 ELSE 0 END), 0)
             FROM study_days WHERE date = ?1",
            params![today_str],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if today_active == 0 {
        check_date = today - Duration::days(1);
    }

    loop {
        let ds = check_date.format("%Y-%m-%d").to_string();
        let active: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(CASE WHEN minutes >= 10 OR quests_done >= 1 THEN 1 ELSE 0 END), 0)
                 FROM study_days WHERE date = ?1",
                params![ds],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if active == 1 {
            streak += 1;
            check_date -= Duration::days(1);
        } else {
            break;
        }
    }

    Ok(streak)
}

pub fn get_heatmap(conn: &Connection, today: NaiveDate, days: i64) -> Result<Vec<StudyDay>, String> {
    let start = today - Duration::days(days - 1);
    let mut out = Vec::with_capacity(days as usize);
    for i in 0..days {
        let d = start + Duration::days(i);
        let ds = d.format("%Y-%m-%d").to_string();
        let row: Option<(i64, i64)> = conn
            .query_row(
                "SELECT minutes, quests_done FROM study_days WHERE date = ?1",
                params![ds],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let (minutes, quests_done) = row.unwrap_or((0, 0));
        out.push(StudyDay {
            date: ds,
            track: "ielts".to_string(),
            minutes,
            quests_done,
        });
    }
    Ok(out)
}

fn map_vocab_card(r: &rusqlite::Row<'_>) -> rusqlite::Result<VocabCard> {
    Ok(VocabCard {
        id: r.get(0)?,
        track: r.get(1)?,
        topic: r.get(2)?,
        word: r.get(3)?,
        meaning: r.get(4)?,
        example: r.get(5)?,
        ease: r.get(6)?,
        interval_days: r.get(7)?,
        reps: r.get(8)?,
        due_date: r.get(9)?,
        lapses: r.get::<_, i64>(10)?,
        hidden: r.get::<_, i64>(11)? != 0,
        is_user_card: r.get::<_, i64>(12)? != 0,
        source: r.get::<_, Option<String>>(13)?.unwrap_or_else(|| "seed".to_string()),
    })
}

pub fn get_due_vocab(
    conn: &Connection,
    today: &str,
    only_leech: Option<bool>,
) -> Result<Vec<VocabCard>, String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);

    let leech_only = only_leech.unwrap_or(false);
    let order_sql = if leech_only {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
         FROM vocab_cards
         WHERE COALESCE(hidden, 0) = 0 AND COALESCE(lapses, 0) >= 3
         ORDER BY lapses DESC, due_date ASC, id DESC
         LIMIT 40"
    } else if target_band >= 7.5 {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
         FROM vocab_cards
         WHERE due_date <= ?1 AND COALESCE(hidden, 0) = 0
         ORDER BY COALESCE(is_user_card, 0) DESC,
                  (CASE WHEN COALESCE(lapses, 0) >= 3 THEN 0 ELSE 1 END) ASC,
                  (CASE WHEN reps = 0 AND ease < 2.5 THEN 0 ELSE 1 END) ASC,
                  due_date ASC, ease ASC, id DESC
         LIMIT 40"
    } else {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
         FROM vocab_cards
         WHERE due_date <= ?1 AND COALESCE(hidden, 0) = 0
         ORDER BY COALESCE(is_user_card, 0) DESC,
                  (CASE WHEN COALESCE(lapses, 0) >= 3 THEN 0 ELSE 1 END) ASC,
                  (CASE WHEN reps = 0 AND ease < 2.5 THEN 0 ELSE 1 END) ASC,
                  due_date ASC, ease ASC, id ASC
         LIMIT 40"
    };

    let mut stmt = conn.prepare(order_sql).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    if leech_only {
        let rows = stmt
            .query_map([], map_vocab_card)
            .map_err(|e| e.to_string())?;
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
    } else {
        let rows = stmt
            .query_map(params![today], map_vocab_card)
            .map_err(|e| e.to_string())?;
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
    }
    Ok(out)
}

pub fn add_vocab_card(
    conn: &Connection,
    word: &str,
    meaning: &str,
    example: &str,
    topic: &str,
    today: &str,
) -> Result<VocabCard, String> {
    let w = word.trim();
    let m = meaning.trim();
    if w.is_empty() || m.is_empty() {
        return Err("Vui lòng nhập đầy đủ Từ và Nghĩa.".to_string());
    }
    let ex = if example.trim().is_empty() {
        format!("Example with {w}.")
    } else {
        example.trim().to_string()
    };
    let tp = if topic.trim().is_empty() {
        "Sentence Mining".to_string()
    } else {
        topic.trim().to_string()
    };

    conn.execute(
        "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, lapses, hidden, is_user_card, source)
         VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 0, 0, 1, 'user')",
        params![tp, w, m, ex, today],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
         FROM vocab_cards WHERE id = ?1",
        params![id],
        map_vocab_card,
    )
    .map_err(|e| e.to_string())
}

fn sentence_contains_vocab_root(sentence: &str, card_word: &str) -> bool {
    let sent_low = sentence.to_lowercase();
    let primary = card_word
        .split('(')
        .next()
        .unwrap_or(card_word)
        .split('/')
        .next()
        .unwrap_or(card_word)
        .trim()
        .to_lowercase();
    if primary.is_empty() {
        return true;
    }
    if sent_low.contains(&primary) {
        return true;
    }
    let words: Vec<&str> = primary
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return true;
    }
    // Check main content word (longest or first >= 4 chars) with >=4 char stem
    let target = words
        .iter()
        .max_by_key(|w| w.len())
        .copied()
        .unwrap_or(words[0]);
    let stem_len = target.chars().count().min(4);
    let stem: String = target.chars().take(stem_len).collect();
    !stem.is_empty() && sent_low.contains(&stem)
}

pub fn cure_leech_card(
    conn: &Connection,
    card_id: i64,
    user_sentence: &str,
    today: &str,
) -> Result<VocabCard, String> {
    let sent = user_sentence.trim();
    if sent.len() < 6 {
        return Err("Hãy đặt 1 câu tiếng Anh trọn vẹn (tối thiểu 6 ký tự) chứa từ này.".to_string());
    }
    let card_word: Option<String> = conn
        .query_row(
            "SELECT word FROM vocab_cards WHERE id = ?1",
            params![card_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some(word) = card_word else {
        return Err("Không tìm thấy thẻ từ vựng cần chữa.".to_string());
    };
    if !sentence_contains_vocab_root(sent, &word) {
        let short_w = word.split('(').next().unwrap_or(&word).trim();
        return Err(format!(
            "Câu mới cần chứa từ/cụm từ \"{short_w}\" (hoặc dạng chia từ của nó) để lưu thành thẻ Cloze."
        ));
    }

    conn.execute(
        "UPDATE vocab_cards
         SET example = ?1, lapses = 0, reps = 0, interval_days = 0, due_date = ?2, is_user_card = 1
         WHERE id = ?3",
        params![sent, today, card_id],
    )
    .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
         FROM vocab_cards WHERE id = ?1",
        params![card_id],
        map_vocab_card,
    )
    .map_err(|e| e.to_string())
}

pub fn toggle_hide_vocab_card(conn: &Connection, card_id: i64, hidden: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE vocab_cards SET hidden = ?1 WHERE id = ?2",
        params![if hidden { 1 } else { 0 }, card_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_vocab_card(conn: &Connection, card_id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM vocab_cards WHERE id = ?1", params![card_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn next_rand(seed: &mut u64) -> u64 {
    let mut x = *seed;
    if x == 0 {
        x = 0x9e37_79b9_7f4a_7c15;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    x
}

fn shuffle_slice<T>(items: &mut [T], seed: &mut u64) {
    let len = items.len();
    if len <= 1 {
        return;
    }
    for i in (1..len).rev() {
        let j = (next_rand(seed) as usize) % (i + 1);
        items.swap(i, j);
    }
}

fn make_cloze_sentence(example: &str, word: &str) -> Option<String> {
    let clean_word = word.split('(').next().unwrap_or(word).split('/').next().unwrap_or(word).trim();
    if clean_word.is_empty() {
        return None;
    }
    let ex_lower = example.to_lowercase();
    let w_lower = clean_word.to_lowercase();
    if let Some(byte_idx) = ex_lower.find(&w_lower) {
        if example.is_char_boundary(byte_idx) && example.is_char_boundary(byte_idx + w_lower.len()) {
            let mut out = String::with_capacity(example.len());
            out.push_str(&example[..byte_idx]);
            out.push_str("________");
            out.push_str(&example[byte_idx + w_lower.len()..]);
            return Some(out);
        }
    }
    // Try inflected first word (e.g. "play a pivotal role in" -> "plays a pivotal role in")
    let parts: Vec<&str> = w_lower.split_whitespace().collect();
    if !parts.is_empty() {
        let first = parts[0];
        let rest = if parts.len() > 1 {
            format!(" {}", parts[1..].join(" "))
        } else {
            String::new()
        };
        let mut variants = vec![
            format!("{first}s{rest}"),
            format!("{first}es{rest}"),
            format!("{first}ed{rest}"),
            format!("{first}d{rest}"),
            format!("{first}ing{rest}"),
        ];
        if let Some(stem) = first.strip_suffix('e') {
            variants.push(format!("{stem}ing{rest}"));
            variants.push(format!("{stem}ed{rest}"));
        }
        if let Some(stem) = first.strip_suffix('y') {
            variants.push(format!("{stem}ies{rest}"));
            variants.push(format!("{stem}ied{rest}"));
        }
        for v in variants {
            if let Some(byte_idx) = ex_lower.find(&v) {
                if example.is_char_boundary(byte_idx) && example.is_char_boundary(byte_idx + v.len()) {
                    let mut out = String::with_capacity(example.len());
                    out.push_str(&example[..byte_idx]);
                    out.push_str("________");
                    out.push_str(&example[byte_idx + v.len()..]);
                    return Some(out);
                }
            }
        }
    }
    None
}

fn make_typing_hint(word: &str) -> String {
    let primary = word
        .split('(')
        .next()
        .unwrap_or(word)
        .split('/')
        .next()
        .unwrap_or(word)
        .trim();
    primary
        .split_whitespace()
        .map(|tok| {
            tok.split('-')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => {
                            let rest: String = chars
                                .map(|c| if c.is_alphanumeric() { '_' } else { c })
                                .collect();
                            format!("{first}{rest}")
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join("-")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn get_vocab_quiz_status(conn: &Connection, today_naive: NaiveDate) -> Result<VocabQuizStatus, String> {
    let today = today_naive.format("%Y-%m-%d").to_string();
    let total_quizzes: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocab_quizzes", [], |r| r.get(0))
        .unwrap_or(0);

    let last_row: Option<(String, i64, i64)> = conn
        .query_row(
            "SELECT taken_at, correct_count, total_questions FROM vocab_quizzes ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (last_quiz_at, last_quiz_score) = match last_row {
        Some((taken, corr, tot)) => (Some(taken), Some(format!("{corr}/{tot}"))),
        None => (None, None),
    };

    let days_since_last_quiz = last_quiz_at
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s.get(..10).unwrap_or(s), "%Y-%m-%d").ok())
        .map(|d| (today_naive - d).num_days().max(0));

    let done_today = last_quiz_at
        .as_deref()
        .map(|s| s.starts_with(&today))
        .unwrap_or(false);
    let is_scheduled_day = matches!(today_naive.weekday(), Weekday::Thu | Weekday::Sun);
    let study_days_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM study_days WHERE minutes > 0 OR quests_done > 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let (quiz_due, quiz_due_reason) = if done_today {
        (false, "Đã hoàn thành Quiz từ vựng hôm nay ✓".to_string())
    } else if is_scheduled_day {
        let msg = if today_naive.weekday() == Weekday::Thu {
            "Đến lịch Quiz định kỳ Thứ 5 (kiểm tra chống quên)"
        } else {
            "Đến lịch Quiz tổng kết Chủ Nhật (rà từ hay quên)"
        };
        (true, msg.to_string())
    } else if let Some(days) = days_since_last_quiz {
        if days >= 3 {
            (true, format!("Đã {days} ngày chưa kiểm tra trí nhớ từ vựng"))
        } else {
            (false, "Gợi ý kiểm tra nhanh 10 từ để chống quên".to_string())
        }
    } else if study_days_count >= 1 {
        (true, "Gợi ý kiểm tra nhanh 10 từ để chống quên".to_string())
    } else {
        (false, "Gợi ý kiểm tra nhanh 10 từ để chống quên".to_string())
    };

    Ok(VocabQuizStatus {
        last_quiz_at,
        days_since_last_quiz,
        last_quiz_score,
        total_quizzes,
        quiz_due,
        quiz_due_reason,
    })
}

fn is_quiz_eligible_card(card: &VocabCard) -> bool {
    if card.hidden {
        return false;
    }
    let w = card.word.trim();
    if w.is_empty()
        || w.contains(" vs ")
        || w.contains(" / ")
        || w.contains('(')
        || w.contains(')')
    {
        return false;
    }
    let topic_lower = card.topic.to_lowercase();
    if topic_lower.contains("word family") || topic_lower.contains("nhầm từ") {
        return false;
    }
    let w_lower = w.to_lowercase();
    if w_lower.len() >= 3 && card.meaning.to_lowercase().contains(&w_lower) {
        return false;
    }
    true
}

pub fn generate_vocab_quiz(
    conn: &Connection,
    count: Option<i64>,
    mode: Option<String>,
) -> Result<Vec<QuizQuestion>, String> {
    let target_count = count.unwrap_or(10).clamp(1, 20) as usize;
    let mode_norm = match mode.as_deref().unwrap_or("mixed").trim() {
        "mcq" => "mcq",
        "typing" => "typing",
        _ => "mixed",
    };

    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let is_80 = target_band >= 7.5;

    let mut stmt = conn
        .prepare(
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
             FROM vocab_cards
             WHERE COALESCE(hidden, 0) = 0",
        )
        .map_err(|e| e.to_string())?;
    let all_cards: Vec<VocabCard> = stmt
        .query_map([], map_vocab_card)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if all_cards.is_empty() {
        return Ok(Vec::new());
    }

    let eligible_cards: Vec<VocabCard> = all_cards
        .iter()
        .filter(|c| is_quiz_eligible_card(c))
        .cloned()
        .collect();
    let active_cards = if eligible_cards.len() >= 4 {
        eligible_cards
    } else {
        all_cards
    };

    let mut studied: Vec<VocabCard> = active_cards.iter().filter(|c| c.reps >= 1 || c.lapses > 0).cloned().collect();
    let mut unstudied: Vec<VocabCard> = active_cards.iter().filter(|c| c.reps == 0 && c.lapses == 0).cloned().collect();

    studied.sort_by(|a, b| {
        b.lapses
            .cmp(&a.lapses)
            .then_with(|| a.ease.partial_cmp(&b.ease).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| b.interval_days.cmp(&a.interval_days))
            .then_with(|| a.due_date.cmp(&b.due_date))
    });

    if is_80 {
        unstudied.sort_by(|a, b| b.id.cmp(&a.id));
    } else {
        unstudied.sort_by(|a, b| a.id.cmp(&b.id));
    }

    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(123456789)
        ^ (active_cards.len() as u64 * 0x9e37_79b9);

    let mut pool = Vec::with_capacity(active_cards.len());
    pool.extend(studied);
    pool.extend(unstudied);

    let pick_n = target_count.min(pool.len());
    let candidate_window = (pick_n + 4).min(pool.len());
    shuffle_slice(&mut pool[..candidate_window], &mut seed);

    let typing_count = match mode_norm {
        "typing" => pick_n,
        "mcq" => 0,
        _ => {
            if pick_n >= 4 {
                (pick_n * 3 + 5) / 10
            } else {
                0
            }
        }
    };
    let mcq_count = pick_n.saturating_sub(typing_count);

    let mut selected = pool[..pick_n].to_vec();
    if mode_norm == "mixed" && typing_count > 0 {
        let mut long_cards: Vec<VocabCard> = Vec::new();
        let mut short_cards: Vec<VocabCard> = Vec::new();
        for c in selected {
            if c.word.split_whitespace().count() <= 3 {
                short_cards.push(c);
            } else {
                long_cards.push(c);
            }
        }
        let mut ordered = Vec::with_capacity(pick_n);
        if short_cards.len() >= typing_count {
            let split_at = short_cards.len() - typing_count;
            ordered.extend(long_cards);
            ordered.extend(short_cards.drain(..split_at));
            ordered.extend(short_cards);
        } else {
            ordered.extend(long_cards);
            ordered.extend(short_cards);
        }
        selected = ordered;
    }

    let mut questions = Vec::with_capacity(pick_n);
    for (idx, card) in selected.iter().enumerate() {
        let raw_cloze = make_cloze_sentence(&card.example, &card.word);
        let desired_type = if idx >= mcq_count {
            "typing"
        } else if idx % 2 == 1 && raw_cloze.is_some() {
            "cloze_mcq"
        } else {
            "meaning_mcq"
        };
        let cloze_sentence = if desired_type == "meaning_mcq" {
            None
        } else {
            raw_cloze
        };

        let options = if desired_type == "typing" {
            Vec::new()
        } else {
            let mut distractors: Vec<String> = active_cards
                .iter()
                .filter(|c| c.id != card.id && c.word != card.word)
                .map(|c| c.word.clone())
                .collect();
            distractors.sort();
            distractors.dedup();
            shuffle_slice(&mut distractors, &mut seed);
            let mut opts: Vec<String> = distractors.into_iter().take(3).collect();
            opts.push(card.word.clone());
            shuffle_slice(&mut opts, &mut seed);
            opts
        };

        let prompt_text = match desired_type {
            "cloze_mcq" => format!("Chọn từ/cụm từ đúng điền vào chỗ trống (Nghĩa: {})", card.meaning),
            "typing" => format!("Gõ từ/cụm từ tiếng Anh có nghĩa: {}", card.meaning),
            _ => format!("Cụm từ nào có nghĩa là: \"{}\"?", card.meaning),
        };

        questions.push(QuizQuestion {
            card_id: card.id,
            word: card.word.clone(),
            meaning: card.meaning.clone(),
            example: card.example.clone(),
            topic: card.topic.clone(),
            question_type: desired_type.to_string(),
            prompt_text,
            cloze_sentence,
            hint: make_typing_hint(&card.word),
            options,
        });
    }

    Ok(questions)
}

pub fn submit_vocab_quiz(
    conn: &Connection,
    mode: String,
    results: Vec<QuizAnswerInput>,
    today: NaiveDate,
) -> Result<QuizSubmitResult, String> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let tomorrow_str = (today + Duration::days(1)).format("%Y-%m-%d").to_string();
    let total_questions = results.len() as i64;
    let mut correct_count = 0_i64;
    let mut forgotten_cards = Vec::new();
    let mut forgotten_words = Vec::new();

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    for item in &results {
        let card: Option<(String, String, String, f64, i64)> = tx
            .query_row(
                "SELECT word, meaning, example, ease, reps FROM vocab_cards WHERE id = ?1",
                params![item.card_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let Some((word, meaning, example, ease, reps)) = card else {
            continue;
        };

        if !item.correct || item.gave_up {
            let new_ease = (ease - 0.25).max(1.3);
            tx.execute(
                "UPDATE vocab_cards
                 SET reps = 0, interval_days = 0, ease = ?1, due_date = ?2, lapses = COALESCE(lapses, 0) + 1
                 WHERE id = ?3",
                params![new_ease, today_str, item.card_id],
            )
            .map_err(|e| e.to_string())?;
            forgotten_words.push(word.clone());
            forgotten_cards.push(ForgottenVocabCard {
                card_id: item.card_id,
                word,
                meaning,
                example,
            });
        } else {
            correct_count += 1;
            if reps == 0 {
                tx.execute(
                    "UPDATE vocab_cards SET reps = 1, interval_days = 1, due_date = ?1 WHERE id = ?2",
                    params![tomorrow_str, item.card_id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                let new_ease = (ease + 0.05).min(2.8);
                tx.execute(
                    "UPDATE vocab_cards SET ease = ?1 WHERE id = ?2",
                    params![new_ease, item.card_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    let forgotten_json = serde_json::to_string(&forgotten_words).unwrap_or_else(|_| "[]".to_string());
    tx.execute(
        "INSERT INTO vocab_quizzes (taken_at, mode, total_questions, correct_count, forgotten_words)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![today_str, mode, total_questions, correct_count, forgotten_json],
    )
    .map_err(|e| e.to_string())?;

    let updated_quests = tx
        .execute(
            "UPDATE daily_quests SET done = 1
             WHERE date = ?1 AND track = 'ielts' AND done = 0
               AND (action_hint = 'quiz' OR lower(title) LIKE '%quiz%')",
            params![today_str],
        )
        .map_err(|e| e.to_string())?;
    let marked_quest_done = updated_quests > 0;

    let quests_done_today: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts' AND done = 1 AND COALESCE(tier, 'core') = 'core'",
            params![today_str],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let existing_day: Option<i64> = tx
        .query_row(
            "SELECT minutes FROM study_days WHERE date = ?1",
            params![today_str],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match existing_day {
        Some(m) => {
            tx.execute(
                "UPDATE study_days SET minutes = ?1, quests_done = ?2 WHERE date = ?3",
                params![m + 5, quests_done_today, today_str],
            )
            .map_err(|e| e.to_string())?;
        }
        None => {
            tx.execute(
                "INSERT INTO study_days (date, track, minutes, quests_done) VALUES (?1, 'ielts', 5, ?2)",
                params![today_str, quests_done_today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    let accuracy_pct = if total_questions > 0 {
        ((correct_count * 100) + (total_questions / 2)) / total_questions
    } else {
        0
    };

    Ok(QuizSubmitResult {
        total_questions,
        correct_count,
        accuracy_pct,
        forgotten_cards,
        marked_quest_done,
    })
}

pub fn review_vocab_card(
    conn: &Connection,
    card_id: i64,
    quality: u8,
    today: NaiveDate,
) -> Result<VocabCard, String> {
    let card: VocabCard = conn
        .query_row(
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
             FROM vocab_cards WHERE id = ?1",
            params![card_id],
            map_vocab_card,
        )
        .map_err(|e| e.to_string())?;

    let outcome = apply_sm2(card.ease, card.interval_days, card.reps, quality, today);
    let new_lapses = if quality == 0 {
        card.lapses + 1
    } else {
        card.lapses
    };
    conn.execute(
        "UPDATE vocab_cards SET ease = ?1, interval_days = ?2, reps = ?3, due_date = ?4, lapses = ?5 WHERE id = ?6",
        params![
            outcome.ease,
            outcome.interval_days,
            outcome.reps,
            outcome.due_date,
            new_lapses,
            card_id
        ],
    )
    .map_err(|e| e.to_string())?;

    let today_str = today.format("%Y-%m-%d").to_string();
    adjust_study_day(conn, &today_str, 1, 0)?;
    let _ = auto_tick_quest_by_hint(conn, &today_str, "srs")?;

    Ok(VocabCard {
        ease: outcome.ease,
        interval_days: outcome.interval_days,
        reps: outcome.reps,
        due_date: outcome.due_date,
        lapses: new_lapses,
        ..card
    })
}

pub fn generate_error_drills(
    conn: &Connection,
    count: Option<i64>,
) -> Result<Vec<ErrorDrillQuestion>, String> {
    let limit = count.unwrap_or(5).clamp(1, 20) as usize;
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(987654321);

    let mut e_stmt = conn
        .prepare(
            "SELECT id, category, wrong_text, right_text, count, created_at,
                    COALESCE(drill_mode, 'GRAMMAR_MCQ'), COALESCE(status, 'active'),
                    COALESCE(streak_distinct, 0), COALESCE(last_drill_id, 0),
                    COALESCE(last_context_tag, '')
             FROM error_logs
             WHERE EXISTS (SELECT 1 FROM error_drills d WHERE d.error_log_id = error_logs.id)
             ORDER BY (CASE WHEN COALESCE(status, 'active') = 'active' THEN 0 ELSE 1 END) ASC,
                      count DESC, COALESCE(streak_distinct, 0) ASC, id ASC
             LIMIT 30",
        )
        .map_err(|e| e.to_string())?;

    struct ErrRow {
        id: i64,
        category: String,
        wrong_text: String,
        right_text: String,
        streak_distinct: i64,
        status: String,
        last_drill_id: i64,
        last_context_tag: String,
    }

    let err_rows: Vec<ErrRow> = e_stmt
        .query_map([], |r| {
            Ok(ErrRow {
                id: r.get(0)?,
                category: r.get(1)?,
                wrong_text: r.get(2)?,
                right_text: r.get(3)?,
                streak_distinct: r.get(8)?,
                status: r.get(7)?,
                last_drill_id: r.get(9)?,
                last_context_tag: r.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for er in err_rows {
        if out.len() >= limit {
            break;
        }
        let mut d_stmt = conn
            .prepare(
                "SELECT id, drill_mode, context_tag, stem, options_json, answer, rationale_vi, COALESCE(source, 'seed')
                 FROM error_drills
                 WHERE error_log_id = ?1
                 ORDER BY
                   (CASE WHEN id != ?2 AND context_tag != ?3 THEN 0
                         WHEN id != ?2 THEN 1
                         ELSE 2 END) ASC,
                   id ASC",
            )
            .map_err(|e| e.to_string())?;
        let drills: Vec<(i64, String, String, String, String, String, String, String)> = d_stmt
            .query_map(params![er.id, er.last_drill_id, er.last_context_tag], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        if let Some((drill_id, drill_mode, context_tag, stem, opts_json, answer, rationale_vi, source)) = drills.into_iter().next() {
            let mut options: Vec<String> = serde_json::from_str(&opts_json).unwrap_or_default();
            let is_tfng = options.len() == 3
                && options.iter().any(|o| o == "TRUE")
                && options.iter().any(|o| o == "FALSE")
                && options.iter().any(|o| o == "NOT GIVEN");
            if options.len() >= 2 && !is_tfng {
                shuffle_slice(&mut options, &mut seed);
            }
            out.push(ErrorDrillQuestion {
                drill_id,
                error_log_id: er.id,
                category: er.category,
                drill_mode,
                context_tag,
                stem,
                options,
                answer,
                rationale_vi,
                wrong_text: er.wrong_text,
                right_text: er.right_text,
                streak_distinct: er.streak_distinct,
                status: er.status,
                source,
            });
        }
    }
    Ok(out)
}

fn apply_error_drill_outcome(
    conn: &Connection,
    error_log_id: i64,
    drill_id: i64,
    context_tag: &str,
    correct: bool,
) -> Result<(ErrorDrillSingleResult, String), String> {
    let row: Option<(i64, String, String, String)> = conn
        .query_row(
            "SELECT COALESCE(streak_distinct, 0), COALESCE(status, 'active'), COALESCE(last_context_tag, ''), category
             FROM error_logs WHERE id = ?1",
            params![error_log_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some((cur_streak, cur_status, prev_ctx, category)) = row else {
        return Err("Không tìm thấy mục Sổ lỗi tương ứng.".to_string());
    };

    let clean_ctx = context_tag.trim();
    let (new_streak, new_status, newly_mastered) = if correct {
        let next_streak = cur_streak + 1;
        let distinct_available: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT context_tag) FROM error_drills WHERE error_log_id = ?1",
                params![error_log_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        let ctx_ok = if distinct_available >= 2 && !prev_ctx.is_empty() {
            prev_ctx != clean_ctx || cur_streak >= 2
        } else {
            true
        };
        let became_mastered = next_streak >= 3 && ctx_ok;
        let next_status = if became_mastered {
            "mastered".to_string()
        } else {
            cur_status.clone()
        };
        (
            next_streak,
            next_status.clone(),
            became_mastered && cur_status != "mastered",
        )
    } else {
        (0, "active".to_string(), false)
    };

    conn.execute(
        "UPDATE error_logs
         SET streak_distinct = ?1, status = ?2, last_drill_id = ?3, last_context_tag = ?4
         WHERE id = ?5",
        params![new_streak, new_status, drill_id, clean_ctx, error_log_id],
    )
    .map_err(|e| e.to_string())?;

    Ok((
        ErrorDrillSingleResult {
            error_log_id,
            streak_distinct: new_streak,
            status: new_status,
            newly_mastered,
        },
        category,
    ))
}

pub fn submit_error_drill_result(
    conn: &Connection,
    error_log_id: i64,
    drill_id: i64,
    context_tag: &str,
    correct: bool,
) -> Result<ErrorDrillSingleResult, String> {
    let (res, _) = apply_error_drill_outcome(conn, error_log_id, drill_id, context_tag, correct)?;
    Ok(res)
}

pub fn submit_error_drill_session(
    conn: &Connection,
    results: Vec<ErrorDrillResultInput>,
    today: NaiveDate,
) -> Result<ErrorDrillSubmitResult, String> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let total_questions = results.len() as i64;
    let mut correct_count = 0_i64;
    let mut newly_mastered: Vec<String> = Vec::new();

    for r in &results {
        if r.correct {
            correct_count += 1;
        }
        if let Ok((outcome, cat)) =
            apply_error_drill_outcome(conn, r.error_log_id, r.drill_id, &r.context_tag, r.correct)
        {
            if outcome.newly_mastered && !newly_mastered.contains(&cat) {
                newly_mastered.push(cat);
            }
        }
    }

    let mut marked_quest_done = auto_tick_quest_by_hint(conn, &today_str, "quiz")?;
    if !marked_quest_done {
        let updated = conn
            .execute(
                "UPDATE daily_quests SET done = 1
                 WHERE date = ?1 AND track = 'ielts' AND done = 0
                   AND COALESCE(tier, 'core') = 'core' AND lower(title) LIKE '%quiz%'",
                params![today_str],
            )
            .map_err(|e| e.to_string())?;
        if updated > 0 {
            marked_quest_done = true;
            let qd: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts' AND done = 1 AND COALESCE(tier, 'core') = 'core'",
                    params![today_str],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let _ = conn.execute(
                "UPDATE study_days SET quests_done = ?1 WHERE date = ?2",
                params![qd, today_str],
            );
        }
    }
    adjust_study_day(conn, &today_str, 5, 0)?;

    let accuracy_pct = if total_questions > 0 {
        ((correct_count * 100) + (total_questions / 2)) / total_questions
    } else {
        0
    };

    Ok(ErrorDrillSubmitResult {
        total_questions,
        correct_count,
        accuracy_pct,
        newly_mastered,
        marked_quest_done,
    })
}

fn blank_word_in_sentence(sentence: &str, target: &str) -> String {
    let t = target.trim();
    if t.is_empty() {
        return format!("{sentence} (________)");
    }
    let s_low = sentence.to_lowercase();
    let t_low = t.to_lowercase();
    let mut start = 0;
    while let Some(rel) = s_low[start..].find(&t_low) {
        let idx = start + rel;
        let end = idx + t_low.len();
        let left_ok = idx == 0
            || !sentence[..idx]
                .chars()
                .next_back()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        let right_ok = end == sentence.len()
            || !sentence[end..]
                .chars()
                .next()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        if left_ok && right_ok && sentence.is_char_boundary(idx) && sentence.is_char_boundary(end) {
            return format!("{}________{}", &sentence[..idx], &sentence[end..]);
        }
        start = idx + 1;
        if start >= s_low.len() {
            break;
        }
    }
    format!("{sentence} [điền từ nghe đúng: ________]")
}

pub fn log_dictation_errors(
    conn: &Connection,
    sentence_id: i64,
    trap_focus: &str,
    sentence: &str,
    pairs: Vec<DictationErrorPair>,
) -> Result<(), String> {
    let ctx_tag = format!("dictation-{sentence_id}");
    let now = Local::now().format("%Y-%m-%d").to_string();

    for pair in pairs.into_iter().take(3) {
        let right = pair.right.trim();
        if right.is_empty() {
            continue;
        }
        let wrong = if pair.wrong.trim().is_empty() {
            "(nghe sót)"
        } else {
            pair.wrong.trim()
        };

        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM error_logs WHERE lower(wrong_text) = lower(?1) AND lower(right_text) = lower(?2)",
                params![wrong, right],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let error_log_id = if let Some(id) = existing_id {
            conn.execute(
                "UPDATE error_logs SET count = count + 1, status = 'active', streak_distinct = 0, drill_mode = 'TYPING' WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
            id
        } else {
            conn.execute(
                "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at, drill_mode, status, streak_distinct, source)
                 VALUES ('Listening · Chính tả', ?1, ?2, 1, ?3, 'TYPING', 'active', 0, 'dictation')",
                params![wrong, right, now],
            )
            .map_err(|e| e.to_string())?;
            conn.last_insert_rowid()
        };

        let stem = blank_word_in_sentence(sentence, right);
        let rationale = format!(
            "Bẫy {}: nghe rõ âm cuối/chính tả của \"{}\".",
            if trap_focus.trim().is_empty() {
                "Listening"
            } else {
                trap_focus.trim()
            },
            right
        );
        conn.execute(
            "INSERT INTO error_drills (error_log_id, drill_mode, context_tag, stem, options_json, answer, rationale_vi, source)
             SELECT ?1, 'TYPING', ?2, ?3, '[]', ?4, ?5, 'dictation'
             WHERE NOT EXISTS (SELECT 1 FROM error_drills WHERE error_log_id = ?1 AND lower(answer) = lower(?4) AND stem = ?3)",
            params![error_log_id, ctx_tag, stem, right, rationale],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn insert_ai_drills_for_error(
    conn: &Connection,
    error_log_id: i64,
    drills: &[TopicPackDrill],
) -> Result<i64, String> {
    let mut added = 0_i64;
    for d in drills {
        let stem = d.stem.trim();
        let answer = d.answer.trim();
        if stem.is_empty() || answer.is_empty() {
            continue;
        }
        let mode = match d.drill_mode.trim() {
            "TYPING" | "MINI_CASE" => d.drill_mode.trim(),
            _ => "GRAMMAR_MCQ",
        };
        let mut opts = d.options.clone();
        if mode != "TYPING" {
            if opts.len() < 3 || !opts.iter().any(|o| o.trim() == answer) {
                continue;
            }
        } else {
            opts.clear();
        }
        let ctx = if d.context_tag.trim().is_empty() {
            "general"
        } else {
            d.context_tag.trim()
        };
        let opts_json = serde_json::to_string(&opts).unwrap_or_else(|_| "[]".to_string());
        let changed = conn
            .execute(
                "INSERT INTO error_drills (error_log_id, drill_mode, context_tag, stem, options_json, answer, rationale_vi, source)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'gemini_cache'
                 WHERE NOT EXISTS (SELECT 1 FROM error_drills WHERE error_log_id = ?1 AND stem = ?4)",
                params![error_log_id, mode, ctx, stem, opts_json, answer, d.rationale_vi.trim()],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            added += 1;
        }
    }
    Ok(added)
}

pub fn generate_error_fix_quiz(
    conn: &Connection,
    count: Option<i64>,
) -> Result<Vec<ErrorFixQuestion>, String> {
    let limit = count.unwrap_or(5).clamp(1, 20);
    let mut stmt = conn
        .prepare(
            "SELECT id, category, wrong_text, right_text, count, created_at
             FROM error_logs
             ORDER BY count DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(ErrorFixQuestion {
                error_id: r.get(0)?,
                category: r.get(1)?,
                wrong_text: r.get(2)?,
                right_text: r.get(3)?,
                count: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn generate_micro_check(conn: &Connection) -> Result<Vec<MicroCheckItem>, String> {
    let mut items: Vec<MicroCheckItem> = Vec::new();

    // 1. Two easily-forgotten vocab cards (prioritizing lapses & low ease)
    let mut v_stmt = conn
        .prepare(
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0), COALESCE(source, 'seed')
             FROM vocab_cards
             WHERE COALESCE(hidden, 0) = 0
             ORDER BY COALESCE(lapses, 0) DESC, ease ASC, reps ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let all_vocab: Vec<VocabCard> = v_stmt
        .query_map([], map_vocab_card)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for card in all_vocab.iter().filter(|c| is_quiz_eligible_card(c)).take(2) {
        items.push(MicroCheckItem {
            item_type: "vocab".to_string(),
            ref_id: card.id,
            source_label: format!("SRS · {} (quên {} lần)", card.topic, card.lapses),
            prompt_text: format!("Gõ cụm từ tiếng Anh có nghĩa: “{}”", card.meaning),
            answer_text: card.word.clone(),
            hint: make_typing_hint(&card.word),
            meaning_or_note: card.example.clone(),
            options: Vec::new(),
        });
    }

    // 2. Two error_logs items (prioritizing active errors & highest count)
    let mut e_stmt = conn
        .prepare(
            "SELECT id, category, wrong_text, right_text, count, created_at,
                    COALESCE(drill_mode, 'GRAMMAR_MCQ'), COALESCE(status, 'active'),
                    COALESCE(streak_distinct, 0), COALESCE(last_drill_id, 0),
                    COALESCE(last_context_tag, '')
             FROM error_logs
             ORDER BY (CASE WHEN COALESCE(status, 'active') = 'active' THEN 0 ELSE 1 END) ASC,
                      count DESC, id DESC
             LIMIT 2",
        )
        .map_err(|e| e.to_string())?;
    let errs = e_stmt
        .query_map([], |r| {
            Ok(ErrorLog {
                id: r.get(0)?,
                category: r.get(1)?,
                wrong_text: r.get(2)?,
                right_text: r.get(3)?,
                count: r.get(4)?,
                created_at: r.get(5)?,
                drill_mode: r.get(6)?,
                status: r.get(7)?,
                streak_distinct: r.get(8)?,
                last_drill_id: r.get(9)?,
                last_context_tag: r.get(10)?,
                drill_count: 0,
            })
        })
        .map_err(|e| e.to_string())?;
    for err in errs.flatten() {
        items.push(MicroCheckItem {
            item_type: "error_fix".to_string(),
            ref_id: err.id,
            source_label: format!("Sổ lỗi · {} · {} lần", err.category, err.count),
            prompt_text: err.wrong_text.clone(),
            answer_text: err.right_text.clone(),
            hint: make_typing_hint(&err.right_text),
            meaning_or_note: format!("{} ({})", err.category, err.created_at),
            options: Vec::new(),
        });
    }

    // 3. One paraphrase pair (MCQ or typing)
    let all_paras = list_paraphrases(conn)?;
    if let Some(first_para) = all_paras.first() {
        let mut opts: Vec<String> = all_paras
            .iter()
            .skip(1)
            .take(3)
            .map(|p| p.passage_paraphrase.clone())
            .collect();
        opts.push(first_para.passage_paraphrase.clone());
        let mut seed = Local::now().timestamp_subsec_nanos() as u64 + 42;
        shuffle_slice(&mut opts, &mut seed);

        items.push(MicroCheckItem {
            item_type: "paraphrase".to_string(),
            ref_id: first_para.id,
            source_label: format!("Paraphrase · {}", first_para.source_type),
            prompt_text: format!(
                "Từ câu hỏi “{}” thường được paraphrase trong bài Reading là gì?",
                first_para.question_term
            ),
            answer_text: first_para.passage_paraphrase.clone(),
            hint: first_para.trap_note.clone(),
            meaning_or_note: first_para.trap_note.clone(),
            options: opts,
        });
    }

    Ok(items)
}

pub fn submit_micro_check(
    conn: &Connection,
    results: Vec<MicroCheckAnswerInput>,
    today: NaiveDate,
) -> Result<MicroCheckSubmitResult, String> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let total_questions = results.len() as i64;
    let mut correct_count = 0_i64;

    for r in &results {
        if r.correct && !r.gave_up {
            correct_count += 1;
        } else if r.item_type == "vocab" {
            let _ = conn.execute(
                "UPDATE vocab_cards
                 SET reps = 0, interval_days = 0, due_date = ?1,
                     lapses = COALESCE(lapses, 0) + 1,
                     ease = MAX(1.3, ease - 0.2)
                 WHERE id = ?2",
                params![today_str, r.ref_id],
            );
        }
    }

    let marked_quest_done = auto_tick_quest_by_hint(conn, &today_str, "micro_check")?;
    adjust_study_day(conn, &today_str, 2, 0)?;

    let accuracy_pct = if total_questions > 0 {
        ((correct_count * 100) + (total_questions / 2)) / total_questions
    } else {
        0
    };

    Ok(MicroCheckSubmitResult {
        total_questions,
        correct_count,
        accuracy_pct,
        marked_quest_done,
    })
}

pub fn get_dictation_batch(
    conn: &Connection,
    count: Option<i64>,
) -> Result<Vec<DictationSentence>, String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let level = if target_band >= 7.5 { "8.0" } else { "6.5" };
    let limit = count.unwrap_or(5).clamp(1, 24);

    let mut stmt = conn
        .prepare(
            "SELECT id, level, section, trap_focus, sentence, meaning_vi, practiced_count, last_accuracy, COALESCE(source, 'seed')
             FROM dictation_sentences
             WHERE level = ?1
             ORDER BY practiced_count ASC, (CASE WHEN COALESCE(source, 'seed') = 'ai_pack' THEN 0 ELSE 1 END) ASC, id ASC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![level, limit], |r| {
            Ok(DictationSentence {
                id: r.get(0)?,
                level: r.get(1)?,
                section: r.get(2)?,
                trap_focus: r.get(3)?,
                sentence: r.get(4)?,
                meaning_vi: r.get(5)?,
                practiced_count: r.get(6)?,
                last_accuracy: r.get(7)?,
                source: r.get::<_, Option<String>>(8)?.unwrap_or_else(|| "seed".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn submit_dictation_session(
    conn: &Connection,
    results: Vec<DictationResultInput>,
    today: NaiveDate,
) -> Result<DictationSubmitResult, String> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let total_sentences = results.len() as i64;
    let mut sum_acc = 0_i64;
    let mut all_wrong: Vec<String> = Vec::new();

    for r in &results {
        let acc = r.accuracy_pct.clamp(0, 100);
        sum_acc += acc;
        conn.execute(
            "UPDATE dictation_sentences
             SET practiced_count = practiced_count + 1, last_accuracy = ?1
             WHERE id = ?2",
            params![acc, r.sentence_id],
        )
        .map_err(|e| e.to_string())?;
        for w in &r.wrong_words {
            if !w.trim().is_empty() && !all_wrong.contains(w) {
                all_wrong.push(w.clone());
            }
        }
    }

    let avg_accuracy = if total_sentences > 0 {
        (sum_acc + total_sentences / 2) / total_sentences
    } else {
        0
    };
    let wrong_json = serde_json::to_string(&all_wrong).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO dictation_sessions (created_at, total_sentences, avg_accuracy, wrong_words_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![today_str, total_sentences, avg_accuracy, wrong_json],
    )
    .map_err(|e| e.to_string())?;

    let marked_quest_done = auto_tick_quest_by_hint(conn, &today_str, "dictation")?;
    adjust_study_day(conn, &today_str, 6, 0)?;

    Ok(DictationSubmitResult {
        total_sentences,
        avg_accuracy,
        marked_quest_done,
    })
}

pub fn list_paraphrases(conn: &Connection) -> Result<Vec<ParaphrasePair>, String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let level = if target_band >= 7.5 { "8.0" } else { "6.5" };
    let mut stmt = conn
        .prepare(
            "SELECT id, question_term, passage_paraphrase, source_type, trap_note, created_at,
                    COALESCE(level, '6.5'), COALESCE(source, 'seed')
             FROM paraphrase_pairs
             WHERE COALESCE(level, '6.5') = ?1 OR COALESCE(source, 'seed') != 'seed'
             ORDER BY (CASE WHEN COALESCE(source, 'seed') = 'ai_pack' THEN 0 ELSE 1 END) ASC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![level], |r| {
            Ok(ParaphrasePair {
                id: r.get(0)?,
                question_term: r.get(1)?,
                passage_paraphrase: r.get(2)?,
                source_type: r.get(3)?,
                trap_note: r.get(4)?,
                created_at: r.get(5)?,
                level: r.get::<_, Option<String>>(6)?.unwrap_or_else(|| "6.5".to_string()),
                source: r.get::<_, Option<String>>(7)?.unwrap_or_else(|| "seed".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn add_paraphrase(
    conn: &Connection,
    question_term: &str,
    passage_paraphrase: &str,
    source_type: &str,
    trap_note: &str,
) -> Result<ParaphrasePair, String> {
    let qt = question_term.trim();
    let pp = passage_paraphrase.trim();
    if qt.is_empty() || pp.is_empty() {
        return Err("Vui lòng nhập Từ câu hỏi và Paraphrase trong bài đọc.".to_string());
    }
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let level = if target_band >= 7.5 { "8.0" } else { "6.5" };
    let now = Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at, level, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'user')",
        params![qt, pp, source_type.trim(), trap_note.trim(), now, level],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, question_term, passage_paraphrase, source_type, trap_note, created_at,
                COALESCE(level, '6.5'), COALESCE(source, 'seed')
         FROM paraphrase_pairs WHERE id = ?1",
        params![id],
        |r| {
            Ok(ParaphrasePair {
                id: r.get(0)?,
                question_term: r.get(1)?,
                passage_paraphrase: r.get(2)?,
                source_type: r.get(3)?,
                trap_note: r.get(4)?,
                created_at: r.get(5)?,
                level: r.get::<_, Option<String>>(6)?.unwrap_or_else(|| "6.5".to_string()),
                source: r.get::<_, Option<String>>(7)?.unwrap_or_else(|| "user".to_string()),
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn delete_paraphrase(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM paraphrase_pairs WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn current_roadmap_week(conn: &Connection) -> i64 {
    let undone_count: Option<i64> = conn
        .query_row(
            "SELECT COUNT(*) FROM roadmap_items WHERE track = 'ielts' AND done = 1",
            [],
            |r| r.get(0),
        )
        .ok();
    match undone_count {
        Some(done_n) => (done_n + 1).clamp(1, 24),
        None => 1,
    }
}

pub fn get_speaking_cue(
    conn: &Connection,
    week_num: Option<i64>,
    random_next: Option<bool>,
    exclude_id: Option<i64>,
) -> Result<SpeakingCue, String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let level = if target_band >= 7.5 { "8.0" } else { "6.5" };
    let wk = week_num.unwrap_or_else(|| current_roadmap_week(conn)).clamp(1, 24);
    let ex_id = exclude_id.unwrap_or(-1);

    let sql = if random_next.unwrap_or(false) {
        "SELECT id, week_num, level, topic, cue_title, cue_bullets_json, part3_questions_json, target_phrases_json, used_count
         FROM speaking_bank
         WHERE level = ?1 AND id != ?2
         ORDER BY used_count ASC, ABS(week_num - ?3) ASC, id ASC
         LIMIT 1"
    } else {
        "SELECT id, week_num, level, topic, cue_title, cue_bullets_json, part3_questions_json, target_phrases_json, used_count
         FROM speaking_bank
         WHERE level = ?1 AND id != ?2
         ORDER BY (CASE WHEN week_num = ?3 THEN 0 ELSE 1 END) ASC, used_count ASC, ABS(week_num - ?3) ASC, id ASC
         LIMIT 1"
    };

    conn.query_row(sql, params![level, ex_id, wk], |r| {
        let bullets_json: String = r.get(5)?;
        let p3_json: String = r.get(6)?;
        let phrases_json: String = r.get(7)?;
        Ok(SpeakingCue {
            id: r.get(0)?,
            week_num: r.get(1)?,
            level: r.get(2)?,
            topic: r.get(3)?,
            cue_title: r.get(4)?,
            cue_bullets: serde_json::from_str(&bullets_json).unwrap_or_default(),
            part3_questions: serde_json::from_str(&p3_json).unwrap_or_default(),
            target_phrases: serde_json::from_str(&phrases_json).unwrap_or_default(),
            used_count: r.get(8)?,
        })
    })
    .map_err(|e| format!("Không tìm thấy Cue Card phù hợp: {e}"))
}

#[allow(clippy::too_many_arguments)]
pub fn save_speaking_session(
    conn: &Connection,
    cue_id: Option<i64>,
    cue_title: &str,
    track: &str,
    mode: &str,
    wpm_start: i64,
    wpm_end: i64,
    fillers_start: i64,
    fillers_end: i64,
    fc_band: f64,
    lr_band: f64,
    gra_band: f64,
    p_band: f64,
    overall_band: f64,
    progress_note: &str,
    transcript: &str,
    feedback_json: &str,
    today: NaiveDate,
) -> Result<(SpeakingSession, bool), String> {
    let created_at = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let today_str = today.format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO speaking_sessions (
            created_at, cue_id, cue_title, track, mode,
            wpm_start, wpm_end, fillers_start, fillers_end,
            fc_band, lr_band, gra_band, p_band, overall_band,
            progress_note, transcript, feedback_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            created_at,
            cue_id,
            cue_title,
            track,
            mode,
            wpm_start,
            wpm_end,
            fillers_start,
            fillers_end,
            fc_band,
            lr_band,
            gra_band,
            p_band,
            overall_band,
            progress_note,
            transcript,
            feedback_json
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();

    if let Some(cid) = cue_id {
        let _ = conn.execute(
            "UPDATE speaking_bank SET used_count = used_count + 1 WHERE id = ?1",
            params![cid],
        );
    }

    let marked_quest_done = auto_tick_quest_by_hint(conn, &today_str, "speaking")?;
    adjust_study_day(conn, &today_str, 10, 0)?;

    let session = SpeakingSession {
        id,
        created_at,
        cue_id,
        cue_title: cue_title.to_string(),
        track: track.to_string(),
        mode: mode.to_string(),
        wpm_start,
        wpm_end,
        fillers_start,
        fillers_end,
        fc_band,
        lr_band,
        gra_band,
        p_band,
        overall_band,
        progress_note: progress_note.to_string(),
        transcript: transcript.to_string(),
        feedback_json: feedback_json.to_string(),
    };
    Ok((session, marked_quest_done))
}

pub fn list_speaking_sessions(conn: &Connection) -> Result<Vec<SpeakingSession>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, cue_id, cue_title, track, mode,
                    wpm_start, wpm_end, fillers_start, fillers_end,
                    fc_band, lr_band, gra_band, p_band, overall_band,
                    progress_note, transcript, feedback_json
             FROM speaking_sessions
             ORDER BY id DESC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SpeakingSession {
                id: r.get(0)?,
                created_at: r.get(1)?,
                cue_id: r.get(2)?,
                cue_title: r.get(3)?,
                track: r.get(4)?,
                mode: r.get(5)?,
                wpm_start: r.get(6)?,
                wpm_end: r.get(7)?,
                fillers_start: r.get(8)?,
                fillers_end: r.get(9)?,
                fc_band: r.get(10)?,
                lr_band: r.get(11)?,
                gra_band: r.get(12)?,
                p_band: r.get(13)?,
                overall_band: r.get(14)?,
                progress_note: r.get(15)?,
                transcript: r.get(16)?,
                feedback_json: r.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn get_top_errors(conn: &Connection, limit: i64) -> Result<Vec<ErrorLog>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category, wrong_text, right_text, count, created_at,
                    COALESCE(drill_mode, 'GRAMMAR_MCQ'), COALESCE(status, 'active'),
                    COALESCE(streak_distinct, 0), COALESCE(last_drill_id, 0),
                    COALESCE(last_context_tag, ''),
                    (SELECT COUNT(*) FROM error_drills d WHERE d.error_log_id = error_logs.id)
             FROM error_logs
             ORDER BY (CASE WHEN COALESCE(status, 'active') = 'active' THEN 0 ELSE 1 END) ASC,
                      count DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(ErrorLog {
                id: r.get(0)?,
                category: r.get(1)?,
                wrong_text: r.get(2)?,
                right_text: r.get(3)?,
                count: r.get(4)?,
                created_at: r.get(5)?,
                drill_mode: r.get(6)?,
                status: r.get(7)?,
                streak_distinct: r.get(8)?,
                last_drill_id: r.get(9)?,
                last_context_tag: r.get(10)?,
                drill_count: r.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn upsert_error_log(
    conn: &Connection,
    category: &str,
    wrong_text: &str,
    right_text: &str,
) -> Result<(), String> {
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM error_logs WHERE lower(wrong_text) = lower(?1) AND lower(right_text) = lower(?2)",
            params![wrong_text.trim(), right_text.trim()],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(id) = existing_id {
        // Relapse trigger: repeating an error reactivates it and resets streak_distinct
        conn.execute(
            "UPDATE error_logs SET count = count + 1, status = 'active', streak_distinct = 0 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let now = Local::now().format("%Y-%m-%d").to_string();
        let cat_low = category.to_lowercase();
        let drill_mode = if cat_low.contains("t/f/ng") || cat_low.contains("paraphrase") || cat_low.contains("part 3") || cat_low.contains("concession") {
            "MINI_CASE"
        } else if cat_low.contains("chính tả") || cat_low.contains("nhầm từ") || cat_low.contains("effect/affect") {
            "TYPING"
        } else {
            "GRAMMAR_MCQ"
        };
        conn.execute(
            "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at, drill_mode, status, streak_distinct, source)
             VALUES (?1, ?2, ?3, 1, ?4, ?5, 'active', 0, 'user')",
            params![category.trim(), wrong_text.trim(), right_text.trim(), now, drill_mode],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn trigger_error_relapse_from_grade(conn: &Connection, grade: &GradeResult) -> Result<i64, String> {
    let mut relapsed = 0_i64;
    let mut stmt = conn
        .prepare("SELECT id, category, wrong_text FROM error_logs WHERE COALESCE(status, 'active') = 'mastered'")
        .map_err(|e| e.to_string())?;
    let mastered: Vec<(i64, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, cat, wrong) in mastered {
        let cat_low = cat.to_lowercase();
        let wrong_low = wrong.to_lowercase();
        let cat_tail = cat_low.split('·').last().unwrap_or("").trim().to_string();

        let matched_rep = grade
            .repeated_top_errors
            .iter()
            .any(|msg| msg.to_lowercase().contains(&cat_low));
        let matched_corr = grade.corrections.iter().any(|c| {
            let cw = c.wrong.to_lowercase();
            let cc = c.category.to_lowercase();
            let cr = c.reason.to_lowercase();
            (!wrong_low.is_empty() && cw.contains(&wrong_low))
                || (!cat_tail.is_empty() && (cc.contains(&cat_tail) || cr.contains(&cat_tail)))
        });

        if matched_rep || matched_corr {
            conn.execute(
                "UPDATE error_logs SET status = 'active', streak_distinct = 0, count = count + 1 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
            relapsed += 1;
        }
    }
    Ok(relapsed)
}

pub fn list_writing_prompts(conn: &Connection, level: &str) -> Result<Vec<WritingPrompt>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, task_type, topic, title, prompt_text, outline_hint, target_vocab_json, source, practiced_count, COALESCE(chart_spec_json, '')
             FROM writing_prompts
             WHERE level = ?1
             ORDER BY (CASE WHEN source = 'ai_pack' THEN 0 ELSE 1 END) ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![level], |r| {
            let id: i64 = r.get(0)?;
            let task_type: String = r.get(1)?;
            let target_words = match task_type.as_str() {
                "task1" => 150,
                "task2" => 250,
                _ => 90,
            };
            let tv_json: String = r.get(6)?;
            let target_vocab: Vec<String> = serde_json::from_str(&tv_json).unwrap_or_default();
            Ok(WritingPrompt {
                id: format!("wp-{id}"),
                mode: task_type,
                topic: r.get(2)?,
                title: r.get(3)?,
                prompt: r.get(4)?,
                outline_hint: r.get(5)?,
                target_words,
                target_vocab,
                source: r.get(7)?,
                practiced_count: r.get(8)?,
                chart_spec_json: r.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn mark_writing_practiced(
    conn: &Connection,
    prompt_id: Option<&str>,
    today: NaiveDate,
) -> Result<bool, String> {
    if let Some(pid) = prompt_id {
        if let Some(raw_num) = pid.strip_prefix("wp-") {
            if let Ok(num) = raw_num.parse::<i64>() {
                let _ = conn.execute(
                    "UPDATE writing_prompts SET practiced_count = practiced_count + 1 WHERE id = ?1",
                    params![num],
                );
            }
        }
    }
    let today_str = today.format("%Y-%m-%d").to_string();
    let ticked = auto_tick_quest_by_hint(conn, &today_str, "writing")?;
    adjust_study_day(conn, &today_str, 10, 0)?;
    Ok(ticked)
}

pub fn get_pack_context_for_ai(conn: &Connection) -> Result<(String, Vec<String>, Vec<ErrorLog>), String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let level = if target_band >= 7.5 { "8.0" } else { "6.5" }.to_string();

    let mut w_stmt = conn
        .prepare("SELECT word FROM vocab_cards ORDER BY id DESC LIMIT 60")
        .map_err(|e| e.to_string())?;
    let exclude_words: Vec<String> = w_stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let top_errors = get_top_errors(conn, 3)?;
    Ok((level, exclude_words, top_errors))
}

pub fn apply_topic_pack_to_db(
    conn: &Connection,
    topic: &str,
    level: &str,
    pack: &TopicPackPayload,
    today: &str,
    endpoint: &str,
) -> Result<RefillPackResult, String> {
    let clean_topic = topic.trim();
    let mut vocab_added = 0_i64;
    let mut dictations_added = 0_i64;
    let mut prompts_added = 0_i64;
    let mut paraphrases_added = 0_i64;
    let mut drills_added = 0_i64;

    let mut active_error_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM error_logs
                 ORDER BY (CASE WHEN COALESCE(status, 'active') = 'active' THEN 0 ELSE 1 END) ASC,
                          count DESC, id DESC
                 LIMIT 3",
            )
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        ids
    };
    if active_error_ids.is_empty() {
        active_error_ids.push(1);
    }

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // 1. Vocab (10 collocations)
    for v in &pack.vocab {
        let w = v.word.trim();
        let m = v.meaning.trim();
        let ex = v.example.trim();
        if w.is_empty() || m.is_empty() || ex.is_empty() {
            continue;
        }
        let changed = tx
            .execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, source)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 'ai_pack'
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE lower(word) = lower(?2))",
                params![clean_topic, w, m, ex, today],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            vocab_added += 1;
        }
    }

    // 2. Dictation (6..=30 words, non-empty meaning)
    for d in &pack.dictations {
        let sent = d.sentence.trim();
        let meaning = d.meaning_vi.trim();
        let wc = sent.split_whitespace().count();
        if wc < 6 || wc > 30 || meaning.is_empty() {
            continue;
        }
        let section = if d.section.trim().is_empty() {
            "Listening Part 3"
        } else {
            d.section.trim()
        };
        let trap = if d.trap_focus.trim().is_empty() {
            clean_topic
        } else {
            d.trap_focus.trim()
        };
        let changed = tx
            .execute(
                "INSERT INTO dictation_sentences (level, section, trap_focus, sentence, meaning_vi, source)
                 SELECT ?1, ?2, ?3, ?4, ?5, 'ai_pack'
                 WHERE NOT EXISTS (SELECT 1 FROM dictation_sentences WHERE lower(sentence) = lower(?4))",
                params![level, section, trap, sent, meaning],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            dictations_added += 1;
        }
    }

    // 3. Writing Prompts
    for wp in &pack.writing_prompts {
        let title = wp.title.trim();
        let prompt_text = wp.prompt_text.trim();
        if title.is_empty() || prompt_text.is_empty() {
            continue;
        }
        let task_type = match wp.task_type.trim().to_lowercase().as_str() {
            "task1" | "task_1" => "task1",
            "paragraph_90" | "body" => "paragraph_90",
            _ => "task2",
        };
        let tv_json = serde_json::to_string(&wp.target_vocab).unwrap_or_else(|_| "[]".to_string());
        let chart_spec = if task_type == "task1" {
            wp.chart_spec_json.trim()
        } else {
            ""
        };
        let changed = tx
            .execute(
                "INSERT INTO writing_prompts (level, task_type, topic, title, prompt_text, outline_hint, target_vocab_json, source, practiced_count, chart_spec_json)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ai_pack', 0, ?8
                 WHERE NOT EXISTS (SELECT 1 FROM writing_prompts WHERE level = ?1 AND lower(title) = lower(?4))",
                params![level, task_type, clean_topic, title, prompt_text, wp.outline_hint.trim(), tv_json, chart_spec],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            prompts_added += 1;
        }
    }

    // 4. Paraphrases
    for p in &pack.paraphrases {
        let qt = p.question_term.trim();
        let pp = p.passage_paraphrase.trim();
        if qt.is_empty() || pp.is_empty() || qt.eq_ignore_ascii_case(pp) {
            continue;
        }
        let src_type = if p.source_type.trim().is_empty() {
            format!("Reading · {clean_topic}")
        } else {
            p.source_type.trim().to_string()
        };
        let changed = tx
            .execute(
                "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at, level, source)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, 'ai_pack'
                 WHERE NOT EXISTS (SELECT 1 FROM paraphrase_pairs WHERE lower(question_term) = lower(?1) AND lower(passage_paraphrase) = lower(?2))",
                params![qt, pp, src_type, p.trap_note.trim(), today, level],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            paraphrases_added += 1;
        }
    }

    // 5. Error Drills (Partial validation: TYPING or >=3 options containing answer)
    for dr in &pack.error_drills {
        let stem = dr.stem.trim();
        let answer = dr.answer.trim();
        if stem.is_empty() || answer.is_empty() {
            continue;
        }
        let mode = match dr.drill_mode.trim() {
            "TYPING" | "MINI_CASE" => dr.drill_mode.trim(),
            _ => "GRAMMAR_MCQ",
        };
        let mut opts = dr.options.clone();
        if mode != "TYPING" {
            if opts.len() < 3 || !opts.iter().any(|o| o.trim() == answer) {
                continue;
            }
        } else {
            opts.clear();
        }
        let eid = active_error_ids
            .get(dr.error_index)
            .copied()
            .unwrap_or(active_error_ids[0]);
        let ctx = if dr.context_tag.trim().is_empty() {
            clean_topic.to_lowercase()
        } else {
            dr.context_tag.trim().to_string()
        };
        let opts_json = serde_json::to_string(&opts).unwrap_or_else(|_| "[]".to_string());
        let changed = tx
            .execute(
                "INSERT INTO error_drills (error_log_id, drill_mode, context_tag, stem, options_json, answer, rationale_vi, source)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ai_pack'
                 WHERE NOT EXISTS (SELECT 1 FROM error_drills WHERE error_log_id = ?1 AND stem = ?4)",
                params![eid, mode, ctx, stem, opts_json, answer, dr.rationale_vi.trim()],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            drills_added += 1;
        }
    }

    tx.execute(
        "UPDATE settings SET last_ai_refill_at = ?1, last_ai_refill_topic = ?2 WHERE id = 1",
        params![today, clean_topic],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RefillPackResult {
        topic: clean_topic.to_string(),
        level: level.to_string(),
        vocab_added,
        dictations_added,
        prompts_added,
        paraphrases_added,
        drills_added,
        endpoint: endpoint.to_string(),
    })
}

pub fn add_practice_score(
    conn: &Connection,
    skill: &str,
    source: &str,
    score: f64,
    note: &str,
) -> Result<(), String> {
    let now = Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO practice_scores (skill, source, score, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![skill, source, score, note, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle_roadmap_item(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE roadmap_items SET done = CASE WHEN done = 0 THEN 1 ELSE 0 END WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_essay(
    conn: &Connection,
    mode: &str,
    prompt: &str,
    content: &str,
    word_count: i64,
    ta: f64,
    cc: f64,
    lr: f64,
    gra: f64,
    overall: f64,
    feedback_json: &str,
) -> Result<i64, String> {
    let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
    conn.execute(
        "INSERT INTO essays (mode, prompt, content, word_count, ta, cc, lr, gra, overall, feedback_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            mode,
            prompt,
            content,
            word_count,
            ta,
            cc,
            lr,
            gra,
            overall,
            feedback_json,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let _ = auto_tick_quest_by_hint(conn, &today, "writing");
    adjust_study_day(conn, &today, 10, 0)?;

    Ok(conn.last_insert_rowid())
}

pub fn check_auto_refill_due(conn: &Connection, today: NaiveDate) -> Result<AutoRefillStatus, String> {
    let raw = get_settings_raw(conn)?;
    let today_str = today.format("%Y-%m-%d").to_string();
    let has_key = !raw.gemini_api_key.trim().is_empty();
    let enabled = raw.auto_refill_enabled;
    let level = if raw.target_band >= 7.5 { "8.0" } else { "6.5" };

    let roadmap_done: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM roadmap_items WHERE track = 'ielts' AND done = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let current_week = (roadmap_done + 1).clamp(1, 24) as usize;
    let mut idx = (current_week - 1) % IELTS_TOPICS.len();
    if IELTS_TOPICS[idx].eq_ignore_ascii_case(raw.last_ai_refill_topic.trim()) {
        idx = (idx + 1) % IELTS_TOPICS.len();
    }
    let recommended_topic = IELTS_TOPICS[idx].to_string();

    if !has_key || !enabled || raw.last_ai_refill_at.trim() == today_str {
        return Ok(AutoRefillStatus {
            enabled,
            should_auto_refill: false,
            trigger_reason: String::new(),
            recommended_topic,
            last_refill_at: raw.last_ai_refill_at,
            last_refill_topic: raw.last_ai_refill_topic,
        });
    }

    let days_since = NaiveDate::parse_from_str(raw.last_ai_refill_at.trim(), "%Y-%m-%d")
        .ok()
        .map(|d| (today - d).num_days());

    // 1. Sunday weekly trigger (>= 6 days since last refill or never refilled)
    if today.weekday() == Weekday::Sun && days_since.map_or(true, |d| d >= 6) {
        return Ok(AutoRefillStatus {
            enabled,
            should_auto_refill: true,
            trigger_reason: "sunday".to_string(),
            recommended_topic,
            last_refill_at: raw.last_ai_refill_at,
            last_refill_topic: raw.last_ai_refill_topic,
        });
    }

    // 2. Study progress trigger (>= 5 days cooldown or >= 3 study_days if never refilled, PLUS milestone met)
    let study_days_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM study_days WHERE minutes > 0", [], |r| r.get(0))
        .unwrap_or(0);
    let learned_vocab: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocab_cards WHERE reps >= 1", [], |r| r.get(0))
        .unwrap_or(0);
    let dictation_unpracticed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dictation_sentences WHERE level = ?1 AND practiced_count = 0",
            params![level],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let time_ready = days_since.map_or_else(|| study_days_count >= 3, |d| d >= 5);
    let progress_ready = learned_vocab >= 10 || dictation_unpracticed < 5 || roadmap_done >= 2;
    let (should_auto_refill, trigger_reason) = if time_ready && progress_ready {
        (true, "progress".to_string())
    } else {
        (false, String::new())
    };

    Ok(AutoRefillStatus {
        enabled,
        should_auto_refill,
        trigger_reason,
        recommended_topic,
        last_refill_at: raw.last_ai_refill_at,
        last_refill_topic: raw.last_ai_refill_topic,
    })
}

pub fn get_dashboard_state(conn: &Connection) -> Result<DashboardState, String> {
    let today_naive = Local::now().date_naive();
    let today = today_naive.format("%Y-%m-%d").to_string();
    ensure_today_quests(conn, &today)?;

    let settings = get_settings_public(conn)?;
    let streak = calculate_streak(conn, today_naive)?;
    let exam_naive = NaiveDate::parse_from_str(&settings.exam_date, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2027, 5, 15).unwrap());
    let days_until_exam = (exam_naive - today_naive).num_days().max(0);

    let mut q_stmt = conn
        .prepare(
            "SELECT id, date, track, title, meta, minutes, done,
                    COALESCE(tier, 'core'), COALESCE(action_hint, '')
             FROM daily_quests WHERE date = ?1 ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;
    let quests: Vec<DailyQuest> = q_stmt
        .query_map(params![today], |r| {
            let title: String = r.get(3)?;
            let raw_hint: String = r.get(8)?;
            let action_hint = if raw_hint.trim().is_empty() {
                infer_quest_action_hint(&title)
            } else {
                raw_hint
            };
            Ok(DailyQuest {
                id: r.get(0)?,
                date: r.get(1)?,
                track: r.get(2)?,
                title,
                meta: r.get(4)?,
                minutes: r.get(5)?,
                done: r.get::<_, i64>(6)? != 0,
                tier: r.get(7)?,
                action_hint,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let heatmap = get_heatmap(conn, today_naive, 70)?;

    let raw_settings = get_settings_raw(conn)?;
    let default_targets = if raw_settings.target_band >= 7.5 {
        default_skill_targets_80()
    } else {
        default_skill_targets()
    };
    let mut skill_bands = Vec::new();
    for skill in ["Listening", "Reading", "Speaking", "Writing"] {
        let target = raw_settings
            .skill_targets
            .get(skill)
            .copied()
            .or_else(|| default_targets.get(skill).copied())
            .unwrap_or(6.5);
        let latest: f64 = conn
            .query_row(
                "SELECT score FROM practice_scores WHERE skill = ?1 ORDER BY id DESC LIMIT 1",
                params![skill],
                |r| r.get(0),
            )
            .unwrap_or(4.5);
        skill_bands.push(SkillBand {
            skill: skill.to_string(),
            current: latest,
            target,
        });
    }

    let due_vocab_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM vocab_cards WHERE due_date <= ?1 AND COALESCE(hidden, 0) = 0",
            params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total_vocab_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM vocab_cards WHERE COALESCE(hidden, 0) = 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let leech_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM vocab_cards WHERE COALESCE(lapses, 0) >= 3 AND COALESCE(hidden, 0) = 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let quiz_status = get_vocab_quiz_status(conn, today_naive)?;

    let mut r_stmt = conn
        .prepare("SELECT id, track, phase, week_label, title, done FROM roadmap_items ORDER BY id ASC")
        .map_err(|e| e.to_string())?;
    let roadmap: Vec<RoadmapItem> = r_stmt
        .query_map([], |r| {
            Ok(RoadmapItem {
                id: r.get(0)?,
                track: r.get(1)?,
                phase: r.get(2)?,
                week_label: r.get(3)?,
                title: r.get(4)?,
                done: r.get::<_, i64>(5)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let current_week = roadmap
        .iter()
        .position(|r| !r.done)
        .map(|i| (i as i64) + 1)
        .unwrap_or_else(|| roadmap.len().max(1) as i64)
        .clamp(1, 24);

    let error_logs = get_top_errors(conn, 40)?;
    let paraphrase_pairs = list_paraphrases(conn)?;
    let speaking_sessions = list_speaking_sessions(conn)?;

    let mut es_stmt = conn
        .prepare("SELECT id, mode, prompt, content, word_count, ta, cc, lr, gra, overall, feedback_json, created_at FROM essays ORDER BY id DESC LIMIT 10")
        .map_err(|e| e.to_string())?;
    let recent_essays: Vec<Essay> = es_stmt
        .query_map([], |r| {
            Ok(Essay {
                id: r.get(0)?,
                mode: r.get(1)?,
                prompt: r.get(2)?,
                content: r.get(3)?,
                word_count: r.get(4)?,
                ta: r.get(5)?,
                cc: r.get(6)?,
                lr: r.get(7)?,
                gra: r.get(8)?,
                overall: r.get(9)?,
                feedback_json: r.get(10)?,
                created_at: r.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut ps_stmt = conn
        .prepare("SELECT id, skill, source, score, note, created_at FROM practice_scores ORDER BY id DESC LIMIT 20")
        .map_err(|e| e.to_string())?;
    let practice_scores: Vec<PracticeScore> = ps_stmt
        .query_map([], |r| {
            Ok(PracticeScore {
                id: r.get(0)?,
                skill: r.get(1)?,
                source: r.get(2)?,
                score: r.get(3)?,
                note: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let level = if raw_settings.target_band >= 7.5 { "8.0" } else { "6.5" };
    let prompts = list_writing_prompts(conn, level)?;
    let auto_refill_status = check_auto_refill_due(conn, Local::now().date_naive())?;

    Ok(DashboardState {
        settings,
        streak,
        days_until_exam,
        quests,
        heatmap,
        skill_bands,
        due_vocab_count,
        total_vocab_count,
        leech_count,
        current_week,
        quiz_status,
        roadmap,
        error_logs,
        paraphrase_pairs,
        speaking_sessions,
        recent_essays,
        practice_scores,
        prompts,
        auto_refill_status,
    })
}

fn normalize_week_key(label: &str) -> String {
    let trimmed = label.trim();
    if let Some(idx) = trimmed.find('(') {
        trimmed[..idx].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn set_study_level(conn: &Connection, level: &str) -> Result<DashboardState, String> {
    let is_80 = level.trim() == "8.0";
    let new_target = if is_80 { 8.0 } else { 7.0 };

    let skill_targets = if is_80 {
        default_skill_targets_80()
    } else {
        default_skill_targets()
    };
    let checklist = if is_80 {
        default_writing_checklist_80()
    } else {
        default_writing_checklist()
    };
    let focus_notes = if is_80 {
        default_writing_focus_notes_80()
    } else {
        default_writing_focus_notes()
    };
    let phase_labels = if is_80 {
        default_phase_labels_80()
    } else {
        default_phase_labels()
    };

    let tgt_json = serde_json::to_string(&skill_targets).map_err(|e| e.to_string())?;
    let chk_json = serde_json::to_string(&checklist).map_err(|e| e.to_string())?;
    let foc_json = serde_json::to_string(&focus_notes).map_err(|e| e.to_string())?;
    let ph_json = serde_json::to_string(&phase_labels).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE settings
         SET target_band = ?1,
             skill_targets_json = ?2,
             writing_checklist_json = ?3,
             writing_focus_json = ?4,
             phase_labels_json = ?5,
             daily_quests_template_json = '[]'
         WHERE id = 1",
        params![new_target, tgt_json, chk_json, foc_json, ph_json],
    )
    .map_err(|e| e.to_string())?;

    let user_score_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM practice_scores
             WHERE source NOT IN ('Test đầu vào (02/11/2026)', 'Nền đầu vào Aim 8.0')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total_score_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM practice_scores", [], |r| r.get(0))
        .unwrap_or(0);

    let today_naive = Local::now().date_naive();
    let today = today_naive.format("%Y-%m-%d").to_string();

    if user_score_count == 0 && total_score_count <= 4 {
        conn.execute("DELETE FROM practice_scores", [])
            .map_err(|e| e.to_string())?;
        let seeds = if is_80 {
            [
                ("Listening", "Nền đầu vào Aim 8.0", 7.0, "Mục tiêu 8.5 · Nền 6.5+"),
                ("Reading", "Nền đầu vào Aim 8.0", 7.0, "Mục tiêu 8.5 · Nền 6.5+"),
                ("Speaking", "Nền đầu vào Aim 8.0", 6.5, "Mục tiêu 7.5 · Nền 6.5+"),
                ("Writing", "Nền đầu vào Aim 8.0", 6.5, "Mục tiêu 7.5 · Nền 6.5+"),
            ]
        } else {
            [
                ("Listening", "Test đầu vào (02/11/2026)", 5.5, "Mục tiêu 7.5 · Sàn 7.0"),
                ("Reading", "Test đầu vào (02/11/2026)", 5.0, "Mục tiêu 7.0 · Sàn 6.5"),
                ("Speaking", "Test đầu vào (02/11/2026)", 5.5, "Mục tiêu 6.5 · Sàn 6.0"),
                ("Writing", "Test đầu vào (02/11/2026)", 4.5, "Mục tiêu 6.0 · Sàn 5.5"),
            ]
        };
        for (skill, src, score, note) in seeds {
            conn.execute(
                "INSERT INTO practice_scores (skill, source, score, note, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![skill, src, score, note, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let mut done_map: HashMap<String, bool> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT week_label, done FROM roadmap_items WHERE track = 'ielts'")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            done_map.insert(normalize_week_key(&row.0), row.1);
        }
    }

    conn.execute("DELETE FROM roadmap_items WHERE track = 'ielts'", [])
        .map_err(|e| e.to_string())?;
    let roadmap_seeds = if is_80 { ROADMAP_SEEDS_80 } else { ROADMAP_SEEDS_65 };
    for item in roadmap_seeds {
        let is_done = done_map
            .get(&normalize_week_key(item.week_label))
            .copied()
            .unwrap_or(false);
        conn.execute(
            "INSERT INTO roadmap_items (track, phase, week_label, title, done)
             VALUES ('ielts', ?1, ?2, ?3, ?4)",
            params![item.phase, item.week_label, item.title, if is_done { 1 } else { 0 }],
        )
        .map_err(|e| e.to_string())?;
    }

    let seeds_65 = all_vocab_seeds_65();
    let seeds_80 = all_vocab_seeds_80();
    if is_80 {
        for v in &seeds_65 {
            conn.execute(
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0 AND COALESCE(is_user_card, 0) = 0 AND COALESCE(source, 'seed') = 'seed'",
                params![v.word],
            )
            .map_err(|e| e.to_string())?;
        }
        for v in &seeds_80 {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, source)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 'seed'
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE lower(word) = lower(?2))",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    } else {
        for v in &seeds_80 {
            conn.execute(
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0 AND COALESCE(is_user_card, 0) = 0 AND COALESCE(source, 'seed') = 'seed'",
                params![v.word],
            )
            .map_err(|e| e.to_string())?;
        }
        for v in &seeds_65 {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, source)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 'seed'
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE lower(word) = lower(?2))",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Swap untouched opposite-track seed errors + their seed drills cleanly, keeping all user/practiced errors!
    let old_seed_src = if is_80 { "seed_65" } else { "seed_80" };
    let new_seed_src = if is_80 { "seed_80" } else { "seed_65" };
    conn.execute(
        "DELETE FROM error_drills
         WHERE source = 'seed'
           AND error_log_id IN (
             SELECT id FROM error_logs
             WHERE source = ?1 AND COALESCE(streak_distinct, 0) = 0 AND COALESCE(status, 'active') = 'active'
           )",
        params![old_seed_src],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM error_logs
         WHERE source = ?1 AND COALESCE(streak_distinct, 0) = 0 AND COALESCE(status, 'active') = 'active'",
        params![old_seed_src],
    )
    .map_err(|e| e.to_string())?;

    let err_seeds = if is_80 { ERROR_LOG_SEEDS_80 } else { ERROR_LOG_SEEDS_65 };
    for e in err_seeds {
        conn.execute(
            "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at, drill_mode, status, streak_distinct, source)
             SELECT ?1, ?2, ?3, ?4, ?5, ?6, 'active', 0, ?7
             WHERE NOT EXISTS (SELECT 1 FROM error_logs WHERE lower(wrong_text) = lower(?2) AND lower(right_text) = lower(?3))",
            params![e.category, e.wrong_text, e.right_text, e.count, today, e.drill_mode, new_seed_src],
        )
        .map_err(|e| e.to_string())?;
    }
    ensure_seed_error_drills(conn)?;

    let mut prev_done: Vec<bool> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT done FROM daily_quests WHERE date = ?1 AND track = 'ielts' ORDER BY id ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![today], |r| Ok(r.get::<_, i64>(0)? != 0))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            prev_done.push(row);
        }
    }
    conn.execute(
        "DELETE FROM daily_quests WHERE date = ?1 AND track = 'ielts'",
        params![today],
    )
    .map_err(|e| e.to_string())?;

    let defaults = default_quests_for_weekday(today_naive.weekday(), is_80);
    for (idx, q) in defaults.iter().enumerate() {
        let was_done = prev_done.get(idx).copied().unwrap_or(false);
        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
             VALUES (?1, 'ielts', ?2, ?3, ?4, ?5, ?6, ?7)",
            params![today, q.title, q.meta, q.minutes, if was_done { 1 } else { 0 }, q.tier, q.action_hint],
        )
        .map_err(|e| e.to_string())?;
    }

    get_dashboard_state(conn)
}

#[derive(serde::Serialize)]
pub struct WidgetStatus {
    pub theme: String,
    pub quests_done: i64,
    pub quests_total: i64,
    pub streak: i64,
    pub overall_band: f64,
    pub target_band: f64,
    pub days_left: i64,
    pub today_minutes: i64,
    pub updated_at: String,
}

pub fn import_roadmap_to_db(
    conn: &Connection,
    markdown: &str,
    replace_existing: bool,
    merge_vocab_errors: bool,
) -> Result<(), String> {
    let parsed = parse_roadmap_markdown(markdown)?;
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let cur = get_settings_raw(conn)?;

    let new_exam = parsed.exam_date.unwrap_or(cur.exam_date);
    let new_target = parsed
        .buffer_target
        .or(parsed.overall_target)
        .unwrap_or(cur.target_band);
    let new_rem = parsed.reminder_time.unwrap_or(cur.reminder_time);

    let new_checklist = if !parsed.writing_checklist.is_empty() {
        parsed.writing_checklist
    } else if replace_existing {
        default_writing_checklist()
    } else {
        cur.writing_checklist
    };

    let new_focus = if !parsed.writing_focus_notes.is_empty() {
        parsed.writing_focus_notes
    } else if replace_existing {
        default_writing_focus_notes()
    } else {
        cur.writing_focus_notes
    };

    let mut new_targets = cur.skill_targets.clone();
    for sb in &parsed.skill_bands {
        new_targets.insert(sb.skill.clone(), sb.target);
    }

    let new_phase_labels = if !parsed.phase_labels.is_empty() {
        parsed.phase_labels.clone()
    } else if replace_existing {
        default_phase_labels()
    } else {
        cur.phase_labels
    };

    let chk_json = serde_json::to_string(&new_checklist).map_err(|e| e.to_string())?;
    let foc_json = serde_json::to_string(&new_focus).map_err(|e| e.to_string())?;
    let tgt_json = serde_json::to_string(&new_targets).map_err(|e| e.to_string())?;
    let ph_json = serde_json::to_string(&new_phase_labels).map_err(|e| e.to_string())?;
    let quests_tmpl_json = if !parsed.daily_quests.is_empty() {
        Some(serde_json::to_string(&parsed.daily_quests).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Snapshot existing roadmap done flags before replace so completed weeks are 100% preserved
    let mut done_map: HashMap<String, bool> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT week_label, done FROM roadmap_items WHERE track = 'ielts'")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            done_map.insert(normalize_week_key(&row.0), row.1);
        }
    }

    conn.execute("BEGIN IMMEDIATE", []).map_err(|e| e.to_string())?;
    let res: Result<(), String> = (|| {
        conn.execute(
            "UPDATE settings
             SET exam_date = ?1, reminder_time = ?2, target_band = ?3,
                 writing_checklist_json = ?4, writing_focus_json = ?5,
                 skill_targets_json = ?6, phase_labels_json = ?7
             WHERE id = 1",
            params![new_exam, new_rem, new_target, chk_json, foc_json, tgt_json, ph_json],
        )
        .map_err(|e| e.to_string())?;

        if let Some(qt) = quests_tmpl_json {
            conn.execute(
                "UPDATE settings SET daily_quests_template_json = ?1 WHERE id = 1",
                params![qt],
            )
            .map_err(|e| e.to_string())?;
        }

        if !parsed.skill_bands.is_empty() {
            if replace_existing {
                conn.execute("DELETE FROM practice_scores", [])
                    .map_err(|e| e.to_string())?;
            }
            for sb in &parsed.skill_bands {
                conn.execute(
                    "INSERT INTO practice_scores (skill, source, score, note, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        sb.skill,
                        "Nạp từ lộ trình .md",
                        sb.current,
                        format!("Mục tiêu {:.1}", sb.target),
                        today
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        if !parsed.daily_quests.is_empty() {
            if replace_existing {
                conn.execute(
                    "DELETE FROM daily_quests WHERE date = ?1 AND track = 'ielts'",
                    params![today],
                )
                .map_err(|e| e.to_string())?;
            }
            for q in &parsed.daily_quests {
                let tier = if q.tier == "bonus" { "bonus" } else { "core" };
                let hint = if q.action_hint.trim().is_empty() {
                    infer_quest_action_hint(&q.title)
                } else {
                    q.action_hint.clone()
                };
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0, ?5, ?6)",
                    params![today, q.title, q.meta, q.minutes, tier, hint],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        if !parsed.roadmap_weeks.is_empty() {
            if replace_existing {
                conn.execute("DELETE FROM roadmap_items WHERE track = 'ielts'", [])
                    .map_err(|e| e.to_string())?;
            }
            for w in &parsed.roadmap_weeks {
                let preserved = done_map
                    .get(&normalize_week_key(&w.week_label))
                    .copied()
                    .unwrap_or(false);
                let is_done = preserved || w.checked;
                conn.execute(
                    "INSERT INTO roadmap_items (track, phase, week_label, title, done)
                     VALUES ('ielts', ?1, ?2, ?3, ?4)",
                    params![w.phase, w.week_label, w.title, if is_done { 1 } else { 0 }],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        if !parsed.error_logs.is_empty() {
            if replace_existing && !merge_vocab_errors {
                conn.execute("DELETE FROM error_logs", [])
                    .map_err(|e| e.to_string())?;
            }
            for err in &parsed.error_logs {
                upsert_error_log(conn, &err.category, &err.wrong_text, &err.right_text)?;
            }
        }

        if !parsed.vocab_cards.is_empty() {
            if replace_existing && !merge_vocab_errors {
                conn.execute("DELETE FROM vocab_cards WHERE track = 'ielts'", [])
                    .map_err(|e| e.to_string())?;
            }
            for v in &parsed.vocab_cards {
                let exists: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM vocab_cards WHERE lower(word) = lower(?1)",
                        params![v.word],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if exists == 0 {
                    conn.execute(
                        "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, is_user_card)
                         VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 1)",
                        params![v.topic, v.word, v.meaning, v.example, today],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }

        if !parsed.paraphrase_pairs.is_empty() {
            for p in &parsed.paraphrase_pairs {
                conn.execute(
                    "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at, level, source)
                     SELECT ?1, ?2, ?3, ?4, ?5, '6.5', 'user'
                     WHERE NOT EXISTS (SELECT 1 FROM paraphrase_pairs WHERE lower(question_term) = lower(?1) AND lower(passage_paraphrase) = lower(?2))",
                    params![p.question_term, p.passage_paraphrase, p.source_type, p.trap_note, today],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    })();

    match res {
        Ok(()) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn export_roadmap_markdown(conn: &Connection) -> Result<String, String> {
    let dash = get_dashboard_state(conn)?;
    let mut lines: Vec<String> = Vec::new();
    lines.push("---".to_string());
    lines.push(format!("exam_date: {}", dash.settings.exam_date));

    let avg_cur = if dash.skill_bands.is_empty() {
        5.0
    } else {
        let sum: f64 = dash.skill_bands.iter().map(|b| b.current).sum();
        ((sum / dash.skill_bands.len() as f64) * 2.0).round() / 2.0
    };
    let avg_tgt = if dash.skill_bands.is_empty() {
        6.5
    } else {
        let sum: f64 = dash.skill_bands.iter().map(|b| b.target).sum();
        ((sum / dash.skill_bands.len() as f64) * 2.0).round() / 2.0
    };
    let core_mins: i64 = dash
        .quests
        .iter()
        .filter(|q| q.tier != "bonus")
        .map(|q| q.minutes)
        .sum::<i64>()
        .max(20);

    lines.push(format!("overall_current: {:.1}", avg_cur));
    lines.push(format!("overall_target: {:.1}", avg_tgt));
    lines.push(format!("buffer_target: {:.1}", dash.settings.target_band));
    for sb in &dash.skill_bands {
        lines.push(format!(
            "{}: {:.1} -> {:.1}",
            sb.skill.to_lowercase(),
            sb.current,
            sb.target
        ));
    }
    lines.push(format!("daily_minutes: {}", core_mins));
    lines.push(format!("reminder_time: {}", dash.settings.reminder_time));
    lines.push("---".to_string());
    lines.push(String::new());

    lines.push("## Nhiệm vụ Core".to_string());
    for q in &dash.quests {
        let tag = if q.tier == "bonus" { "bonus" } else { "core" };
        lines.push(format!("- [{}] [{}p] {} | {}", tag, q.minutes, q.title, q.meta));
    }
    lines.push(String::new());

    lines.push("## Checklist Writing cá nhân".to_string());
    for c in &dash.settings.writing_checklist {
        lines.push(format!("- {c}"));
    }
    lines.push(String::new());

    lines.push("## Điểm yếu Writing cần AI soi kỹ".to_string());
    for f in &dash.settings.writing_focus_notes {
        lines.push(format!("- {f}"));
    }
    lines.push(String::new());

    lines.push("## Sổ lỗi ban đầu".to_string());
    for e in dash.error_logs.iter().take(15) {
        lines.push(format!("- [{}] {} -> {}", e.category, e.wrong_text, e.right_text));
    }
    lines.push(String::new());

    lines.push("## Bảng Paraphrase".to_string());
    for p in dash.paraphrase_pairs.iter().take(15) {
        lines.push(format!(
            "- {} -> {} | {} | {}",
            p.question_term, p.passage_paraphrase, p.source_type, p.trap_note
        ));
    }
    lines.push(String::new());

    let mut v_stmt = conn
        .prepare("SELECT topic, word, meaning, example FROM vocab_cards WHERE COALESCE(hidden, 0) = 0 ORDER BY id ASC LIMIT 20")
        .map_err(|e| e.to_string())?;
    let v_rows = v_stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    lines.push("## Sentence Mining".to_string());
    for vr in v_rows {
        let (topic, word, meaning, example) = vr.map_err(|e| e.to_string())?;
        lines.push(format!("- [{topic}] {word} | {meaning} | {example}"));
    }
    lines.push(String::new());

    lines.push("## Lộ trình theo tuần".to_string());
    let mut last_phase: i64 = -1;
    for r in &dash.roadmap {
        if r.phase != last_phase {
            let label = dash
                .settings
                .phase_labels
                .get(&r.phase)
                .cloned()
                .unwrap_or_else(|| format!("GĐ{}: Giai đoạn {}", r.phase, r.phase));
            lines.push(format!("### {label}"));
            last_phase = r.phase;
        }
        let mark = if r.done { "x" } else { " " };
        lines.push(format!("- [{}] {}: {}", mark, r.week_label, r.title));
    }
    lines.push(String::new());

    Ok(lines.join("\n"))
}

pub fn build_widget_status(conn: &Connection) -> Result<WidgetStatus, String> {
    let today_naive = Local::now().date_naive();
    let today = today_naive.format("%Y-%m-%d").to_string();
    ensure_today_quests(conn, &today)?;

    let settings = get_settings_raw(conn)?;
    let streak = calculate_streak(conn, today_naive)?;
    let exam_naive = NaiveDate::parse_from_str(&settings.exam_date, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2027, 5, 15).unwrap());
    let days_left = (exam_naive - today_naive).num_days().max(0);

    let (quests_done, quests_total): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN done = 1 THEN 1 ELSE 0 END), 0), COUNT(*)
             FROM daily_quests
             WHERE date = ?1 AND track = 'ielts' AND COALESCE(tier, 'core') = 'core'",
            params![today],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 3));

    let today_minutes: i64 = conn
        .query_row(
            "SELECT COALESCE(minutes, 0) FROM study_days WHERE date = ?1",
            params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut sum_band = 0.0_f64;
    for skill in ["Listening", "Reading", "Speaking", "Writing"] {
        let s: f64 = conn
            .query_row(
                "SELECT score FROM practice_scores WHERE skill = ?1 ORDER BY id DESC LIMIT 1",
                params![skill],
                |r| r.get(0),
            )
            .unwrap_or(4.5);
        sum_band += s;
    }
    let overall_band = ((sum_band / 4.0) * 2.0).round() / 2.0;

    Ok(WidgetStatus {
        theme: settings.theme,
        quests_done,
        quests_total: quests_total.max(1),
        streak,
        overall_band,
        target_band: settings.target_band,
        days_left,
        today_minutes,
        updated_at: Local::now().to_rfc3339(),
    })
}

pub fn checkpoint_and_copy_backup(
    conn: &Connection,
    src_path: &Path,
    dest_path: &Path,
) -> Result<(), String> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("Không thể đồng bộ WAL trước khi sao lưu: {e}"))?;
    fs::copy(src_path, dest_path)
        .map_err(|e| format!("Không thể ghi file sao lưu: {e}"))?;
    Ok(())
}

pub fn validate_backup_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("File sao lưu không tồn tại.".to_string());
    }
    let test_conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("Không thể mở file sao lưu (có thể không phải file SQLite): {e}"))?;

    let integrity: String = test_conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| format!("File sao lưu bị lỗi hoặc không phải cơ sở dữ liệu SQLite hợp lệ: {e}"))?;
    if !integrity.eq_ignore_ascii_case("ok") {
        return Err(format!("Kiểm tra toàn vẹn thất bại: {integrity}"));
    }

    let table_count: i64 = test_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN ('settings', 'vocab_cards', 'roadmap_items')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Không đọc được cấu trúc bảng trong file: {e}"))?;
    if table_count < 3 {
        return Err("File .db này không phải bản sao lưu của XD4T IELTS (thiếu bảng dữ liệu cốt lõi).".to_string());
    }
    Ok(())
}

pub fn write_widget_status(conn: &Connection) -> Result<(), String> {
    let dir = data_dir()?;
    let status = build_widget_status(conn)?;
    let json = serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?;
    let tmp_path = dir.join("widget-status.json.tmp");
    let final_path = dir.join("widget-status.json");
    fs::write(&tmp_path, json).map_err(|e| format!("Failed writing widget-status.json.tmp: {e}"))?;
    fs::rename(&tmp_path, &final_path).map_err(|e| format!("Failed renaming widget-status.json: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_seed_imports_60_vocab_24_roadmap_48_dictation_48_speaking_12_paraphrase_24_prompts() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.total_vocab_count, 60);
        assert_eq!(dash.roadmap.len(), 24);
        assert_eq!(dash.paraphrase_pairs.len(), 12);
        assert_eq!(dash.heatmap.len(), 70);
        assert_eq!(dash.prompts.len(), 12);

        let dict_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM dictation_sentences", [], |r| r.get(0))
            .unwrap();
        assert_eq!(dict_count, 48);

        let spk_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM speaking_bank", [], |r| r.get(0))
            .unwrap();
        assert_eq!(spk_count, 48);

        let wp_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM writing_prompts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(wp_count, 24);

        let drill_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM error_drills", [], |r| r.get(0))
            .unwrap();
        assert_eq!(drill_count, 30);
    }

    #[test]
    fn migration_v150_to_v160_preserves_all_existing_user_data() {
        let conn = Connection::open_in_memory().unwrap();
        // Simulate v1.5.0 schema before v1.6.0 columns/tables existed
        conn.execute_batch(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                theme TEXT NOT NULL DEFAULT 'pastel-male',
                gemini_api_key TEXT NOT NULL DEFAULT 'AIzaOldKey123',
                gemini_model TEXT NOT NULL DEFAULT 'gemini-3-flash-preview',
                exam_date TEXT NOT NULL DEFAULT '2027-05-15',
                reminder_time TEXT NOT NULL DEFAULT '21:00',
                autostart INTEGER NOT NULL DEFAULT 0,
                target_band REAL NOT NULL DEFAULT 7.0,
                onboarding_done INTEGER NOT NULL DEFAULT 1
            );
            INSERT INTO settings (id) VALUES (1);
            CREATE TABLE daily_quests (id INTEGER PRIMARY KEY AUTOINCREMENT, date TEXT NOT NULL, track TEXT NOT NULL DEFAULT 'ielts', title TEXT NOT NULL, meta TEXT NOT NULL, minutes INTEGER NOT NULL, done INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE study_days (date TEXT PRIMARY KEY, track TEXT NOT NULL DEFAULT 'ielts', minutes INTEGER NOT NULL DEFAULT 0, quests_done INTEGER NOT NULL DEFAULT 0);
            INSERT INTO study_days (date, minutes, quests_done) VALUES ('2026-11-24', 30, 3);
            CREATE TABLE vocab_cards (id INTEGER PRIMARY KEY AUTOINCREMENT, track TEXT NOT NULL DEFAULT 'ielts', topic TEXT NOT NULL, word TEXT NOT NULL, meaning TEXT NOT NULL, example TEXT NOT NULL, ease REAL NOT NULL DEFAULT 2.5, interval_days INTEGER NOT NULL DEFAULT 0, reps INTEGER NOT NULL DEFAULT 0, due_date TEXT NOT NULL);
            INSERT INTO vocab_cards (topic, word, meaning, example, reps, due_date) VALUES ('Custom', 'resilient', 'kiên cường', 'She is resilient.', 2, '2026-11-25');
            CREATE TABLE essays (id INTEGER PRIMARY KEY AUTOINCREMENT, mode TEXT NOT NULL, prompt TEXT NOT NULL, content TEXT NOT NULL, word_count INTEGER NOT NULL, ta REAL NOT NULL, cc REAL NOT NULL, lr REAL NOT NULL, gra REAL NOT NULL, overall REAL NOT NULL, feedback_json TEXT NOT NULL, created_at TEXT NOT NULL);
            CREATE TABLE error_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, category TEXT NOT NULL, wrong_text TEXT NOT NULL, right_text TEXT NOT NULL, count INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL);
            CREATE TABLE practice_scores (id INTEGER PRIMARY KEY AUTOINCREMENT, skill TEXT NOT NULL, source TEXT NOT NULL, score REAL NOT NULL, note TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL);
            INSERT INTO practice_scores (skill, source, score, note, created_at) VALUES ('Reading', 'Study4', 6.5, '27/40', '2026-11-24');
            CREATE TABLE roadmap_items (id INTEGER PRIMARY KEY AUTOINCREMENT, track TEXT NOT NULL DEFAULT 'ielts', phase INTEGER NOT NULL, week_label TEXT NOT NULL, title TEXT NOT NULL, done INTEGER NOT NULL DEFAULT 0);
            INSERT INTO roadmap_items (phase, week_label, title, done) VALUES (1, 'Tuần 1', 'Old Week 1', 1);
            CREATE TABLE vocab_quizzes (id INTEGER PRIMARY KEY AUTOINCREMENT, taken_at TEXT NOT NULL, mode TEXT NOT NULL, total_questions INTEGER NOT NULL, correct_count INTEGER NOT NULL, forgotten_words TEXT NOT NULL DEFAULT '[]');",
        )
        .unwrap();

        init_schema_and_seed(&conn, None).unwrap();
        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.settings.tts_accent, "en-GB");
        assert!(dash.settings.has_api_key);
        assert!(dash.roadmap[0].done);
        assert_eq!(dash.skill_bands.iter().find(|b| b.skill == "Reading").unwrap().current, 6.5);
        assert_eq!(dash.paraphrase_pairs.len(), 12);
    }

    #[test]
    fn add_vocab_card_and_cure_leech_work_end_to_end() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let added = add_vocab_card(
            &conn,
            "exacerbate",
            "làm trầm trọng thêm",
            "Rising heat can exacerbate drought.",
            "Environment",
            &today_str,
        )
        .unwrap();
        assert!(added.is_user_card);

        // Fail 3 times -> becomes leech
        review_vocab_card(&conn, added.id, 0, today).unwrap();
        review_vocab_card(&conn, added.id, 0, today).unwrap();
        let card3 = review_vocab_card(&conn, added.id, 0, today).unwrap();
        assert_eq!(card3.lapses, 3);

        let leeches = get_due_vocab(&conn, &today_str, Some(true)).unwrap();
        assert_eq!(leeches.len(), 1);
        assert_eq!(leeches[0].id, added.id);

        let cured = cure_leech_card(
            &conn,
            added.id,
            "Overcrowding can exacerbate urban traffic delays.",
            &today_str,
        )
        .unwrap();
        assert_eq!(cured.lapses, 0);
        assert!(cured.example.contains("Overcrowding"));
        assert!(get_due_vocab(&conn, &today_str, Some(true)).unwrap().is_empty());

        toggle_hide_vocab_card(&conn, added.id, true).unwrap();
        let due_after_hide = get_due_vocab(&conn, &today_str, None).unwrap();
        assert!(!due_after_hide.iter().any(|c| c.id == added.id));
        delete_vocab_card(&conn, added.id).unwrap();
    }

    #[test]
    fn micro_check_and_error_fix_quiz_and_dictation_work() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();

        let err_quiz = generate_error_fix_quiz(&conn, Some(5)).unwrap();
        assert_eq!(err_quiz.len(), 5);

        let micro = generate_micro_check(&conn).unwrap();
        assert_eq!(micro.len(), 5);
        assert_eq!(micro.iter().filter(|m| m.item_type == "vocab").count(), 2);
        assert_eq!(micro.iter().filter(|m| m.item_type == "error_fix").count(), 2);
        assert_eq!(micro.iter().filter(|m| m.item_type == "paraphrase").count(), 1);

        let micro_res = submit_micro_check(
            &conn,
            micro
                .iter()
                .map(|m| MicroCheckAnswerInput {
                    item_type: m.item_type.clone(),
                    ref_id: m.ref_id,
                    correct: true,
                    gave_up: false,
                })
                .collect(),
            today,
        )
        .unwrap();
        assert_eq!(micro_res.accuracy_pct, 100);

        let dict = get_dictation_batch(&conn, Some(5)).unwrap();
        assert_eq!(dict.len(), 5);
        assert_eq!(dict[0].level, "6.5");

        let dict_res = submit_dictation_session(
            &conn,
            vec![DictationResultInput {
                sentence_id: dict[0].id,
                accuracy_pct: 90,
                wrong_words: vec!["wednesdays".to_string()],
            }],
            today,
        )
        .unwrap();
        assert_eq!(dict_res.avg_accuracy, 90);
    }

    #[test]
    fn speaking_bank_and_sessions_and_paraphrase_crud_work() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();

        let cue = get_speaking_cue(&conn, Some(3), Some(false), None).unwrap();
        assert_eq!(cue.week_num, 3);
        assert_eq!(cue.level, "6.5");
        assert_eq!(cue.cue_bullets.len(), 4);
        assert_eq!(cue.part3_questions.len(), 4);

        let cue2 = get_speaking_cue(&conn, Some(3), Some(true), Some(cue.id)).unwrap();
        assert_ne!(cue2.id, cue.id);

        let (sess, _) = save_speaking_session(
            &conn,
            Some(cue.id),
            &cue.cue_title,
            "6.5",
            "4-3-2",
            110,
            135,
            10,
            4,
            6.0,
            6.0,
            5.5,
            6.0,
            6.0,
            "Trôi chảy hơn",
            "West lake is peaceful...",
            "{}",
            today,
        )
        .unwrap();
        assert_eq!(sess.wpm_end, 135);
        assert_eq!(list_speaking_sessions(&conn).unwrap().len(), 1);

        let p = add_paraphrase(&conn, "ubiquitous", "omnipresent, pervasive", "Cam 18 P3", "C1").unwrap();
        assert!(list_paraphrases(&conn).unwrap().iter().any(|x| x.id == p.id));
        delete_paraphrase(&conn, p.id).unwrap();
    }

    #[test]
    fn weekday_core_quests_stay_under_30_minutes_and_bonus_excluded_from_streak() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        for is_80 in [false, true] {
            for wd in [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ] {
                let q = default_quests_for_weekday(wd, is_80);
                let core_mins: i64 = q.iter().filter(|x| x.tier == "core").map(|x| x.minutes).sum();
                assert!(core_mins <= 30, "Core mins exceeded 30 on {:?}: {}", wd, core_mins);
                assert!(q.iter().any(|x| x.tier == "bonus"));
            }
        }

        let dash = get_dashboard_state(&conn).unwrap();
        let bonus_q = dash.quests.iter().find(|q| q.tier == "bonus").unwrap();
        toggle_quest(&conn, bonus_q.id).unwrap();
        let dash_after_bonus = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash_after_bonus.streak, 0); // Bonus alone does not grant streak

        let core_big = dash_after_bonus
            .quests
            .iter()
            .find(|q| q.tier == "core" && q.minutes >= 10)
            .unwrap();
        toggle_quest(&conn, core_big.id).unwrap();
        let dash_after_core = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash_after_core.streak, 1);
    }

    #[test]
    fn import_roadmap_preserves_done_weeks_and_imports_paraphrases() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let d0 = get_dashboard_state(&conn).unwrap();
        toggle_roadmap_item(&conn, d0.roadmap[0].id).unwrap();

        let md = crate::roadmap_parser::get_sample_roadmap_md();
        import_roadmap_to_db(&conn, &md, true, true).unwrap();

        let dash = get_dashboard_state(&conn).unwrap();
        assert!(dash.roadmap[0].done); // Week 1 done preserved!
        assert!(dash.paraphrase_pairs.len() >= 6);
    }

    #[test]
    fn seed_key_file_is_imported_and_deleted() {
        let temp_dir = std::env::temp_dir().join("xd4t_seed_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let seed_file = temp_dir.join("test_api_key.seed");
        fs::write(&seed_file, "AQ.TestDummyKey123456").unwrap();

        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, Some(&seed_file)).unwrap();

        assert!(!seed_file.exists());
        let s = get_settings_raw(&conn).unwrap();
        assert_eq!(s.gemini_api_key, "AQ.TestDummyKey123456");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn quest_toggle_updates_study_days_and_streak() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.streak, 0);

        let core_quest = dash
            .quests
            .iter()
            .find(|q| q.tier == "core" && q.minutes >= 10)
            .unwrap();
        let core_id = core_quest.id;
        toggle_quest(&conn, core_id).unwrap();

        let dash2 = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash2.streak, 1);
        assert!(dash2.quests.iter().find(|q| q.id == core_id).unwrap().done);
    }

    #[test]
    fn error_log_increments_duplicate_entries() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        upsert_error_log(&conn, "Grammar", "he go", "he goes").unwrap();
        upsert_error_log(&conn, "Grammar", "he go", "he goes").unwrap();
        let cnt: i64 = conn
            .query_row(
                "SELECT count FROM error_logs WHERE wrong_text = 'he go'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cnt, 2);
    }

    #[test]
    fn widget_status_builds_expected_baseline() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let ws = build_widget_status(&conn).unwrap();
        assert_eq!(ws.quests_done, 0);
        assert_eq!(ws.quests_total, 3);
        assert_eq!(ws.overall_band, 5.0);
        assert_eq!(ws.target_band, 7.0);
    }

    #[test]
    fn import_and_export_roadmap_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let md = crate::roadmap_parser::get_sample_roadmap_md();
        import_roadmap_to_db(&conn, &md, true, true).unwrap();

        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.settings.exam_date, "2027-05-15");
        assert_eq!(dash.settings.writing_checklist.len(), 5);
        assert_eq!(dash.settings.writing_focus_notes.len(), 3);
        assert!(dash.roadmap.len() >= 6);
        assert!(dash.quests.len() >= 3);
        assert!(dash.total_vocab_count >= 40);

        let exported = export_roadmap_markdown(&conn).unwrap();
        assert!(exported.contains("## Nhiệm vụ Core"));
        assert!(exported.contains("## Checklist Writing cá nhân"));
        assert!(exported.contains("### GĐ1:"));
    }

    #[test]
    fn new_db_defaults_to_pastel_male_and_onboarding_false() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let s = get_settings_public(&conn).unwrap();
        assert_eq!(s.theme, "pastel-male");
        assert!(!s.onboarding_done);
        let ws = build_widget_status(&conn).unwrap();
        assert_eq!(ws.theme, "pastel-male");
    }

    #[test]
    fn legacy_db_migrates_dark_light_and_onboarding_done() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                theme TEXT NOT NULL DEFAULT 'dark',
                gemini_api_key TEXT NOT NULL DEFAULT '',
                gemini_model TEXT NOT NULL DEFAULT 'gemini-3-flash-preview',
                exam_date TEXT NOT NULL DEFAULT '2027-05-15',
                reminder_time TEXT NOT NULL DEFAULT '21:00',
                autostart INTEGER NOT NULL DEFAULT 0,
                target_band REAL NOT NULL DEFAULT 7.0
            );
            INSERT INTO settings (id, theme, gemini_api_key) VALUES (1, 'dark', 'AQ.TestLegacyKey');",
        )
        .unwrap();

        init_schema_and_seed(&conn, None).unwrap();
        let s = get_settings_public(&conn).unwrap();
        assert_eq!(s.theme, "mono-dark");
        assert!(s.onboarding_done);

        let updated = update_settings(
            &conn,
            Some("light".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(updated.theme, "mono-light");
    }

    #[test]
    fn backup_and_restore_validation_checks_integrity_and_schema() {
        let temp_dir = std::env::temp_dir().join("xd4t_backup_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_db = temp_dir.join("src.db");
        let bak_db = temp_dir.join("bak.db");
        let bad_db = temp_dir.join("bad.db");
        let wrong_db = temp_dir.join("wrong.db");

        {
            let conn = Connection::open(&src_db).unwrap();
            init_schema_and_seed(&conn, None).unwrap();
            checkpoint_and_copy_backup(&conn, &src_db, &bak_db).unwrap();
        }
        validate_backup_file(&bak_db).unwrap();

        fs::write(&bad_db, b"not a sqlite file").unwrap();
        assert!(validate_backup_file(&bad_db).is_err());

        {
            let wrong = Connection::open(&wrong_db).unwrap();
            wrong.execute("CREATE TABLE unrelated (id INTEGER PRIMARY KEY)", []).unwrap();
        }
        assert!(validate_backup_file(&wrong_db).is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn set_study_level_switches_between_65_and_80_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let d0 = get_dashboard_state(&conn).unwrap();
        assert_eq!(d0.settings.target_band, 7.0);
        assert_eq!(d0.total_vocab_count, 60);
        assert_eq!(d0.error_logs.len(), 10);
        // Mark Tuần 1 done before switching
        toggle_roadmap_item(&conn, d0.roadmap[0].id).unwrap();

        let d80 = set_study_level(&conn, "8.0").unwrap();
        assert_eq!(d80.settings.target_band, 8.0);
        assert_eq!(d80.total_vocab_count, 60);
        assert_eq!(d80.roadmap.len(), 24);
        assert!(d80.roadmap[0].done); // Preserved!
        assert!(!d80.roadmap[0].title.starts_with('['));
        assert_eq!(d80.skill_bands[0].target, 8.5);
        assert_eq!(d80.skill_bands[1].target, 8.5);
        assert_eq!(d80.skill_bands[2].target, 7.5);
        assert_eq!(d80.skill_bands[3].target, 7.5);
        assert_eq!(d80.skill_bands[0].current, 7.0); // Fresh DB baseline for 8.0
        assert_eq!(d80.error_logs.len(), 10);
        assert!(ERROR_LOG_SEEDS_80.iter().any(|e| e.wrong_text == d80.error_logs[0].wrong_text));
        assert!(d80.paraphrase_pairs.iter().any(|p| p.question_term == "ubiquitous"));

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let due_80 = get_due_vocab(&conn, &today, None).unwrap();
        assert!(!due_80.is_empty());
        assert!(all_vocab_seeds_80().iter().any(|v| v.word == due_80[0].word));

        // Review first C1-C2 card so reps > 0, and add a custom error log
        review_vocab_card(&conn, due_80[0].id, 3, Local::now().date_naive()).unwrap();
        upsert_error_log(&conn, "Custom", "my wrong", "my right").unwrap();

        // Switching to 8.0 again does not duplicate vocab cards
        let d80_again = set_study_level(&conn, "8.0").unwrap();
        assert_eq!(d80_again.total_vocab_count, 60);

        // User enters a real practice score, then switches back to 6.5 -> score, studied card, and custom error preserved!
        add_practice_score(&conn, "Reading", "Cam 16 T1", 8.0, "35/40").unwrap();
        let d65 = set_study_level(&conn, "6.5").unwrap();
        assert_eq!(d65.settings.target_band, 7.0);
        assert_eq!(d65.skill_bands[1].target, 7.0);
        assert_eq!(d65.skill_bands[1].current, 8.0); // User score preserved!
        assert!(d65.roadmap[0].done);
        assert_eq!(d65.total_vocab_count, 61); // 60 Band 6.5 + 1 studied Band 8.0 card
        assert!(d65.error_logs.iter().any(|e| e.wrong_text == "my wrong"));
    }

    #[test]
    fn weekday_quest_rotation_covers_all_7_days_for_both_tracks() {
        for is_80 in [false, true] {
            let mut titles = std::collections::HashSet::new();
            for wd in [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ] {
                let q = default_quests_for_weekday(wd, is_80);
                let expected_len = if matches!(wd, Weekday::Sun) { 3 } else { 4 };
                assert_eq!(q.len(), expected_len);
                let main_core = q.iter().filter(|x| x.tier == "core").last().unwrap();
                titles.insert(main_core.title);
            }
            assert_eq!(titles.len(), 7);
        }
    }

    #[test]
    fn generate_vocab_quiz_supports_all_three_modes() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let assert_clean_question = |q: &QuizQuestion| {
            assert!(!q.word.contains(" vs "), "word leaked vs note: {}", q.word);
            assert!(!q.word.contains(" / "), "word leaked slash note: {}", q.word);
            assert!(!q.word.contains('('), "word leaked paren note: {}", q.word);
            assert!(
                !q.meaning.to_lowercase().contains(&q.word.trim().to_lowercase()),
                "meaning leaked answer '{}' in '{}'",
                q.word,
                q.meaning
            );
            for opt in &q.options {
                assert!(!opt.contains(" vs "), "option leaked vs note: {}", opt);
                assert!(!opt.contains(" / "), "option leaked slash note: {}", opt);
                assert!(!opt.contains('('), "option leaked paren note: {}", opt);
            }
            if q.question_type == "meaning_mcq" {
                assert!(q.cloze_sentence.is_none());
            } else if q.question_type == "cloze_mcq" {
                assert!(q.cloze_sentence.is_some());
            }
        };

        let mixed = generate_vocab_quiz(&conn, Some(10), Some("mixed".to_string())).unwrap();
        assert_eq!(mixed.len(), 10);
        let typing_n = mixed.iter().filter(|q| q.question_type == "typing").count();
        let mcq_n = mixed.len() - typing_n;
        assert_eq!(typing_n, 3);
        assert_eq!(mcq_n, 7);
        for q in &mixed {
            assert_clean_question(q);
        }

        let mcq = generate_vocab_quiz(&conn, Some(10), Some("mcq".to_string())).unwrap();
        assert_eq!(mcq.len(), 10);
        for q in &mcq {
            assert_clean_question(q);
            assert!(q.question_type == "meaning_mcq" || q.question_type == "cloze_mcq");
            assert_eq!(q.options.len(), 4);
            assert!(q.options.contains(&q.word));
            let uniq: std::collections::HashSet<&String> = q.options.iter().collect();
            assert_eq!(uniq.len(), 4);
        }

        let typing = generate_vocab_quiz(&conn, Some(5), Some("typing".to_string())).unwrap();
        assert_eq!(typing.len(), 5);
        for q in &typing {
            assert_clean_question(q);
            assert_eq!(q.question_type, "typing");
            assert!(q.options.is_empty());
            assert!(!q.hint.is_empty());
        }
    }

    #[test]
    fn submit_vocab_quiz_requeues_forgotten_cards_and_ticks_quiz_quest() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // Insert a Quiz quest for today so we can verify auto-tick
        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
             VALUES (?1, 'ielts', 'Quiz kiểm tra 10 từ vựng định kỳ', 'Test', 5, 0)",
            params![today_str],
        )
        .unwrap();

        let quiz = generate_vocab_quiz(&conn, Some(4), Some("mcq".to_string())).unwrap();
        assert_eq!(quiz.len(), 4);

        // Mark first card as already learned (reps = 3, due in future) so we prove forgetting resets it
        let future_date = (today + Duration::days(10)).format("%Y-%m-%d").to_string();
        conn.execute(
            "UPDATE vocab_cards SET reps = 3, interval_days = 10, ease = 2.5, due_date = ?1 WHERE id = ?2",
            params![future_date, quiz[0].card_id],
        )
        .unwrap();

        let inputs = vec![
            QuizAnswerInput { card_id: quiz[0].card_id, correct: false, gave_up: false },
            QuizAnswerInput { card_id: quiz[1].card_id, correct: true, gave_up: true }, // gave_up counts as forgotten
            QuizAnswerInput { card_id: quiz[2].card_id, correct: true, gave_up: false },
            QuizAnswerInput { card_id: quiz[3].card_id, correct: true, gave_up: false },
        ];

        let res = submit_vocab_quiz(&conn, "mcq".to_string(), inputs, today).unwrap();
        assert_eq!(res.total_questions, 4);
        assert_eq!(res.correct_count, 2);
        assert_eq!(res.accuracy_pct, 50);
        assert_eq!(res.forgotten_cards.len(), 2);
        assert!(res.marked_quest_done);

        // Verify forgotten card 0 was reset and appears at the head of get_due_vocab
        let (reps0, interval0, ease0, due0): (i64, i64, f64, String) = conn
            .query_row(
                "SELECT reps, interval_days, ease, due_date FROM vocab_cards WHERE id = ?1",
                params![quiz[0].card_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(reps0, 0);
        assert_eq!(interval0, 0);
        assert!(ease0 < 2.5);
        assert_eq!(due0, today_str);

        let due_list = get_due_vocab(&conn, &today_str, None).unwrap();
        let top_ids: Vec<i64> = due_list.iter().take(2).map(|c| c.id).collect();
        assert!(top_ids.contains(&quiz[0].card_id));
        assert!(top_ids.contains(&quiz[1].card_id));

        // Verify vocab_quizzes row saved
        let q_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM vocab_quizzes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(q_count, 1);

        // Verify daily_quests quiz row marked done = 1
        let quest_done: i64 = conn
            .query_row(
                "SELECT done FROM daily_quests WHERE date = ?1 AND lower(title) LIKE '%quiz%' LIMIT 1",
                params![today_str],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(quest_done, 1);
    }

    #[test]
    fn error_drills_seeds_have_60_questions_across_tracks() {
        assert_eq!(ERROR_DRILL_SEEDS_65.len(), 30);
        assert_eq!(ERROR_DRILL_SEEDS_80.len(), 30);
        for d in ERROR_DRILL_SEEDS_65.iter().chain(ERROR_DRILL_SEEDS_80.iter()) {
            assert!(!d.stem.trim().is_empty());
            assert!(!d.answer.trim().is_empty());
            if d.drill_mode != "TYPING" {
                assert!(d.options.len() >= 3, "MCQ/MINI_CASE missing options: {}", d.stem);
                assert!(d.options.contains(&d.answer), "Answer missing from options: {}", d.stem);
            }
        }

        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let drills_65 = generate_error_drills(&conn, Some(10)).unwrap();
        assert_eq!(drills_65.len(), 10);

        set_study_level(&conn, "8.0").unwrap();
        let drills_80 = generate_error_drills(&conn, Some(10)).unwrap();
        assert_eq!(drills_80.len(), 10);
    }

    #[test]
    fn error_mastery_loop_and_relapse_trigger_work() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        // 3 consecutive correct on distinct contexts -> mastered
        let first = generate_error_drills(&conn, Some(10)).unwrap().into_iter().next().unwrap();
        let eid = first.error_log_id;
        let r1 = submit_error_drill_result(&conn, eid, first.drill_id, &first.context_tag, true).unwrap();
        assert_eq!(r1.streak_distinct, 1);
        assert_eq!(r1.status, "active");

        let d2 = generate_error_drills(&conn, Some(10))
            .unwrap()
            .into_iter()
            .find(|d| d.error_log_id == eid)
            .unwrap();
        assert_ne!(d2.drill_id, first.drill_id);
        let r2 = submit_error_drill_result(&conn, eid, d2.drill_id, &d2.context_tag, true).unwrap();
        assert_eq!(r2.streak_distinct, 2);

        let d3 = generate_error_drills(&conn, Some(10))
            .unwrap()
            .into_iter()
            .find(|d| d.error_log_id == eid)
            .unwrap();
        assert_ne!(d3.drill_id, d2.drill_id);
        let r3 = submit_error_drill_result(&conn, eid, d3.drill_id, &d3.context_tag, true).unwrap();
        assert_eq!(r3.streak_distinct, 3);
        assert_eq!(r3.status, "mastered");
        assert!(r3.newly_mastered);

        // Relapse trigger when Writing AI flags a correction matching that error
        let (cat, wrong): (String, String) = conn
            .query_row("SELECT category, wrong_text FROM error_logs WHERE id = ?1", params![eid], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        let grade = GradeResult {
            ta: 6.0, cc: 6.0, lr: 6.0, gra: 5.5, overall: 6.0,
            corrections: vec![crate::models::Correction {
                wrong: wrong.clone(),
                right: "fixed".to_string(),
                category: cat.clone(),
                reason: "Repeated".to_string(),
            }],
            band65_rewrite: String::new(),
            summary_vi: String::new(),
            repeated_top_errors: vec![format!("Lỗi #1 ({cat})")],
        };
        let relapsed = trigger_error_relapse_from_grade(&conn, &grade).unwrap();
        assert!(relapsed >= 1);
        let (st, stk): (String, i64) = conn
            .query_row("SELECT status, streak_distinct FROM error_logs WHERE id = ?1", params![eid], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(st, "active");
        assert_eq!(stk, 0);
    }

    #[test]
    fn submit_error_drill_session_ticks_quiz_quest_and_study_days() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
             VALUES (?1, 'ielts', 'Quiz luyện lỗi', 'Meta', 10, 0, 'core', 'quiz')",
            params![today_str],
        ).unwrap();

        let drills = generate_error_drills(&conn, Some(3)).unwrap();
        let inputs: Vec<ErrorDrillResultInput> = drills
            .iter()
            .map(|d| ErrorDrillResultInput {
                error_log_id: d.error_log_id,
                drill_id: d.drill_id,
                context_tag: d.context_tag.clone(),
                correct: true,
            })
            .collect();
        let res = submit_error_drill_session(&conn, inputs, today).unwrap();
        assert!(res.marked_quest_done);
        assert_eq!(res.accuracy_pct, 100);
    }

    #[test]
    fn review_vocab_card_auto_ticks_srs_quest_and_updates_study_days() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
             VALUES (?1, 'ielts', 'Ôn SRS 8p', 'Meta', 8, 0, 'core', 'srs')",
            params![today_str],
        ).unwrap();

        let cards = get_due_vocab(&conn, &today_str, None).unwrap();
        review_vocab_card(&conn, cards[0].id, 2, today).unwrap();

        let srs_done: i64 = conn
            .query_row(
                "SELECT done FROM daily_quests WHERE date = ?1 AND action_hint = 'srs' LIMIT 1",
                params![today_str],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(srs_done, 1);
        let qd: i64 = conn
            .query_row("SELECT quests_done FROM study_days WHERE date = ?1", params![today_str], |r| r.get(0))
            .unwrap();
        assert!(qd >= 1);
    }

    #[test]
    fn cure_leech_card_validates_target_word_presence() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today_str = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let card = add_vocab_card(&conn, "mitigate", "giảm thiểu", "Trees mitigate heat.", "Env", &today_str).unwrap();

        // Off-topic sentence without word/root must be rejected
        let bad = cure_leech_card(&conn, card.id, "I went to the market yesterday.", &today_str);
        assert!(bad.is_err());

        // Sentence containing inflected form passes
        let ok = cure_leech_card(&conn, card.id, "Green roofs help mitigating summer heat.", &today_str);
        assert!(ok.is_ok());
    }

    #[test]
    fn log_dictation_errors_creates_clean_pair_and_cloze_drill() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        log_dictation_errors(
            &conn,
            1,
            "Bẫy số nhiều -s",
            "The community library opens at quarter to nine on Wednesdays.",
            vec![DictationErrorPair {
                wrong: "open".to_string(),
                right: "opens".to_string(),
            }],
        ).unwrap();

        let (cat, mode, eid): (String, String, i64) = conn
            .query_row(
                "SELECT category, drill_mode, id FROM error_logs WHERE wrong_text = 'open' AND right_text = 'opens'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(cat, "Listening · Chính tả");
        assert_eq!(mode, "TYPING");

        let (stem, ans): (String, String) = conn
            .query_row(
                "SELECT stem, answer FROM error_drills WHERE error_log_id = ?1 LIMIT 1",
                params![eid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(stem.contains("________"));
        assert!(!stem.contains("opens"));
        assert_eq!(ans, "opens");
    }

    #[test]
    fn writing_prompts_24_and_mark_practiced_ticks_writing_quest() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done, tier, action_hint)
             VALUES (?1, 'ielts', 'Writing 18p', 'Meta', 18, 0, 'core', 'writing')",
            params![today_str],
        ).unwrap();

        let prompts65 = list_writing_prompts(&conn, "6.5").unwrap();
        let prompts80 = list_writing_prompts(&conn, "8.0").unwrap();
        assert_eq!(prompts65.len(), 12);
        assert_eq!(prompts80.len(), 12);

        let ticked = mark_writing_practiced(&conn, Some(&prompts65[0].id), today).unwrap();
        assert!(ticked);

        let practiced: i64 = conn
            .query_row("SELECT practiced_count FROM writing_prompts ORDER BY id ASC LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(practiced, 1);
    }

    #[test]
    fn refill_content_pack_partial_validation_and_dedup_work() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let today_str = Local::now().date_naive().format("%Y-%m-%d").to_string();

        let pack = TopicPackPayload {
            vocab: vec![
                crate::models::TopicPackVocab { word: "curb urban sprawl".into(), meaning: "kiềm chế đô thị hóa tràn lan".into(), example: "Zoning helps curb urban sprawl.".into() },
                crate::models::TopicPackVocab { word: "".into(), meaning: "invalid empty".into(), example: "bad".into() },
            ],
            dictations: vec![
                crate::models::TopicPackDictation { section: "Part 4".into(), trap_focus: "-s".into(), sentence: "Green belts help cities preserve biodiversity and curb urban sprawl.".into(), meaning_vi: "Vành đai xanh bảo vệ đa dạng sinh học.".into() },
                crate::models::TopicPackDictation { section: "Part 1".into(), trap_focus: "short".into(), sentence: "Too short.".into(), meaning_vi: "Ngắn".into() },
            ],
            writing_prompts: vec![
                crate::models::TopicPackPrompt { task_type: "task2".into(), title: "Task 2 · Green Belt Policy".into(), prompt_text: "Discuss green belts around megacities.".into(), outline_hint: "Intro/Body".into(), target_vocab: vec!["curb urban sprawl".into()], chart_spec_json: "".into() },
            ],
            paraphrases: vec![
                crate::models::TopicPackParaphrase { question_term: "expand rapidly".into(), passage_paraphrase: "mushroom, sprawl unchecked".into(), source_type: "Reading".into(), trap_note: "scope".into() },
            ],
            error_drills: vec![
                TopicPackDrill { error_index: 0, drill_mode: "GRAMMAR_MCQ".into(), context_tag: "urban".into(), stem: "Pick valid:".into(), options: vec!["A".into(), "B".into(), "C".into(), "D".into()], answer: "A".into(), rationale_vi: "Ok".into() },
                TopicPackDrill { error_index: 0, drill_mode: "GRAMMAR_MCQ".into(), context_tag: "urban".into(), stem: "Invalid missing answer".into(), options: vec!["X".into(), "Y".into()], answer: "Z".into(), rationale_vi: "Bad".into() },
            ],
        };

        let res = apply_topic_pack_to_db(&conn, "Urbanization", "6.5", &pack, &today_str, "test").unwrap();
        assert_eq!(res.vocab_added, 1);
        assert_eq!(res.dictations_added, 1);
        assert_eq!(res.prompts_added, 1);
        assert_eq!(res.paraphrases_added, 1);
        assert_eq!(res.drills_added, 1);

        // Re-applying same pack deduplicates everything
        let res2 = apply_topic_pack_to_db(&conn, "Urbanization", "6.5", &pack, &today_str, "test").unwrap();
        assert_eq!(res2.vocab_added, 0);
        assert_eq!(res2.dictations_added, 0);
        assert_eq!(res2.prompts_added, 0);
        assert_eq!(res2.paraphrases_added, 0);
        assert_eq!(res2.drills_added, 0);
    }

    #[test]
    fn task1_seeds_all_have_valid_chart_spec_json_and_auto_refill_due_works() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        for level in ["6.5", "8.0"] {
            let prompts = list_writing_prompts(&conn, level).unwrap();
            let task1s: Vec<_> = prompts.iter().filter(|p| p.mode == "task1").collect();
            assert_eq!(task1s.len(), 4);
            for p in task1s {
                assert!(!p.chart_spec_json.trim().is_empty(), "missing chart spec on {}", p.title);
                let parsed: serde_json::Value = serde_json::from_str(&p.chart_spec_json).unwrap();
                assert!(parsed.get("type").is_some());
            }
        }

        // No API key -> should_auto_refill is false even on Sunday
        let sunday = NaiveDate::from_ymd_opt(2026, 11, 29).unwrap();
        assert_eq!(sunday.weekday(), Weekday::Sun);
        let st_no_key = check_auto_refill_due(&conn, sunday).unwrap();
        assert!(!st_no_key.should_auto_refill);

        // With API key & enabled -> triggers on Sunday
        conn.execute("UPDATE settings SET gemini_api_key = 'test-fake-key' WHERE id = 1", []).unwrap();
        let st_sun = check_auto_refill_due(&conn, sunday).unwrap();
        assert!(st_sun.should_auto_refill);
        assert_eq!(st_sun.trigger_reason, "sunday");

        // Same day lock after refill
        conn.execute(
            "UPDATE settings SET last_ai_refill_at = '2026-11-29', last_ai_refill_topic = ?1 WHERE id = 1",
            params![st_sun.recommended_topic],
        ).unwrap();
        let st_same_day = check_auto_refill_due(&conn, sunday).unwrap();
        assert!(!st_same_day.should_auto_refill);

        let next_sunday = NaiveDate::from_ymd_opt(2026, 12, 6).unwrap();
        // Ticking 2 roadmap weeks advances recommended_topic to Week 3's topic
        conn.execute(
            "UPDATE roadmap_items SET done = 1 WHERE id IN (SELECT id FROM roadmap_items WHERE track = 'ielts' ORDER BY id ASC LIMIT 2)",
            [],
        )
        .unwrap();
        let st_week3 = check_auto_refill_due(&conn, next_sunday).unwrap();
        assert_eq!(st_week3.recommended_topic, IELTS_TOPICS[2]);
        assert_ne!(st_week3.recommended_topic, st_sun.recommended_topic);

        // Toggle off -> disabled even 7 days later
        conn.execute("UPDATE settings SET auto_refill_enabled = 0 WHERE id = 1", []).unwrap();
        let st_disabled = check_auto_refill_due(&conn, next_sunday).unwrap();
        assert!(!st_disabled.enabled);
        assert!(!st_disabled.should_auto_refill);
    }
}
