mod ai;
mod db;
mod models;
mod roadmap_parser;
mod seed_data;
mod srs;

use std::sync::Mutex;

use chrono::{Local, Timelike};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_notification::NotificationExt;

use crate::models::{
    Correction, DashboardState, GradeResult, QuizAnswerInput, QuizQuestion, QuizSubmitResult,
    SettingsPublic, VocabCard,
};

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[derive(Serialize)]
pub struct GradeResponseDto {
    pub result: GradeResult,
    pub endpoint: String,
    pub essay_id: i64,
}

#[tauri::command]
fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn toggle_quest(state: State<'_, AppState>, id: i64) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::toggle_quest(&conn, id)?;
    let _ = db::write_widget_status(&conn);
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn add_pomodoro_minutes(
    state: State<'_, AppState>,
    minutes: i64,
) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    db::adjust_study_day(&conn, &today, minutes.max(1), 0)?;
    let _ = db::write_widget_status(&conn);
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn get_due_vocab(state: State<'_, AppState>) -> Result<Vec<VocabCard>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    db::get_due_vocab(&conn, &today)
}

#[tauri::command]
fn review_vocab(
    state: State<'_, AppState>,
    card_id: i64,
    quality: u8,
) -> Result<VocabCard, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();
    let res = db::review_vocab_card(&conn, card_id, quality, today)?;
    let _ = db::write_widget_status(&conn);
    Ok(res)
}

#[tauri::command]
fn generate_vocab_quiz(
    state: State<'_, AppState>,
    count: Option<i64>,
    mode: Option<String>,
) -> Result<Vec<QuizQuestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::generate_vocab_quiz(&conn, count, mode)
}

#[tauri::command]
fn submit_vocab_quiz(
    state: State<'_, AppState>,
    mode: String,
    results: Vec<QuizAnswerInput>,
) -> Result<QuizSubmitResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();
    let res = db::submit_vocab_quiz(&conn, mode, results, today)?;
    let _ = db::write_widget_status(&conn);
    Ok(res)
}

#[tauri::command]
async fn grade_writing(
    state: State<'_, AppState>,
    mode: String,
    prompt: String,
    content: String,
) -> Result<GradeResponseDto, String> {
    let (api_key, model, focus_notes, target_band) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let s = db::get_settings_raw(&conn)?;
        (s.gemini_api_key, s.gemini_model, s.writing_focus_notes, s.target_band)
    };

    let (result, endpoint) = ai::grade_writing_with_gemini(
        &api_key,
        &model,
        &mode,
        &prompt,
        &content,
        &focus_notes,
        target_band,
    )
    .await?;

    let word_count = content.split_whitespace().count() as i64;
    let feedback_json = serde_json::to_string(&result).unwrap_or_default();

    let essay_id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let id = db::save_essay(
            &conn,
            &mode,
            &prompt,
            &content,
            word_count,
            result.ta,
            result.cc,
            result.lr,
            result.gra,
            result.overall,
            &feedback_json,
        )?;
        let _ = db::write_widget_status(&conn);
        id
    };

    Ok(GradeResponseDto {
        result,
        endpoint,
        essay_id,
    })
}

#[tauri::command]
fn save_corrections_to_errors(
    state: State<'_, AppState>,
    corrections: Vec<Correction>,
) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for c in corrections {
        if !c.wrong.trim().is_empty() && !c.right.trim().is_empty() {
            db::upsert_error_log(&conn, &c.category, &c.wrong, &c.right)?;
        }
    }
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn add_error_log(
    state: State<'_, AppState>,
    category: String,
    wrong_text: String,
    right_text: String,
) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::upsert_error_log(&conn, &category, &wrong_text, &right_text)?;
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn add_practice_score(
    state: State<'_, AppState>,
    skill: String,
    source: String,
    score: f64,
    note: String,
) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::add_practice_score(&conn, &skill, &source, score, &note)?;
    let _ = db::write_widget_status(&conn);
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn toggle_roadmap_item(state: State<'_, AppState>, id: i64) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::toggle_roadmap_item(&conn, id)?;
    db::get_dashboard_state(&conn)
}

