mod ai;
mod db;
mod models;
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
    Correction, DashboardState, GradeResult, SettingsPublic, VocabCard,
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
async fn grade_writing(
    state: State<'_, AppState>,
    mode: String,
    prompt: String,
    content: String,
) -> Result<GradeResponseDto, String> {
    let (api_key, model) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let s = db::get_settings_raw(&conn)?;
        (s.gemini_api_key, s.gemini_model)
    };

    let (result, endpoint) =
        ai::grade_writing_with_gemini(&api_key, &model, &mode, &prompt, &content).await?;

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
    )?;
    let _ = db::write_widget_status(&conn);
    Ok(res)
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
            grade_writing,
            save_corrections_to_errors,
            add_error_log,
            add_practice_score,
            toggle_roadmap_item,
            update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
