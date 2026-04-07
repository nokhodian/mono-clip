use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use crate::state::AppState;
use crate::db::queries;
use crate::clipboard::{detector, image_store};

static RUNNING: AtomicBool = AtomicBool::new(false);

/// Ask macOS for the current NSPasteboard changeCount via FFI.
/// This increments every time any app writes to the clipboard — much
/// cheaper and more reliable than reading the full clipboard content
/// on every poll cycle.
#[cfg(target_os = "macos")]
fn pasteboard_change_count() -> i64 {
    use std::ffi::c_void;

    #[link(name = "AppKit", kind = "framework")]
    extern "C" {
        fn objc_getClass(name: *const i8) -> *mut c_void;
        fn sel_registerName(name: *const i8) -> *mut c_void;
        fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }

    unsafe {
        let cls_name = b"NSPasteboard\0";
        let sel_general = b"generalPasteboard\0";
        let sel_change_count = b"changeCount\0";

        let cls = objc_getClass(cls_name.as_ptr() as *const i8);
        let sel_gp = sel_registerName(sel_general.as_ptr() as *const i8);
        let sel_cc = sel_registerName(sel_change_count.as_ptr() as *const i8);

        let pb = objc_msgSend(cls, sel_gp);
        let count = objc_msgSend(pb, sel_cc) as i64;
        count
    }
}

#[cfg(not(target_os = "macos"))]
fn pasteboard_change_count() -> i64 {
    // On non-macOS fall back to always returning a changing value
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn start_watcher(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    std::thread::spawn(move || {
        let mut last_change_count: i64 = pasteboard_change_count();
        // Keep one arboard instance to avoid repeatedly opening/closing
        // the NSPasteboard connection which can interfere with clipboard reads.
        let mut arboard_cb = arboard::Clipboard::new().ok();

        loop {
            std::thread::sleep(Duration::from_millis(250));

            // Fast path: skip if NSPasteboard hasn't changed
            let change_count = pasteboard_change_count();
            if change_count == last_change_count {
                continue;
            }

            // Clipboard changed — read it
            let entry = read_clipboard_entry(&app, &mut arboard_cb);
            let (content, content_type, preview) = match entry {
                Some(e) => e,
                None => {
                    // Acknowledge the change even if we can't read it,
                    // so we don't spin on it forever.
                    last_change_count = change_count;
                    continue;
                }
            };

            // Filter out sentinel strings written by the folder-shortcut capture
            if content.starts_with("__monoclip_sentinel_") {
                last_change_count = change_count;
                continue;
            }

            // Update the change count before the (potentially slow) DB write
            last_change_count = change_count;

            // Load settings (for max_history enforcement)
            let state = app.state::<AppState>();
            let settings = {
                let conn = state.db.lock();
                match queries::get_settings(&conn) {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("Failed to read settings, will retry: {}", e);
                        continue;
                    }
                }
            };

            // Save to Inbox with deduplication
            let clip = {
                let conn = state.db.lock();
                match queries::find_duplicate_in_folder(&conn, &content, 1) {
                    Ok(Some(existing_id)) => {
                        let _ = queries::touch_clip(&conn, existing_id);
                        match queries::get_clip(&conn, existing_id) {
                            Ok(c) => c,
                            Err(e) => {
                                log::warn!("Failed to fetch existing clip: {}", e);
                                continue;
                            }
                        }
                    }
                    _ => {
                        match queries::insert_clip(&conn, &content, content_type, &preview, 1, None) {
                            Ok(c) => c,
                            Err(e) => {
                                log::error!("Failed to save clip: {}", e);
                                continue;
                            }
                        }
                    }
                }
            };

            // Enforce max history limit
            {
                let conn = state.db.lock();
                let _ = enforce_max_history(&conn, settings.max_history_items);
            }

            // Emit event to frontend
            let _ = app.emit("clip:new", &clip);
            log::debug!("New clip saved: id={}, type={}", clip.id, clip.content_type);
        }
    });
}

/// Try file → image → text in priority order.
fn read_clipboard_entry(
    app: &AppHandle,
    arboard_cb: &mut Option<arboard::Clipboard>,
) -> Option<(String, &'static str, String)> {
    // 1. Try file list first — full paths take priority over bare filename
    if let Some((content, preview)) = read_clipboard_files(arboard_cb) {
        let content_type = detector::detect_content_type(&content);
        return Some((content, content_type, preview));
    }

    // 2. Try image
    if let Some((path, preview)) = read_clipboard_image(app) {
        return Some((path, "image", preview));
    }

    // 3. Fall back to plain text
    if let Some(text) = read_clipboard_text(app) {
        if !text.is_empty() && text.len() <= 50_000 {
            let content_type = detector::detect_content_type(&text);
            let preview = detector::make_preview(&text, 200);
            return Some((text, content_type, preview));
        }
    }

    None
}

fn read_clipboard_text(app: &AppHandle) -> Option<String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().read_text().ok()
}

fn read_clipboard_image(app: &AppHandle) -> Option<(String, String)> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let img = app.clipboard().read_image().ok()?;
    let w = img.width();
    let h = img.height();
    if w < 4 || h < 4 {
        return None;
    }
    match image_store::save_as_png(img.rgba(), w, h) {
        Ok(path) => Some((path, format!("Image ({}×{})", w, h))),
        Err(e) => {
            log::warn!("Failed to save clipboard image: {}", e);
            None
        }
    }
}

/// Read file paths using a reused arboard instance to avoid repeated
/// NSPasteboard open/close cycles that can interfere with other reads.
fn read_clipboard_files(cb: &mut Option<arboard::Clipboard>) -> Option<(String, String)> {
    // Reinitialise if the instance was lost
    if cb.is_none() {
        *cb = arboard::Clipboard::new().ok();
    }
    let cb = cb.as_mut()?;

    let paths = cb.get().file_list().ok()?;
    if paths.is_empty() {
        return None;
    }
    let lines: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let preview = if lines.len() == 1 {
        paths[0]
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| lines[0].clone())
    } else {
        format!(
            "{} and {} more",
            paths[0]
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            lines.len() - 1
        )
    };
    Some((lines.join("\n"), preview))
}

fn enforce_max_history(conn: &rusqlite::Connection, max: i64) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM clip_items
         WHERE is_pinned = 0 AND is_deleted = 0
           AND id NOT IN (
             SELECT id FROM clip_items WHERE is_pinned = 0 AND is_deleted = 0
             ORDER BY updated_at DESC LIMIT ?1
           )",
        rusqlite::params![max],
    )?;
    Ok(())
}
