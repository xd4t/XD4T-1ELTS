use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{
    DailyQuest, DashboardState, ErrorLog, Essay, ForgottenVocabCard, PracticeScore,
    QuizAnswerInput, QuizQuestion, QuizSubmitResult, RoadmapItem, Settings, SettingsPublic,
    SkillBand, StudyDay, VocabCard, VocabQuizStatus,
};
use crate::roadmap_parser::{parse_roadmap_markdown, ParsedQuest};
use crate::seed_data::{
    default_writing_prompts_for_band, ERROR_LOG_SEEDS_65, ERROR_LOG_SEEDS_80, ROADMAP_SEEDS_65,
    ROADMAP_SEEDS_80, VOCAB_SEEDS, VOCAB_SEEDS_80,
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
    map.insert(1, "GĐ1 · Diệt lỗi nền & 5 khung T2 + 3 biểu đồ T1 (T1–7)".to_string());
    map.insert(2, "GĐ2 · Tăng tốc & 6 cấu trúc nâng cao (T8–15)".to_string());
    map.insert(3, "GĐ3 · Vùng đệm 7.0 & Forecast thực chiến (T16–21)".to_string());
    map.insert(4, "GĐ4 · Nước rút phòng thi (T22–24)".to_string());
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
            onboarding_done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS daily_quests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            track TEXT NOT NULL DEFAULT 'ielts',
            title TEXT NOT NULL,
            meta TEXT NOT NULL,
            minutes INTEGER NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
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
            due_date TEXT NOT NULL
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
        "#,
    )
    .map_err(|e| format!("Schema init error: {e}"))?;

    conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
        .map_err(|e| format!("Settings seed error: {e}"))?;

    for col_sql in [
        "ALTER TABLE settings ADD COLUMN writing_checklist_json TEXT NOT NULL DEFAULT '[]'",
        "ALTER TABLE settings ADD COLUMN writing_focus_json TEXT NOT NULL DEFAULT '[]'",
        "ALTER TABLE settings ADD COLUMN skill_targets_json TEXT NOT NULL DEFAULT '{}'",
        "ALTER TABLE settings ADD COLUMN phase_labels_json TEXT NOT NULL DEFAULT '{}'",
        "ALTER TABLE settings ADD COLUMN daily_quests_template_json TEXT NOT NULL DEFAULT '[]'",
    ] {
        let _ = conn.execute(col_sql, []);
    }

    // Detect legacy v1.0/v1.1 DBs when adding onboarding_done column
    let added_onboarding = conn
        .execute(
            "ALTER TABLE settings ADD COLUMN onboarding_done INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .is_ok();

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

    // Backward-compatible theme migration: "dark" -> "mono-dark", "light" -> "mono-light"
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
    // Delete seed only after SQLite commit succeeded
    let _ = fs::remove_file(seed_path);
    Ok(())
}

