use tauri::{AppHandle, Manager, State};
use crate::db::queries;
use crate::db::models::AppStats;
use crate::state::AppState;

#[tauri::command]
pub fn get_stats(state: State<AppState>) -> Result<AppStats, String> {
    let conn = state.db.lock();
    queries::get_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn run_auto_cleanup(state: State<AppState>) -> Result<i64, String> {
    let conn = state.db.lock();
    let settings = queries::get_settings(&conn).map_err(|e| e.to_string())?;
    if !settings.auto_clean_enabled {
        return Ok(0);
    }
    queries::run_auto_cleanup(&conn, settings.auto_clean_days).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