#[tauri::command]
fn set_study_level(state: State<'_, AppState>, level: String) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let dash = db::set_study_level(&conn, &level)?;
    let _ = db::write_widget_status(&conn);
    Ok(dash)
}

#[tauri::command]
fn update_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme: Option<String>,
    gemini_api_key: Option<String>,
    gemini_model: Option<String>,
    exam_date: Option<String>,
    reminder_time: Option<String>,
    autostart: Option<bool>,
    target_band: Option<f64>,
    onboarding_done: Option<bool>,
) -> Result<SettingsPublic, String> {
    if let Some(enable) = autostart {
        let mgr = app.autolaunch();
        if enable {
            let _ = mgr.enable();
        } else {
            let _ = mgr.disable();
        }
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let res = db::update_settings(
        &conn,
        theme,
        gemini_api_key,
        gemini_model,
        exam_date,
        reminder_time,
        autostart,
        target_band,
        onboarding_done,
    )?;
    let _ = db::write_widget_status(&conn);
    Ok(res)
}

#[tauri::command]
fn clear_gemini_api_key(state: State<'_, AppState>) -> Result<SettingsPublic, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let res = db::clear_gemini_api_key(&conn)?;
    let _ = db::write_widget_status(&conn);
    Ok(res)
}

#[tauri::command]
async fn test_gemini_connection(
    state: State<'_, AppState>,
    api_key: Option<String>,
    model: Option<String>,
) -> Result<ai::ApiTestResult, String> {
    let (saved_key, saved_model) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let s = db::get_settings_raw(&conn)?;
        (s.gemini_api_key, s.gemini_model)
    };
    let effective_key = api_key
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .unwrap_or(saved_key);
    let effective_model = model
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or(saved_model);

    ai::test_gemini_connection_live(&effective_key, &effective_model).await
}

#[tauri::command]
fn get_roadmap_prompt() -> String {
    roadmap_parser::get_roadmap_ai_prompt()
}

#[tauri::command]
fn get_sample_roadmap_md() -> String {
    roadmap_parser::get_sample_roadmap_md()
}

#[tauri::command]
fn preview_roadmap_md(markdown: String) -> Result<roadmap_parser::RoadmapPreview, String> {
    let parsed = roadmap_parser::parse_roadmap_markdown(&markdown)?;
    Ok(roadmap_parser::build_preview(&parsed))
}

#[tauri::command]
fn import_roadmap_md(
    state: State<'_, AppState>,
    markdown: String,
    replace_existing: bool,
    merge_vocab_errors: Option<bool>,
) -> Result<DashboardState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::import_roadmap_to_db(
        &conn,
        &markdown,
        replace_existing,
        merge_vocab_errors.unwrap_or(true),
    )?;
    let _ = db::write_widget_status(&conn);
    db::get_dashboard_state(&conn)
}

