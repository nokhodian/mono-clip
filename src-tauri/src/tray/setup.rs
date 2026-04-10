use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager, Runtime,
};

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> anyhow::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Open MonoClip", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit MonoClip", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &separator, &quit_item])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("MonoClip")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "update" => {
                crate::updater::apply_update(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    // Listen for update:available events and rebuild the tray menu
    let handle = app.clone();
    app.listen("update:available", move |event| {
        let tag: String = serde_json::from_str(event.payload())
            .unwrap_or_else(|_| event.payload().trim_matches('"').to_string());

        if let Err(e) = add_update_menu_item(&handle, &tag) {
            log::error!("Failed to update tray menu: {}", e);
        }
    });

    Ok(())
}

/// Rebuilds the tray menu with an "⬆ Update Available" item at the top.
fn add_update_menu_item<R: Runtime>(app: &AppHandle<R>, tag: &str) -> anyhow::Result<()> {
    let tray = app.tray_by_id("").or_else(|| {
        // Tauri 2 uses the first tray if no id was given during build
        app.tray_by_id("tray_0").or_else(|| app.tray_by_id("0"))
    });

    let Some(tray) = tray else {
        return Ok(());
    };

    let label = format!("⬆ Update to {} →", tag);
    let update_item = MenuItem::with_id(app, "update", label, true, None::<&str>)?;
    let separator_top = tauri::menu::PredefinedMenuItem::separator(app)?;
    let show_item = MenuItem::with_id(app, "show", "Open MonoClip", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit MonoClip", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&update_item, &separator_top, &show_item, &separator, &quit_item],
    )?;

    tray.set_menu(Some(menu))?;
    tray.set_tooltip(Some(&format!("MonoClip — {} available", tag)))?;
    Ok(())
}
