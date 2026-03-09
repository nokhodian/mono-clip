use tauri::{AppHandle, Manager};

pub fn position_window_near_cursor(app: &AppHandle) {
    use tauri::PhysicalPosition;

    let Some(window) = app.get_webview_window("main") else { return };

    // Center the window on the primary monitor
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen_size = monitor.size();
        let win_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 680, height: 520 });
        let x = (screen_size.width as i32 - win_size.width as i32) / 2;
        let y = (screen_size.height as i32 - win_size.height as i32) / 3;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}
