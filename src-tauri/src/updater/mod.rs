use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_API: &str =
    "https://api.github.com/repos/nokhodian/mono-clip/releases/latest";
const CHECK_INTERVAL_SECS: u64 = 3600; // 1 hour

/// Returns Some(tag) if the latest GitHub release tag is newer than the running version.
pub fn check_for_update() -> Option<String> {
    let response = ureq::get(RELEASES_API)
        .set("User-Agent", "MonoClip-Updater")
        .call()
        .ok()?;

    let json: serde_json::Value = response.into_json().ok()?;
    let tag = json["tag_name"].as_str()?.to_string();

    // Strip leading 'v' for comparison
    let remote = tag.trim_start_matches('v');
    let local = CURRENT_VERSION.trim_start_matches('v');

    if is_newer(remote, local) {
        Some(tag)
    } else {
        None
    }
}

/// Returns true if `remote` semver is strictly greater than `local`.
fn is_newer(remote: &str, local: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let mut parts = s.splitn(3, '.');
        let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };
    parse(remote) > parse(local)
}

/// Spawns a background thread that checks for updates every hour.
/// On update found, emits `"update:available"` with the new version tag.
pub fn start_update_checker<R: Runtime>(app: AppHandle<R>, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        // Check immediately on startup (after a brief delay)
        std::thread::sleep(Duration::from_secs(10));

        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }

            if let Some(tag) = check_for_update() {
                log::info!("Update available: {}", tag);
                let _ = app.emit("update:available", tag);
            }

            // Wait for next check interval, checking stop flag periodically
            let mut elapsed = 0u64;
            while elapsed < CHECK_INTERVAL_SECS {
                std::thread::sleep(Duration::from_secs(60));
                elapsed += 60;
                if stop.load(Ordering::Relaxed) {
                    return;
                }
            }
        }
    });
}
