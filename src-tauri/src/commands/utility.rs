use tauri::{AppHandle, Emitter, Manager, State};
use crate::db::queries;
use crate::db::models::AppStats;
use crate::state::AppState;

/// Attempt to install `mclip` into `~/.local/bin/` by symlinking the bundled binary.
/// Safe to call on every launch — skips silently if already installed.
pub fn auto_install_cli(app: &AppHandle) {
    let Ok(exe) = std::env::current_exe() else { return };
    let Some(mac_os_dir) = exe.parent() else { return };
    let mclip_bin = mac_os_dir.join("mclip");
    if !mclip_bin.exists() {
        return; // not bundled yet (dev mode)
    }

    let Ok(home) = std::env::var("HOME") else { return };
    let bin_dir = std::path::PathBuf::from(&home).join(".local").join("bin");
    if std::fs::create_dir_all(&bin_dir).is_err() {
        return;
    }
    let link = bin_dir.join("mclip");
    // Already points to the right binary — nothing to do
    if link.read_link().ok().as_deref() == Some(&mclip_bin) {
        return;
    }
    // Remove stale symlink/file then create a fresh one
    let _ = std::fs::remove_file(&link);
    if std::os::unix::fs::symlink(&mclip_bin, &link).is_ok() {
        log::info!("mclip installed → {:?}", link);
        let _ = app.emit("cli:installed", link.to_string_lossy().into_owned());
    }
}

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
pub fn install_cli(app: AppHandle) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mac_os_dir = exe.parent().ok_or("cannot find binary directory")?;
    let mclip_bin = mac_os_dir.join("mclip");

    if !mclip_bin.exists() {
        return Err(
            "mclip binary not found. This is expected in dev mode — build a release first.".into(),
        );
    }

    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let bin_dir = std::path::PathBuf::from(&home).join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    let link = bin_dir.join("mclip");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&mclip_bin, &link).map_err(|e| e.to_string())?;

    let path = link.to_string_lossy().into_owned();
    let _ = app.emit("cli:installed", &path);
    Ok(path)
}

/// Returns true if the Accessibility permission has been granted to this process.
#[tauri::command]
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    true
}

/// Opens System Settings to the Accessibility privacy pane.
#[tauri::command]
pub fn open_accessibility_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

/// Trigger a full automatic update: download, install, and relaunch.
#[tauri::command]
pub fn do_update(app: AppHandle) -> Result<(), String> {
    crate::updater::apply_update(&app);
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
