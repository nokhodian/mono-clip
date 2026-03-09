use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use crate::db::{models::ClipItem, queries};
use crate::state::AppState;
use crate::clipboard::detector;

#[tauri::command]
pub fn get_clips(
    state: State<AppState>,
    folder_id: Option<i64>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipItem>, String> {
    let conn = state.db.lock();
    queries::get_clips(
        &conn,
        folder_id,
        search.as_deref(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_clip(state: State<AppState>, id: i64) -> Result<ClipItem, String> {
    let conn = state.db.lock();
    queries::get_clip(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pin_clip(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::set_clip_pinned(&conn, id, true).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unpin_clip(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::set_clip_pinned(&conn, id, false).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_clip(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::soft_delete_clip(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_clip(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::restore_clip(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hard_delete_clip(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::hard_delete_clip(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_clip(state: State<AppState>, id: i64, folder_id: i64) -> Result<(), String> {
    let conn = state.db.lock();
    queries::move_clip(&conn, id, folder_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_to_clipboard(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let (content, paste_on_click) = {
        let conn = state.db.lock();
        let clip = queries::get_clip(&conn, id).map_err(|e| e.to_string())?;
        let settings = queries::get_settings(&conn).map_err(|e| e.to_string())?;
        (clip.content, settings.paste_on_click)
    };

    app.clipboard().write_text(content).map_err(|e| e.to_string())?;

    if paste_on_click {
        // Simulate Cmd+V after a short delay to allow clipboard to settle
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            tauri::async_runtime::spawn_blocking(|| {
                std::thread::sleep(std::time::Duration::from_millis(150));
            })
            .await
            .ok();
            let _ = simulate_paste(&app_clone);
        });
    }

    Ok(())
}

fn simulate_paste(_app: &AppHandle) -> anyhow::Result<()> {
    // On macOS, use CGEvent to simulate Cmd+V
    // This requires accessibility permissions. We use a simpler approach via AppleScript
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to keystroke \"v\" using command down")
            .spawn()?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_current_clipboard_to_folder(
    app: AppHandle,
    state: State<AppState>,
    folder_id: i64,
) -> Result<ClipItem, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let content = app.clipboard()
        .read_text()
        .map_err(|e| e.to_string())?;

    if content.is_empty() {
        return Err("Clipboard is empty".into());
    }

    let conn = state.db.lock();

    // Deduplication
    if let Ok(Some(existing_id)) = queries::find_duplicate_in_folder(&conn, &content, folder_id) {
        let _ = queries::touch_clip(&conn, existing_id);
        return queries::get_clip(&conn, existing_id).map_err(|e| e.to_string());
    }

    let content_type = detector::detect_content_type(&content);
    let preview = detector::make_preview(&content, 200);
    queries::insert_clip(&conn, &content, content_type, &preview, folder_id, None)
        .map_err(|e| e.to_string())
}
