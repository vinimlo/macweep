use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let auto_clean = MenuItem::with_id(app, "auto_clean", "Auto-clean Safe", true, None::<&str>)?;
    let full_scan = MenuItem::with_id(app, "full_scan", "Full Scan", true, None::<&str>)?;
    let open_dashboard =
        MenuItem::with_id(app, "open_dashboard", "Open Dashboard", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit macweep", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&auto_clean, &full_scan, &open_dashboard, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open_dashboard" => {
                show_main_window(app);
            }
            "full_scan" => {
                show_main_window(app);
                let _ = app.emit("tray-action", "full_scan");
            }
            "auto_clean" => {
                show_main_window(app);
                let _ = app.emit("tray-action", "auto_clean");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
