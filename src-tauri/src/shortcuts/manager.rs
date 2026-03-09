use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_clipboard_manager::ClipboardExt;
use crate::state::AppState;
use crate::db::queries;
use crate::clipboard::detector;
use std::str::FromStr;

/// Capture the currently selected (highlighted) text in the frontmost app.
///
/// Strategy:
///   1. Save the current clipboard contents
///   2. Simulate Cmd+C to copy the selection into the clipboard
///   3. Read the new clipboard value
///   4. Restore the original clipboard so the user doesn't notice
///
/// Returns `None` if nothing was selected or the clipboard didn't change.
fn capture_selected_text(app: &AppHandle) -> Option<String> {
    // 1. Save current clipboard
    let original = app.clipboard().read_text().ok().unwrap_or_default();

    // 2. Simulate Cmd+C in the frontmost app
    let status = std::process::Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to keystroke \"c\" using command down"])
        .status();

    if status.is_err() {
        return None;
    }

    // 3. Brief pause for clipboard to settle
    std::thread::sleep(std::time::Duration::from_millis(120));

    // 4. Read the (potentially updated) clipboard
    let selected = app.clipboard().read_text().ok().unwrap_or_default();

    // 5. Restore original clipboard contents
    if !original.is_empty() && selected != original {
        let _ = app.clipboard().write_text(original.clone());
    }

    // Return only if content actually changed (meaning text was selected)
    if selected.is_empty() || selected == original {
        None
    } else {
        Some(selected)
    }
}

/// Register a single folder shortcut. Called when a folder is created or its shortcut updated.
pub fn register_folder_shortcut(
    app: &AppHandle,
    folder_id: i64,
    folder_name: String,
    shortcut_str: &str,
) -> anyhow::Result<()> {
    register_shortcut(app, shortcut_str, ShortcutAction::SaveToFolder { folder_id, folder_name })
}

pub fn register_all_shortcuts(app: &AppHandle) -> anyhow::Result<()> {
    let state = app.state::<AppState>();
    let (master_shortcut, folder_shortcuts) = {
        let conn = state.db.lock();
        let settings = queries::get_settings(&conn)?;
        let folders = queries::get_folders(&conn)?;
        let folder_shortcuts: Vec<(i64, String, String)> = folders
            .into_iter()
            .filter_map(|f| f.global_shortcut.map(|s| (f.id, f.name, s)))
            .collect();
        (settings.master_shortcut, folder_shortcuts)
    };

    // Register master shortcut (toggle window)
    register_shortcut(app, &master_shortcut, ShortcutAction::ToggleWindow)?;

    // Register folder shortcuts
    for (folder_id, folder_name, shortcut_str) in folder_shortcuts {
        register_shortcut(app, &shortcut_str, ShortcutAction::SaveToFolder { folder_id, folder_name })?;
    }

    Ok(())
}

enum ShortcutAction {
    ToggleWindow,
    SaveToFolder { folder_id: i64, folder_name: String },
}

fn register_shortcut(app: &AppHandle, shortcut_str: &str, action: ShortcutAction) -> anyhow::Result<()> {
    let shortcut = match Shortcut::from_str(shortcut_str) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Invalid shortcut '{}': {:?}", shortcut_str, e);
            return Ok(());
        }
    };

    let app_clone = app.clone();
    match action {
        ShortcutAction::ToggleWindow => {
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    crate::window::manager::position_window_near_cursor(&app_clone);
                    if let Some(window) = app_clone.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })?;
        }
        ShortcutAction::SaveToFolder { folder_id, folder_name } => {
            app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    // Prefer selected text; fall back to current clipboard
                    let (content, source) = if let Some(sel) = capture_selected_text(app) {
                        (sel, "selection")
                    } else {
                        match app.clipboard().read_text() {
                            Ok(c) if !c.is_empty() => (c, "clipboard"),
                            _ => return,
                        }
                    };

                    let state = app.state::<AppState>();
                    let conn = state.db.lock();
                    let content_type = detector::detect_content_type(&content);
                    let preview = detector::make_preview(&content, 200);
                    match queries::insert_clip(&conn, &content, content_type, &preview, folder_id, None) {
                        Ok(clip) => {
                            let _ = app.emit("folder:saved", serde_json::json!({
                                "clip": clip,
                                "folderName": folder_name,
                                "source": source,
                            }));
                        }
                        Err(e) => log::error!("Failed to save clip to folder {}: {}", folder_id, e),
                    }
                }
            })?;
        }
    }

    Ok(())
}

pub fn unregister_shortcut(app: &AppHandle, shortcut_str: &str) -> anyhow::Result<()> {
    if let Ok(shortcut) = Shortcut::from_str(shortcut_str) {
        let _ = app.global_shortcut().unregister(shortcut);
    }
    Ok(())
}