#[derive(Serialize)]
pub struct BackupResultDto {
    pub cancelled: bool,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct RestoreResultDto {
    pub cancelled: bool,
    pub dashboard: Option<DashboardState>,
    pub message: String,
}

#[derive(Serialize)]
pub struct WidgetRuntimeInfo {
    pub running: bool,
    pub autostart: bool,
    pub exe_found: bool,
}

#[tauri::command]
fn export_db_backup(state: State<'_, AppState>) -> Result<BackupResultDto, String> {
    let default_name = format!("xd4t-ielts-backup-{}.db", Local::now().format("%Y%m%d"));
    let Some(dest_path) = rfd::FileDialog::new()
        .set_title("Sao lưu dữ liệu XD4T IELTS")
        .set_file_name(&default_name)
        .add_filter("SQLite Database", &["db"])
        .save_file()
    else {
        return Ok(BackupResultDto {
            cancelled: true,
            path: None,
            message: "Đã hủy sao lưu.".to_string(),
        });
    };

    let src_path = db::db_file_path()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoint_and_copy_backup(&conn, &src_path, &dest_path)?;

    let path_str = dest_path.display().to_string();
    Ok(BackupResultDto {
        cancelled: false,
        path: Some(path_str.clone()),
        message: format!("✓ Đã sao lưu thành công tới: {path_str}"),
    })
}

#[tauri::command]
fn import_db_backup(state: State<'_, AppState>) -> Result<RestoreResultDto, String> {
    let Some(src_path) = rfd::FileDialog::new()
        .set_title("Khôi phục dữ liệu XD4T IELTS từ file (.db)")
        .add_filter("SQLite Database", &["db"])
        .pick_file()
    else {
        return Ok(RestoreResultDto {
            cancelled: true,
            dashboard: None,
            message: "Đã hủy khôi phục.".to_string(),
        });
    };

    db::validate_backup_file(&src_path)?;
    let target_db = db::db_file_path()?;

    let mut conn_guard = state.db.lock().map_err(|e| e.to_string())?;
    // Flush and close the existing file-backed connection before replacing the .db file
    let _ = conn_guard.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    let temp_mem = Connection::open_in_memory().map_err(|e| e.to_string())?;
    let old_conn = std::mem::replace(&mut *conn_guard, temp_mem);
    drop(old_conn);

    for suffix in ["-wal", "-shm"] {
        let sidecar = std::path::PathBuf::from(format!("{}{}", target_db.display(), suffix));
        if sidecar.exists() {
            let _ = std::fs::remove_file(&sidecar);
        }
    }

    std::fs::copy(&src_path, &target_db)
        .map_err(|e| format!("Không thể ghi đè file dữ liệu: {e}"))?;

    let new_conn = db::open_and_init_db()?;
    *conn_guard = new_conn;
    let _ = db::write_widget_status(&conn_guard);
    let dash = db::get_dashboard_state(&conn_guard)?;

    Ok(RestoreResultDto {
        cancelled: false,
        dashboard: Some(dash),
        message: format!("✓ Đã khôi phục dữ liệu từ: {}", src_path.display()),
    })
}

fn resolve_widget_exe_path() -> Option<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cur) = std::env::current_exe() {
        if let Some(dir) = cur.parent() {
            candidates.push(dir.join("XD4TIeltsBar.exe"));
            candidates.push(dir.join("resources").join("XD4TIeltsBar.exe"));
            candidates.push(dir.join("..").join("..").join("..").join("taskbar-widget").join("XD4TIeltsBar.exe"));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(
            std::path::PathBuf::from(local)
                .join("XD4T-1ELTS")
                .join("XD4TIeltsBar.exe"),
        );
    }
    candidates.push(std::path::PathBuf::from("taskbar-widget").join("XD4TIeltsBar.exe"));
    candidates.into_iter().find(|p| p.exists())
}

#[cfg(target_os = "windows")]
fn hidden_cmd(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(target_os = "windows"))]
fn hidden_cmd(program: &str) -> std::process::Command {
    std::process::Command::new(program)
}

