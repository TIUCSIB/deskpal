use std::fs;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuEvent},
    path::BaseDirectory,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::windowing;

pub const SHOW_MAIN_ID: &str = "show-main";
pub const OPEN_SETTINGS_ID: &str = "open-settings";
pub const QUIT_ID: &str = "quit";
const TRAY_ICON_RESOURCE: &str = "icons/tray.ico";

fn load_tray_icon(app: &AppHandle) -> Option<Image<'static>> {
    let path = match app.path().resolve(TRAY_ICON_RESOURCE, BaseDirectory::Resource) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("无法解析托盘图标资源路径: {error}");
            return None;
        }
    };
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("无法读取托盘图标资源 {}: {error}", path.display());
            return None;
        }
    };
    match Image::from_bytes(&bytes) {
        Ok(icon) => Some(icon),
        Err(error) => {
            eprintln!("无法解析托盘图标资源 {}: {error}", path.display());
            None
        }
    }
}

pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let tray_menu = MenuBuilder::new(app)
        .text(SHOW_MAIN_ID, "显示桌宠")
        .text(OPEN_SETTINGS_ID, "设置")
        .separator()
        .text(QUIT_ID, "退出")
        .build()
        .map_err(|error| error.to_string())?;

    let mut builder = TrayIconBuilder::with_id("deskpal-tray")
        .menu(&tray_menu)
        .tooltip("DeskPal")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = windowing::show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = load_tray_icon(app).or_else(|| app.default_window_icon().cloned()) {
        builder = builder.icon(icon);
    }

    builder.build(app).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        SHOW_MAIN_ID => {
            if let Err(error) = windowing::show_main_window(app) {
                eprintln!("无法显示桌宠窗口: {error}");
            }
        }
        OPEN_SETTINGS_ID => {
            if let Err(error) = windowing::show_settings_window(app) {
                eprintln!("无法打开设置窗口: {error}");
            }
        }
        QUIT_ID => app.exit(0),
        _ => {}
    }
}
