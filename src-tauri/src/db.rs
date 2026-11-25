use chrono::{Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{
    DailyQuest, DashboardState, ErrorLog, Essay, PracticeScore, RoadmapItem, Settings,
    SettingsPublic, SkillBand, StudyDay, VocabCard,
};
use crate::seed_data::{default_writing_prompts, ROADMAP_SEEDS, VOCAB_SEEDS};
use crate::srs::apply_sm2;

pub fn data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA environment variable not set".to_string())?;
    let dir = base.join("xd4t-ielts65");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {e}"))?;
    Ok(dir)
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
            theme TEXT NOT NULL DEFAULT 'dark',
            gemini_api_key TEXT NOT NULL DEFAULT '',
            gemini_model TEXT NOT NULL DEFAULT 'gemini-3-flash-preview',
            exam_date TEXT NOT NULL DEFAULT '2027-05-15',
            reminder_time TEXT NOT NULL DEFAULT '21:00',
            autostart INTEGER NOT NULL DEFAULT 0,
            target_band REAL NOT NULL DEFAULT 7.0
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
        "#,
    )
    .map_err(|e| format!("Schema init error: {e}"))?;

    conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
        .map_err(|e| format!("Settings seed error: {e}"))?;

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
        for item in ROADMAP_SEEDS {
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
        let samples = [
            ("Nhóm 1 · Chia động từ", "Everybody know", "Everybody knows", 3),
            ("Nhóm 1 · Danh từ đếm được", "human have / Student need", "Humans/People have / Students need", 3),
            ("Nhóm 1 · Mạo từ a/an", "an new adult / an talent artist", "a new adult / a talented artist", 2),
            ("Nhóm 2 · Nhầm từ", "a good change to develop", "a good chance to develop", 2),
            ("Nhóm 2 · Effect/Affect", "can effect our lives", "can affect our lives", 2),
            ("Nhóm 2 · Từ loại", "harmfully effects / In additionally", "harmful effects / In addition", 2),
            ("Nhóm 3 · Comma splice", "...university students, they are...", "...students, as they are... (hoặc tách câu)", 2),
            ("Nhóm 3 · Câu cụt", "Firstly, positive point about students can learn...", "One advantage of letting students study what they love is that...", 2),
            ("Reading · T/F/NG", "Bỏ qua từ tuyệt đối every/all/only/most", "Soi kỹ từ tuyệt đối & phân biệt ý kiến vs sự thật", 2),
            ("Reading · Paraphrase", "Nhầm were delivering ↔ introduce", "Soi nghĩa đối lập thật (within a few months ↔ several years)", 2),
        ];
        for (cat, wrong, right, cnt) in samples {
            conn.execute(
                "INSERT INTO error_logs (category, wrong_text, right_text, count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![cat, wrong, right, cnt, today],
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

pub fn ensure_today_quests(conn: &Connection, today: &str) -> Result<(), String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND track = 'ielts'",
            params![today],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count == 0 {
        let defaults = [
            (
                "Ôn 10 thẻ Collocation (SRS)",
                "SRS Flashcards · giữ chuỗi mỗi ngày",
                8_i64,
            ),
            (
                "Viết 1 đoạn Body Paragraph (~90 từ) & chấm AI",
                "Chủ đề tủ IELTS · luyện câu chuẩn 6.5",
                15_i64,
            ),
            (
                "Làm 1 Passage Reading trên Study4 & nhập điểm",
                "Cambridge Practice · bấm giờ 15–20 phút",
                12_i64,
            ),
        ];
        for (title, meta, mins) in defaults {
            conn.execute(
                "INSERT INTO daily_quests (date, track, title, meta, minutes, done)
                 VALUES (?1, 'ielts', ?2, ?3, ?4, 0)",
                params![today, title, meta, mins],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn get_settings_raw(conn: &Connection) -> Result<Settings, String> {
    conn.query_row(
        "SELECT theme, gemini_api_key, gemini_model, exam_date, reminder_time, autostart, target_band
         FROM settings WHERE id = 1",
        [],
        |r| {
            Ok(Settings {
                theme: r.get(0)?,
                gemini_api_key: r.get(1)?,
                gemini_model: r.get(2)?,
                exam_date: r.get(3)?,
                reminder_time: r.get(4)?,
                autostart: r.get::<_, i64>(5)? != 0,
                target_band: r.get(6)?,
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
    })
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
) -> Result<SettingsPublic, String> {
    let cur = get_settings_raw(conn)?;
    let new_theme = theme.unwrap_or(cur.theme);
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

    conn.execute(
        "UPDATE settings
         SET theme = ?1, gemini_api_key = ?2, gemini_model = ?3, exam_date = ?4,
             reminder_time = ?5, autostart = ?6, target_band = ?7
         WHERE id = 1",
        params![
            new_theme,
            new_key,
            new_model,
            new_exam,
            new_rem,
            if new_auto { 1 } else { 0 },
            new_target
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
    let mut stmt = conn
        .prepare(
            "SELECT id, track, topic, word, meaning, example, ease, interval_days, reps, due_date
             FROM vocab_cards
             WHERE due_date <= ?1
             ORDER BY due_date ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
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

    let targets = [
        ("Listening", 7.5),
        ("Reading", 7.0),
        ("Speaking", 6.5),
        ("Writing", 6.0),
    ];
    let mut skill_bands = Vec::new();
    for (skill, target) in targets {
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
        roadmap,
        error_logs,
        recent_essays,
        practice_scores,
        prompts: default_writing_prompts(),
    })
}

#[derive(serde::Serialize)]
pub struct WidgetStatus {
    pub quests_done: i64,
    pub quests_total: i64,
    pub streak: i64,
    pub overall_band: f64,
    pub target_band: f64,
    pub days_left: i64,
    pub today_minutes: i64,
    pub updated_at: String,
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
}