fn is_widget_running() -> bool {
    let Ok(out) = hidden_cmd("tasklist")
        .args(["/FI", "IMAGENAME eq XD4TIeltsBar.exe", "/NH"])
        .output()
    else {
        return false;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    text.to_ascii_lowercase().contains("xd4tieltsbar.exe")
}

fn is_widget_autostart() -> bool {
    hidden_cmd("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "XD4TIeltsBar",
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
fn get_widget_runtime_info() -> WidgetRuntimeInfo {
    WidgetRuntimeInfo {
        running: is_widget_running(),
        autostart: is_widget_autostart(),
        exe_found: resolve_widget_exe_path().is_some(),
    }
}

#[tauri::command]
fn set_widget_enabled(enabled: bool) -> Result<WidgetRuntimeInfo, String> {
    if enabled {
        let exe_path = resolve_widget_exe_path().ok_or_else(|| {
            "Không tìm thấy XD4TIeltsBar.exe trong thư mục cài đặt.".to_string()
        })?;
        let quoted = format!("\"{}\"", exe_path.display());
        let _ = hidden_cmd("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "XD4TIeltsBar",
                "/t",
                "REG_SZ",
                "/d",
                &quoted,
                "/f",
            ])
            .output();
        if !is_widget_running() {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                std::process::Command::new(&exe_path)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
                    .map_err(|e| format!("Không thể khởi chạy Widget Taskbar: {e}"))?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::process::Command::new(&exe_path)
                    .spawn()
                    .map_err(|e| format!("Không thể khởi chạy Widget Taskbar: {e}"))?;
            }
        }
    } else {
        let _ = hidden_cmd("taskkill")
            .args(["/F", "/IM", "XD4TIeltsBar.exe"])
            .output();
        let _ = hidden_cmd("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "XD4TIeltsBar",
                "/f",
            ])
            .output();
        let _ = hidden_cmd("schtasks")
            .args(["/Delete", "/TN", "XD4T-IeltsBar", "/F"])
            .output();
    }
    Ok(get_widget_runtime_info())
}

#[tauri::command]
fn export_current_roadmap_md(state: State<'_, AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::export_roadmap_markdown(&conn)
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn spawn_reminder_loop(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_notified_date = String::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let now = Local::now();
            let today = now.date_naive().format("%Y-%m-%d").to_string();
            if last_notified_date == today {
                continue;
            }

            let state = app_handle.state::<AppState>();
            let check = {
                let Ok(conn) = state.db.lock() else { continue };
                let Ok(settings) = db::get_settings_raw(&conn) else { continue };
                let done_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM daily_quests WHERE date = ?1 AND done = 1",
                        rusqlite::params![today],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let streak = db::calculate_streak(&conn, now.date_naive()).unwrap_or(0);
                (settings.reminder_time, done_count, streak)
            };

            let (rem_time, done_count, streak) = check;
            if done_count > 0 {
                continue;
            }

            let parts: Vec<&str> = rem_time.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(rh), Ok(rm)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    let cur_mins = now.hour() * 60 + now.minute();
                    let rem_mins = rh * 60 + rm;
                    if cur_mins >= rem_mins {
                        let msg = if streak > 0 {
                            format!("Còn 15p học IELTS để giữ streak {streak} ngày!")
                        } else {
                            "Đến giờ học IELTS 6.5! Hoàn thành 1 nhiệm vụ hôm nay nhé.".to_string()
                        };
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("XD4T // IELTS 6.5")
                            .body(&msg)
                            .show();
                        last_notified_date = today;
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::open_and_init_db().expect("Failed to initialize SQLite database");
    let _ = db::write_widget_status(&conn);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .setup(|app| {
            let show_i = MenuItem::with_id(app, "show", "Mở XD4T IELTS 6.5", true, None::<&str>)?;
            let pomo_i = MenuItem::with_id(app, "pomo15", "Bắt đầu 15p học", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Thoát hẳn", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &pomo_i, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("XD4T // IELTS 6.5")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "pomo15" => {
                        show_main_window(app);
                        let _ = app.emit("start-pomodoro-15", ());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            tray_builder.build(app)?;

            spawn_reminder_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            toggle_quest,
            add_pomodoro_minutes,
            get_due_vocab,
            review_vocab,
            generate_vocab_quiz,
            submit_vocab_quiz,
            grade_writing,
            save_corrections_to_errors,
            add_error_log,
            add_practice_score,
            toggle_roadmap_item,
            set_study_level,
            update_settings,
            clear_gemini_api_key,
            test_gemini_connection,
            get_roadmap_prompt,
            get_sample_roadmap_md,
            preview_roadmap_md,
            import_roadmap_md,
            export_current_roadmap_md,
            export_db_backup,
            import_db_backup,
            get_widget_runtime_info,
            set_widget_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
