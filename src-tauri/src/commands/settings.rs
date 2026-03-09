use tauri::State;
use crate::db::{models::{Settings, SettingsPatch}, queries};
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let conn = state.db.lock();
    queries::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    patch: SettingsPatch,
) -> Result<Settings, String> {
    let conn = state.db.lock();
    queries::update_settings(&conn, &patch).map_err(|e| e.to_string())
}
