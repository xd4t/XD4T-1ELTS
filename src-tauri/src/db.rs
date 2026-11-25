use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{
    DailyQuest, DashboardState, DictationResultInput, DictationSentence, DictationSubmitResult,
    ErrorFixQuestion, ErrorLog, Essay, ForgottenVocabCard, MicroCheckAnswerInput, MicroCheckItem,
    MicroCheckSubmitResult, ParaphrasePair, PracticeScore, QuizAnswerInput, QuizQuestion,
    QuizSubmitResult, RoadmapItem, Settings, SettingsPublic, SkillBand, SpeakingCue,
    SpeakingSession, StudyDay, VocabCard, VocabQuizStatus,
};
use crate::roadmap_parser::{infer_quest_action_hint, parse_roadmap_markdown, ParsedQuest};
use crate::seed_data::{
    default_writing_prompts_for_band, DICTATION_SEEDS, ERROR_LOG_SEEDS_65, ERROR_LOG_SEEDS_80,
    PARAPHRASE_SEEDS, ROADMAP_SEEDS_65, ROADMAP_SEEDS_80, SPEAKING_SEEDS, VOCAB_SEEDS,
    VOCAB_SEEDS_80,
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

    let vocab_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocab_cards", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if vocab_count == 0 {
        for v in VOCAB_SEEDS {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date)
                 VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5)",
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
        for e in ERROR_LOG_SEEDS_65 {
            conn.execute(
                "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![e.category, e.wrong_text, e.right_text, e.count, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // v1.6.0 seeds via WHERE NOT EXISTS (safe for migrated v1.5.0 DBs)
    for p in PARAPHRASE_SEEDS {
        conn.execute(
            "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at)
             SELECT ?1, ?2, ?3, ?4, ?5
             WHERE NOT EXISTS (SELECT 1 FROM paraphrase_pairs WHERE lower(question_term) = lower(?1) AND lower(passage_paraphrase) = lower(?2))",
            params![p.question_term, p.passage_paraphrase, p.source_type, p.trap_note, today],
        )
        .map_err(|e| e.to_string())?;
    }

    for d in DICTATION_SEEDS {
        conn.execute(
            "INSERT INTO dictation_sentences (level, section, trap_focus, sentence, meaning_vi)
             SELECT ?1, ?2, ?3, ?4, ?5
             WHERE NOT EXISTS (SELECT 1 FROM dictation_sentences WHERE lower(sentence) = lower(?4))",
            params![d.level, d.section, d.trap_focus, d.sentence, d.meaning_vi],
        )
        .map_err(|e| e.to_string())?;
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
                COALESCE(tts_accent, 'en-GB')
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

    conn.execute(
        "UPDATE settings
         SET theme = ?1, gemini_api_key = ?2, gemini_model = ?3, exam_date = ?4,
             reminder_time = ?5, autostart = ?6, target_band = ?7, onboarding_done = ?8, tts_accent = ?9
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
            new_accent
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
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
         FROM vocab_cards
         WHERE COALESCE(hidden, 0) = 0 AND COALESCE(lapses, 0) >= 3
         ORDER BY lapses DESC, due_date ASC, id DESC
         LIMIT 40"
    } else if target_band >= 7.5 {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
         FROM vocab_cards
         WHERE due_date <= ?1 AND COALESCE(hidden, 0) = 0
         ORDER BY COALESCE(is_user_card, 0) DESC,
                  (CASE WHEN COALESCE(lapses, 0) >= 3 THEN 0 ELSE 1 END) ASC,
                  (CASE WHEN reps = 0 AND ease < 2.5 THEN 0 ELSE 1 END) ASC,
                  due_date ASC, ease ASC, id DESC
         LIMIT 40"
    } else {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
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
        "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date, lapses, hidden, is_user_card)
         VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5, 0, 0, 1)",
        params![tp, w, m, ex, today],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
         FROM vocab_cards WHERE id = ?1",
        params![id],
        map_vocab_card,
    )
    .map_err(|e| e.to_string())
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
    let updated = conn
        .execute(
            "UPDATE vocab_cards
             SET example = ?1, lapses = 0, reps = 0, interval_days = 0, due_date = ?2, is_user_card = 1
             WHERE id = ?3",
            params![sent, today, card_id],
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        return Err("Không tìm thấy thẻ từ vựng cần chữa.".to_string());
    }
    conn.query_row(
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date,
                COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
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
    let clean_word = word.trim();
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
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
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
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
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

    Ok(VocabCard {
        ease: outcome.ease,
        interval_days: outcome.interval_days,
        reps: outcome.reps,
        due_date: outcome.due_date,
        lapses: new_lapses,
        ..card
    })
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
                    COALESCE(lapses, 0), COALESCE(hidden, 0), COALESCE(is_user_card, 0)
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

    // 2. Two error_logs items (prioritizing highest count)
    let mut e_stmt = conn
        .prepare(
            "SELECT id, category, wrong_text, right_text, count, created_at
             FROM error_logs
             ORDER BY count DESC, id DESC
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
            "SELECT id, level, section, trap_focus, sentence, meaning_vi, practiced_count, last_accuracy
             FROM dictation_sentences
             WHERE level = ?1
             ORDER BY practiced_count ASC, id ASC
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
    let mut stmt = conn
        .prepare(
            "SELECT id, question_term, passage_paraphrase, source_type, trap_note, created_at
             FROM paraphrase_pairs
             ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ParaphrasePair {
                id: r.get(0)?,
                question_term: r.get(1)?,
                passage_paraphrase: r.get(2)?,
                source_type: r.get(3)?,
                trap_note: r.get(4)?,
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
    let now = Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![qt, pp, source_type.trim(), trap_note.trim(), now],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, question_term, passage_paraphrase, source_type, trap_note, created_at
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
            "SELECT id, category, wrong_text, right_text, count, created_at
             FROM error_logs
             ORDER BY count DESC, id DESC
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
        conn.execute(
            "UPDATE error_logs SET count = count + 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let now = Local::now().format("%Y-%m-%d").to_string();
        conn.execute(
            "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at)
             VALUES (?1, ?2, ?3, 1, ?4)",
            params![category.trim(), wrong_text.trim(), right_text.trim(), now],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
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
        prompts: default_writing_prompts_for_band(raw_settings.target_band),
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

    if is_80 {
        for v in VOCAB_SEEDS {
            conn.execute(
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0 AND COALESCE(is_user_card, 0) = 0",
                params![v.word],
            )
            .map_err(|e| e.to_string())?;
        }
        for v in VOCAB_SEEDS_80 {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE word = ?2)",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    } else {
        for v in VOCAB_SEEDS_80 {
            conn.execute(
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0 AND COALESCE(is_user_card, 0) = 0",
                params![v.word],
            )
            .map_err(|e| e.to_string())?;
        }
        for v in VOCAB_SEEDS {
            conn.execute(
                "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date)
                 SELECT 'ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5
                 WHERE NOT EXISTS (SELECT 1 FROM vocab_cards WHERE word = ?2)",
                params![v.topic, v.word, v.meaning, v.example, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let mut has_user_errors = false;
    {
        let mut stmt = conn
            .prepare("SELECT wrong_text, right_text FROM error_logs")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let in_65 = ERROR_LOG_SEEDS_65
                .iter()
                .any(|e| e.wrong_text == row.0 && e.right_text == row.1);
            let in_80 = ERROR_LOG_SEEDS_80
                .iter()
                .any(|e| e.wrong_text == row.0 && e.right_text == row.1);
            if !in_65 && !in_80 {
                has_user_errors = true;
                break;
            }
        }
    }
    if !has_user_errors {
        conn.execute("DELETE FROM error_logs", [])
            .map_err(|e| e.to_string())?;
        let err_seeds = if is_80 { ERROR_LOG_SEEDS_80 } else { ERROR_LOG_SEEDS_65 };
        for e in err_seeds {
            conn.execute(
                "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![e.category, e.wrong_text, e.right_text, e.count, today],
            )
            .map_err(|e| e.to_string())?;
        }
    }

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
                    "INSERT INTO paraphrase_pairs (question_term, passage_paraphrase, source_type, trap_note, created_at)
                     SELECT ?1, ?2, ?3, ?4, ?5
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
    fn schema_seed_imports_40_vocab_24_roadmap_24_dictation_48_speaking_6_paraphrase() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.total_vocab_count, 40);
        assert_eq!(dash.roadmap.len(), 24);
        assert_eq!(dash.paraphrase_pairs.len(), 6);
        assert_eq!(dash.heatmap.len(), 70);
        assert_eq!(dash.prompts.len(), 12);

        let dict_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM dictation_sentences", [], |r| r.get(0))
            .unwrap();
        assert_eq!(dict_count, 24);

        let spk_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM speaking_bank", [], |r| r.get(0))
            .unwrap();
        assert_eq!(spk_count, 48);
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
        assert_eq!(dash.paraphrase_pairs.len(), 6);
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
}
