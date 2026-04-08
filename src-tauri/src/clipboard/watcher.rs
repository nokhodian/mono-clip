use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use crate::state::AppState;
use crate::db::queries;
use crate::clipboard::{detector, image_store};

static RUNNING: AtomicBool = AtomicBool::new(false);

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub fn start_watcher(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        // Outer loop: restart the watcher if a panic occurs inside
        loop {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_watcher_loop(&app);
            }));
            if let Err(e) = result {
                log::error!("Clipboard watcher panicked: {:?} — restarting in 1s", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

fn run_watcher_loop(app: &AppHandle) {
    let mut last_hash: u64 = 0;

    loop {
        std::thread::sleep(Duration::from_millis(300));

        let entry = read_clipboard_entry(app);
        let (content, content_type, preview) = match entry {
            Some(e) => e,
            None => continue,
        };

        // Skip sentinel strings written by the folder-shortcut capture
        if content.starts_with("__monoclip_sentinel_") {
            continue;
        }

        let current_hash = hash_str(&content);
        if current_hash == last_hash {
            continue;
        }
        last_hash = current_hash;

        let state = app.state::<AppState>();

        let settings = {
            let conn = state.db.lock();
            match queries::get_settings(&conn) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to read settings: {}", e);
                    continue;
                }
            }
        };

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

        {
            let conn = state.db.lock();
            let _ = enforce_max_history(&conn, settings.max_history_items);
        }

        let _ = app.emit("clip:new", &clip);
        log::debug!("New clip saved: id={}, type={}", clip.id, clip.content_type);
    }
}

fn read_clipboard_entry(app: &AppHandle) -> Option<(String, &'static str, String)> {
    // 1. File list — checked in a catch_unwind so arboard panics can't kill the watcher
    let file_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(read_clipboard_files));
    if let Ok(Some((content, preview))) = file_result {
        let content_type = detector::detect_content_type(&content);
        return Some((content, content_type, preview));
    }

    // 2. Image
    if let Some((path, preview)) = read_clipboard_image(app) {
        return Some((path, "image", preview));
    }

    // 3. Plain text
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

// Fresh arboard instance per call — avoids stale cached state from a long-lived instance.
// Wrapped in catch_unwind at the call site so any arboard panic is contained.
fn read_clipboard_files() -> Option<(String, String)> {
    let mut cb = arboard::Clipboard::new().ok()?;
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
