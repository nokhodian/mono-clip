use tauri::{AppHandle, State};
use crate::db::{models::Folder, queries};
use crate::state::AppState;
use crate::shortcuts::manager;

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock();
    queries::get_folders(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    icon: String,
    color: String,
    shortcut: Option<String>,
) -> Result<Folder, String> {
    if name.trim().is_empty() {
        return Err("Folder name cannot be empty".into());
    }
    let folder = {
        let conn = state.db.lock();
        queries::create_folder(&conn, &name, &icon, &color, shortcut.as_deref())
            .map_err(|e| e.to_string())?
    };

    // Live-register the shortcut immediately so it works without restart
    if let Some(ref s) = folder.global_shortcut {
        if let Err(e) = manager::register_folder_shortcut(&app, folder.id, folder.name.clone(), s) {
            log::warn!("Could not register shortcut '{}': {}", s, e);
        }
    }

    Ok(folder)
}

#[tauri::command]
pub fn update_folder(
    app: AppHandle,
    state: State<AppState>,
    id: i64,
    name: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    shortcut: Option<String>,
    clear_shortcut: Option<bool>,
) -> Result<Folder, String> {
    if id == 1 && name.is_some() {
        return Err("Cannot rename the Inbox folder".into());
    }

    // Read old shortcut before update so we can unregister it
    let old_shortcut = {
        let conn = state.db.lock();
        queries::get_folder(&conn, id)
            .ok()
            .and_then(|f| f.global_shortcut)
    };

    let shortcut_value: Option<Option<&str>> = if clear_shortcut.unwrap_or(false) {
        Some(None)
    } else {
        shortcut.as_deref().map(Some)
    };

    let folder = {
        let conn = state.db.lock();
        queries::update_folder(
            &conn,
            id,
            name.as_deref(),
            icon.as_deref(),
            color.as_deref(),
            shortcut_value,
        )
        .map_err(|e| e.to_string())?
    };

    // Unregister old shortcut (if it changed or was cleared)
    if let Some(ref old) = old_shortcut {
        if folder.global_shortcut.as_deref() != Some(old.as_str()) {
            let _ = manager::unregister_shortcut(&app, old);
        }
    }

    // Register new shortcut
    if let Some(ref new_s) = folder.global_shortcut {
        if let Err(e) = manager::register_folder_shortcut(&app, folder.id, folder.name.clone(), new_s) {
            log::warn!("Could not register shortcut '{}': {}", new_s, e);
        }
    }

    Ok(folder)
}

#[tauri::command]
pub fn delete_folder(
    app: AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    if id == 1 {
        return Err("Cannot delete the Inbox folder".into());
    }

    // Unregister shortcut before deleting
    let old_shortcut = {
        let conn = state.db.lock();
        queries::get_folder(&conn, id)
            .ok()
            .and_then(|f| f.global_shortcut)
    };
    if let Some(ref s) = old_shortcut {
        let _ = manager::unregister_shortcut(&app, s);
    }

    let conn = state.db.lock();
    queries::delete_folder(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_folders(state: State<AppState>, ids: Vec<i64>) -> Result<(), String> {
    let conn = state.db.lock();
    queries::reorder_folders(&conn, &ids).map_err(|e| e.to_string())
}