pub fn default_quests_for_weekday(weekday: Weekday, is_80: bool) -> Vec<(&'static str, &'static str, i64)> {
    if is_80 {
        match weekday {
            Weekday::Mon => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 2 · Collocations Band 7.5–8.0", 8),
                ("Writing: Luyện cấu trúc C1 & viết lại đoạn cũ nâng Band 7.5+", "Sửa lỗi từ Sổ lỗi · nâng +1.0 band", 15),
                ("Listening: Soi transcript & bẫy Distractors Part 3–4 (Study4)", "Bắt nối âm, đuôi s/ed & bẫy đổi ý", 12),
            ],
            Weekday::Tue => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 3 · Collocations Band 7.5–8.0", 8),
                ("Reading: 1 Passage 2–3 trên Study4 bấm giờ 18p & nhập điểm", "Phân tích cấu trúc đoạn & phạm vi ý", 15),
                ("Lập 5 cặp Paraphrase C1 từ bài đọc vào Sổ lỗi/SRS", "Săn từ đồng nghĩa học thuật Passage 3", 10),
            ],
            Weekday::Wed => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 4 · Collocations Band 7.5–8.0", 8),
                ("Writing: Viết Mở bài + 1 Body (PEEL + Counter-argument) & chấm AI", "Lập luận sâu + hedging chuẩn 7.5+", 15),
                ("Speaking: Nói lại luận điểm vừa viết theo A.R.E.A trong 2 phút", "Ghi âm tự nghe · chuyển từ viết sang nói", 10),
            ],
            Weekday::Thu => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 5 · Collocations Band 7.5–8.0", 8),
                ("Listening: Luyện Part 3 & Part 4 trên Study4 bấm giờ & nhập điểm", "Bắt bẫy distractor agreement", 15),
                ("Phân tích câu sai & ghi bẫy đổi ý vào Sổ lỗi", "Chống mất điểm oan Part 3–4", 10),
                ("Quiz C1–C2 giữa tuần: 10 cụm từ nâng cao (rà từ quên để ôn lại)", "Thứ 5 · Kiểm tra trí nhớ từ vựng · đẩy từ quên về SRS", 5),
            ],
            Weekday::Fri => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 6 · Collocations Band 7.5–8.0", 8),
                ("Speaking: Luyện 1 đề Part 2 (2 phút) + 3 câu trừu tượng Part 3", "Mở rộng lập luận đa chiều · tự nhiên", 15),
                ("Reading tốc độ: Luyện riêng T/F/NG suy luận hoặc Matching Headings", "Xử lý dạng bẫy hay mất điểm", 10),
            ],
            Weekday::Sat => vec![
                ("Ôn 12 cụm từ C1–C2 & Hedging (SRS)", "Thứ 7 · Collocations Band 7.5–8.0", 8),
                ("Writing Full: Viết 1 bài hoàn chỉnh + soát Checklist 3p & chấm AI 7.5+", "Bấm giờ chuẩn · kiểm tra 6 lỗi tinh tế", 20),
                ("Reading Full Test (3 Passages bấm giờ 54p trên Study4) & nhập điểm", "Mục tiêu 35–38/40 câu", 15),
            ],
            Weekday::Sun => vec![
                ("Writing Rewrite: Viết lại đoạn yếu nhất của bài T7 đạt chuẩn 7.5–8.0", "Chủ nhật · Nâng cấp lập luận & collocation", 15),
                ("Listening Full Test (4 Parts trên Study4) & nhập điểm", "Mục tiêu 36–39/40 câu", 15),
                ("Rà soát Sổ lỗi tuần & ôn từ vựng chủ đề mới", "Tổng kết tuần · giữ vững nhịp 8.0", 10),
                ("Quiz C1–C2 tổng kết tuần: 10 cụm từ (quên từ nào đẩy về SRS)", "Chủ nhật · Rà soát trí nhớ · giữ vững vốn từ 8.0", 5),
            ],
        }
    } else {
        match weekday {
            Weekday::Mon => vec![
                ("Ôn 10 thẻ Collocations & Word Family (SRS)", "Thứ 2 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Writing: Sửa 3 nhóm lỗi từ Sổ lỗi / luyện 6 cấu trúc câu viết lại bài cũ", "Độ chính xác / Cấu trúc · GĐ1–2", 15),
                ("Listening Dictation: Chép chính tả 1 đoạn ngắn bắt đuôi s/ed", "DailyDictation · bắt nối âm & âm cuối", 12),
            ],
            Weekday::Tue => vec![
                ("Ôn 10 thẻ Collocations (SRS)", "Thứ 3 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Reading: 1 Passage trên Study4 bấm giờ 20p & chữa theo Linearthinking", "Phân tích chủ–vị & bẫy từ tuyệt đối", 15),
                ("Lập 5 cặp Paraphrase từ bài đọc & nhập điểm Reading", "Ghi cặp đồng nghĩa vào Sổ lỗi / SRS", 10),
            ],
            Weekday::Wed => vec![
                ("Ôn 10 thẻ Collocations (SRS)", "Thứ 4 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Writing: Ôn khung dạng đề tuần + viết Mở bài & 1 Body (chấm AI)", "Soát checklist 3p trước khi nộp AI", 15),
                ("Speaking: Nói lại chính luận điểm vừa viết trong 2 phút (ghi âm)", "Chuyển ý từ Writing sang Speaking", 10),
            ],
            Weekday::Thu => vec![
                ("Ôn 10 thẻ Collocations (SRS)", "Thứ 5 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Listening: Làm 2 Sections trên Study4 bấm giờ & soi Transcript", "Cambridge Practice · nghe chủ động", 15),
                ("Đánh dấu bẫy Distractors & nhập điểm Listening", "Chép chính tả lại đúng câu nghe sai", 10),
                ("Quiz kiểm tra 10 từ vựng định kỳ (quên từ nào đẩy về SRS học lại)", "Thứ 5 · Kiểm tra trí nhớ từ vựng · chống rơi rụng", 5),
            ],
            Weekday::Fri => vec![
                ("Ôn 10 thẻ Collocations (SRS)", "Thứ 6 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Speaking: Luyện Part 1/2/3 theo khung A.R.E.A (chú ý âm cuối & chia động từ)", "Ghi âm tự nghe lại sửa lỗi thì", 15),
                ("Reading tốc độ: Luyện riêng 1 dạng hay sai (T/F/NG hoặc Matching Headings)", "Xử lý dạng bài hay mất điểm", 10),
            ],
            Weekday::Sat => vec![
                ("Ôn 10 thẻ Collocations (SRS)", "Thứ 7 · SRS Flashcards · giữ chuỗi mỗi ngày", 8),
                ("Writing Full: Viết 1 bài hoàn chỉnh + soát Checklist 3p & chấm AI 4 tiêu chí", "Bấm giờ chuẩn · cập nhật Sổ lỗi", 20),
                ("Reading Full Test trên Study4 bấm giờ & nhập điểm", "Mục tiêu vùng đệm 30–34/40 câu", 15),
            ],
            Weekday::Sun => vec![
                ("Writing Rewrite: Bắt buộc viết lại đoạn yếu nhất của bài T7 đạt ≥ 6.5", "Chủ nhật · sửa triệt để lỗi đoạn yếu", 15),
                ("Listening Full Test trên Study4 & nhập điểm", "Làm 4 Sections bấm giờ + chữa 20p", 15),
                ("Tổng kết Sổ lỗi tuần & ôn 12 cụm từ chủ đề mới", "Từ vựng ngữ cảnh · nạp vào SRS", 10),
                ("Quiz tổng kết tuần: 10 từ vựng (rà từ hay quên để ôn lại)", "Chủ nhật · Rà soát trí nhớ · đẩy từ quên về SRS", 5),
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
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0)",
                    params![today, q.title, q.meta, q.minutes],
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
            for (title, meta, mins) in defaults {
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0)",
                    params![today, title, meta, mins],
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
                onboarding_done, writing_checklist_json, writing_focus_json, skill_targets_json, phase_labels_json
         FROM settings WHERE id = 1",
        [],
        |r| {
            let raw_theme: String = r.get(0)?;
            let chk_json: String = r.get(8)?;
            let foc_json: String = r.get(9)?;
            let tgt_json: String = r.get(10)?;
            let ph_json: String = r.get(11)?;

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

    conn.execute(
        "UPDATE settings
         SET theme = ?1, gemini_api_key = ?2, gemini_model = ?3, exam_date = ?4,
             reminder_time = ?5, autostart = ?6, target_band = ?7, onboarding_done = ?8
         WHERE id = 1",
        params![
            new_theme,
            new_key,
            new_model,
            new_exam,
            new_rem,
            if new_auto { 1 } else { 0 },
            new_target,
            if new_onboarding { 1 } else { 0 }
        ],
    )
    .map_err(|e| e.to_string())?;

    get_settings_public(conn)
}

pub fn toggle_quest(conn: &Connection, quest_id: i64) -> Result<(), String> {
    let (date, minutes, done): (String, i64, i64) = conn
        .query_row(
            "SELECT date, minutes, done FROM daily_quests WHERE id = ?1",
            params![quest_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    let new_done = if done == 0 { 1 } else { 0 };
    conn.execute(
        "UPDATE daily_quests SET done = ?1 WHERE id = ?2",
        params![new_done, quest_id],
    )
    .map_err(|e| e.to_string())?;

    let delta_min = if new_done == 1 { minutes } else { -minutes };
    let delta_q = if new_done == 1 { 1 } else { -1 };
    adjust_study_day(conn, &date, delta_min, delta_q)
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

    // If today has not been studied yet, streak can still continue from yesterday
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

pub fn get_due_vocab(conn: &Connection, today: &str) -> Result<Vec<VocabCard>, String> {
    let target_band: f64 = conn
        .query_row(
            "SELECT COALESCE(target_band, 7.0) FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7.0);
    let order_sql = if target_band >= 7.5 {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date
         FROM vocab_cards
         WHERE due_date <= ?1
         ORDER BY (CASE WHEN reps = 0 AND ease < 2.5 THEN 0 ELSE 1 END) ASC, due_date ASC, ease ASC, id DESC
         LIMIT 40"
    } else {
        "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date
         FROM vocab_cards
         WHERE due_date <= ?1
         ORDER BY (CASE WHEN reps = 0 AND ease < 2.5 THEN 0 ELSE 1 END) ASC, due_date ASC, ease ASC, id ASC
         LIMIT 40"
    };
    let mut stmt = conn.prepare(order_sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![today], |r| {
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
            })
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
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
        // Ensure char boundary safety (ASCII/UTF-8 lowercase can shift lengths in rare cases, check slice)
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
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date
             FROM vocab_cards",
        )
        .map_err(|e| e.to_string())?;
    let all_cards: Vec<VocabCard> = stmt
        .query_map([], |r| {
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if all_cards.is_empty() {
        return Ok(Vec::new());
    }

    let mut studied: Vec<VocabCard> = all_cards.iter().filter(|c| c.reps >= 1).cloned().collect();
    let mut unstudied: Vec<VocabCard> = all_cards.iter().filter(|c| c.reps == 0).cloned().collect();

    // Prioritize studied cards that are easiest to forget (lowest ease, largest interval, earliest due_date)
    studied.sort_by(|a, b| {
        a.ease
            .partial_cmp(&b.ease)
            .unwrap_or(std::cmp::Ordering::Equal)
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
        ^ (all_cards.len() as u64 * 0x9e37_79b9);

    let mut pool = Vec::with_capacity(all_cards.len());
    pool.extend(studied);
    pool.extend(unstudied);

    let pick_n = target_count.min(pool.len());
    let candidate_window = (pick_n + 4).min(pool.len());
    shuffle_slice(&mut pool[..candidate_window], &mut seed);
    let selected = &pool[..pick_n];

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

    let mut questions = Vec::with_capacity(pick_n);
    for (idx, card) in selected.iter().enumerate() {
        let cloze_sentence = make_cloze_sentence(&card.example, &card.word);
        let desired_type = if idx >= mcq_count {
            "typing"
        } else if idx % 2 == 1 && cloze_sentence.is_some() {
            "cloze_mcq"
        } else {
            "meaning_mcq"
        };

        let options = if desired_type == "typing" {
            Vec::new()
        } else {
            let mut distractors: Vec<String> = all_cards
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
                "UPDATE vocab_cards SET reps = 0, interval_days = 0, ease = ?1, due_date = ?2 WHERE id = ?3",
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
             WHERE date = ?1 AND track = 'ielts' AND done = 0 AND lower(title) LIKE '%quiz%'",
            params![today_str],
        )
        .map_err(|e| e.to_string())?;
    let marked_quest_done = updated_quests > 0;

    let quests_done_today: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts' AND done = 1",
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
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date
             FROM vocab_cards WHERE id = ?1",
            params![card_id],
            |r| {
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
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let outcome = apply_sm2(card.ease, card.interval_days, card.reps, quality, today);
    conn.execute(
        "UPDATE vocab_cards SET ease = ?1, interval_days = ?2, reps = ?3, due_date = ?4 WHERE id = ?5",
        params![
            outcome.ease,
            outcome.interval_days,
            outcome.reps,
            outcome.due_date,
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
        ..card
    })
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
            "SELECT id, date, track, title, meta, minutes, done
             FROM daily_quests WHERE date = ?1 ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;
    let quests: Vec<DailyQuest> = q_stmt
        .query_map(params![today], |r| {
            Ok(DailyQuest {
                id: r.get(0)?,
                date: r.get(1)?,
                track: r.get(2)?,
                title: r.get(3)?,
                meta: r.get(4)?,
                minutes: r.get(5)?,
                done: r.get::<_, i64>(6)? != 0,
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
            "SELECT COUNT(*) FROM vocab_cards WHERE due_date <= ?1",
            params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total_vocab_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocab_cards", [], |r| r.get(0))
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

    let mut e_stmt = conn
        .prepare("SELECT id, category, wrong_text, right_text, count, created_at FROM error_logs ORDER BY count DESC, id DESC LIMIT 30")
        .map_err(|e| e.to_string())?;
    let error_logs: Vec<ErrorLog> = e_stmt
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
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

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
        quiz_status,
        roadmap,
        error_logs,
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

    // Only update baseline practice_scores if user has NOT entered any custom test scores
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

    // Preserve roadmap completion status by week_label
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

    // Swap unreviewed seed vocab cards (reps = 0) between 6.5 and 8.0 while preserving all studied/custom cards
    if is_80 {
        for v in VOCAB_SEEDS {
            conn.execute(
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0",
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
                "DELETE FROM vocab_cards WHERE word = ?1 AND reps = 0",
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

    // Swap error_logs only if all existing rows belong to default seed sets (preserve 100% if user added any)
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

    // Update today's 3 daily quests while preserving done status by position
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
    for (idx, (title, meta, mins)) in defaults.iter().enumerate() {
        let was_done = prev_done.get(idx).copied().unwrap_or(false);
        conn.execute(
            "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
             VALUES (?1, 'ielts', ?2, ?3, ?4, ?5)",
            params![today, title, meta, mins, if was_done { 1 } else { 0 }],
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
                conn.execute(
                    "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
                     VALUES (?1, 'ielts', ?2, ?3, ?4, 0)",
                    params![today, q.title, q.meta, q.minutes],
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
                conn.execute(
                    "INSERT INTO roadmap_items (track, phase, week_label, title, done)
                     VALUES ('ielts', ?1, ?2, ?3, 0)",
                    params![w.phase, w.week_label, w.title],
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
                        "INSERT INTO vocab_cards (track, topic, word, meaning, example, ease, interval_days, reps, due_date)
                         VALUES ('ielts', ?1, ?2, ?3, ?4, 2.5, 0, 0, ?5)",
                        params![v.topic, v.word, v.meaning, v.example, today],
                    )
                    .map_err(|e| e.to_string())?;
                }
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
    let total_mins: i64 = dash.quests.iter().map(|q| q.minutes).sum::<i64>().max(30);

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
    lines.push(format!("daily_minutes: {}", total_mins));
    lines.push(format!("reminder_time: {}", dash.settings.reminder_time));
    lines.push("---".to_string());
    lines.push(String::new());

    lines.push("## Nhiệm vụ hàng ngày".to_string());
    for q in &dash.quests {
        lines.push(format!("- [{}p] {} | {}", q.minutes, q.title, q.meta));
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

    let mut v_stmt = conn
        .prepare("SELECT topic, word, meaning, example FROM vocab_cards ORDER BY id ASC LIMIT 20")
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
    lines.push("## Từ vựng SRS ban đầu".to_string());
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
        lines.push(format!("- {}: {}", r.week_label, r.title));
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
             FROM daily_quests WHERE date = ?1 AND track = 'ielts'",
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
        quests_total,
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
    fn schema_seed_imports_40_vocab_and_24_roadmap() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();
        let dash = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash.total_vocab_count, 40);
        assert_eq!(dash.roadmap.len(), 24);
        assert_eq!(dash.quests.len(), 3);
        assert_eq!(dash.heatmap.len(), 70);
        assert_eq!(dash.prompts.len(), 12);
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

        let second_quest_id = dash.quests[1].id; // 15 minutes
        toggle_quest(&conn, second_quest_id).unwrap();

        let dash2 = get_dashboard_state(&conn).unwrap();
        assert_eq!(dash2.streak, 1);
        assert!(dash2.quests[1].done);
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
        assert_eq!(dash.roadmap.len(), 6);
        assert_eq!(dash.quests.len(), 3);
        assert!(dash.total_vocab_count >= 40);

        let exported = export_roadmap_markdown(&conn).unwrap();
        assert!(exported.contains("## Nhiệm vụ hàng ngày"));
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
        assert_eq!(d0.total_vocab_count, 40);
        assert_eq!(d0.error_logs.len(), 10);
        // Mark Tuần 1 done before switching
        toggle_roadmap_item(&conn, d0.roadmap[0].id).unwrap();

        let d80 = set_study_level(&conn, "8.0").unwrap();
        assert_eq!(d80.settings.target_band, 8.0);
        assert_eq!(d80.total_vocab_count, 40);
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

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let due_80 = get_due_vocab(&conn, &today).unwrap();
        assert!(!due_80.is_empty());
        assert!(VOCAB_SEEDS_80.iter().any(|v| v.word == due_80[0].word));

        // Review first C1-C2 card so reps > 0, and add a custom error log
        review_vocab_card(&conn, due_80[0].id, 3, Local::now().date_naive()).unwrap();
        upsert_error_log(&conn, "Custom", "my wrong", "my right").unwrap();

        // Switching to 8.0 again does not duplicate vocab cards
        let d80_again = set_study_level(&conn, "8.0").unwrap();
        assert_eq!(d80_again.total_vocab_count, 40);

        // User enters a real practice score, then switches back to 6.5 -> score, studied card, and custom error preserved!
        add_practice_score(&conn, "Reading", "Cam 16 T1", 8.0, "35/40").unwrap();
        let d65 = set_study_level(&conn, "6.5").unwrap();
        assert_eq!(d65.settings.target_band, 7.0);
        assert_eq!(d65.skill_bands[1].target, 7.0);
        assert_eq!(d65.skill_bands[1].current, 8.0); // User score preserved!
        assert!(d65.roadmap[0].done);
        assert_eq!(d65.total_vocab_count, 41); // 40 Band 6.5 + 1 studied Band 8.0 card
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
                let expected_len = if matches!(wd, Weekday::Thu | Weekday::Sun) { 4 } else { 3 };
                assert_eq!(q.len(), expected_len);
                titles.insert(q[1].0);
            }
            assert_eq!(titles.len(), 7);
        }
    }

    #[test]
    fn generate_vocab_quiz_supports_all_three_modes() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema_and_seed(&conn, None).unwrap();

        let mixed = generate_vocab_quiz(&conn, Some(10), Some("mixed".to_string())).unwrap();
        assert_eq!(mixed.len(), 10);
        let typing_n = mixed.iter().filter(|q| q.question_type == "typing").count();
        let mcq_n = mixed.len() - typing_n;
        assert_eq!(typing_n, 3);
        assert_eq!(mcq_n, 7);

        let mcq = generate_vocab_quiz(&conn, Some(10), Some("mcq".to_string())).unwrap();
        assert_eq!(mcq.len(), 10);
        for q in &mcq {
            assert!(q.question_type == "meaning_mcq" || q.question_type == "cloze_mcq");
            assert_eq!(q.options.len(), 4);
            assert!(q.options.contains(&q.word));
            let uniq: std::collections::HashSet<&String> = q.options.iter().collect();
            assert_eq!(uniq.len(), 4);
        }

        let typing = generate_vocab_quiz(&conn, Some(5), Some("typing".to_string())).unwrap();
        assert_eq!(typing.len(), 5);
        for q in &typing {
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

        let due_list = get_due_vocab(&conn, &today_str).unwrap();
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
}
