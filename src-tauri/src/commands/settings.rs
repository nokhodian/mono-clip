use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use crate::db::{models::{Settings, SettingsPatch}, queries};
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let conn = state.db.lock();
    queries::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    patch: SettingsPatch,
) -> Result<Settings, String> {
    // Sync launch-at-login with the autostart plugin whenever the flag changes
    if let Some(enable) = patch.launch_at_login {
        let autostart = app.autolaunch();
        let _ = if enable { autostart.enable() } else { autostart.disable() };
    }

    let conn = state.db.lock();
    queries::update_settings(&conn, &patch).map_err(|e| e.to_string())
}
