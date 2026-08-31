//! Menu bar (tray) icon: shows the app icon, click opens a menu with
//! Enable-Disable / Quit.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Runtime};

const TRAY_ID: &str = "lweak-tray";
const ITEM_TOGGLE: &str = "toggle_enable";
const ITEM_QUIT: &str = "quit";

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, ITEM_TOGGLE, "Disable App", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ITEM_QUIT, "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().expect("app icon missing"))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            ITEM_TOGGLE => {
                let enabled = !crate::features::is_enabled();
                crate::features::set_enabled(enabled);
                let label = if enabled { "Disable App" } else { "Enable App" };
                let _ = toggle.set_text(label);
            }
            ITEM_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}