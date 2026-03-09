use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use crate::state::AppState;
use crate::db::queries;
use crate::clipboard::detector;

static RUNNING: AtomicBool = AtomicBool::new(false);

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub fn start_watcher(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    std::thread::spawn(move || {
        let mut last_hash: u64 = 0;

        loop {
            std::thread::sleep(Duration::from_millis(500));

            // Read clipboard content
            let content = match read_clipboard_text(&app) {
                Some(c) => c,
                None => continue,
            };

            if content.is_empty() || content.len() > 50_000 {
                continue;
            }

            let current_hash = hash_str(&content);
            if current_hash == last_hash {
                continue;
            }
            last_hash = current_hash;

            // Check if ignored app
            let state = app.state::<AppState>();
            let settings = {
                let conn = state.db.lock();
                match queries::get_settings(&conn) {
                    Ok(s) => s,
                    Err(_) => continue,
                }
            };

            // Save to Inbox
            let clip = {
                let conn = state.db.lock();

                // Deduplication check
                match queries::find_duplicate_in_folder(&conn, &content, 1) {
                    Ok(Some(existing_id)) => {
                        let _ = queries::touch_clip(&conn, existing_id);
                        match queries::get_clip(&conn, existing_id) {
                            Ok(c) => c,
                            Err(_) => continue,
                        }
                    }
                    _ => {
                        let content_type = detector::detect_content_type(&content);
                        let preview = detector::make_preview(&content, 200);
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
            log::debug!("New clip saved: id={}", clip.id);
        }
    });
}

fn read_clipboard_text(app: &AppHandle) -> Option<String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().read_text().ok()
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
