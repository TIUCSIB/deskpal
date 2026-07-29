use std::fs;

use tauri::{
    image::Image,
    menu::{CheckMenuItem, MenuBuilder, MenuEvent},
    path::BaseDirectory,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::{commands::settings, windowing};

const TRAY_ID: &str = "deskpal-tray";
pub const SHOW_MAIN_ID: &str = "show-main";
pub const OPEN_SETTINGS_ID: &str = "open-settings";
pub const TOGGLE_PASSTHROUGH_ID: &str = "toggle-passthrough";
pub const QUIT_ID: &str = "quit";
const TRAY_ICON_RESOURCE: &str = "icons/tray.ico";

fn load_tray_icon(app: &AppHandle) -> Option<Image<'static>> {
    let path = match app
        .path()
        .resolve(TRAY_ICON_RESOURCE, BaseDirectory::Resource)
    {
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

fn passthrough_label(enabled: bool) -> &'static str {
    if enabled {
        "开启透传"
    } else {
        "关闭透传"
    }
}

fn build_tray_menu(app: &AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, String> {
    let settings = app
        .try_state::<crate::settings::SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    let passthrough = CheckMenuItem::with_id(
        app,
        TOGGLE_PASSTHROUGH_ID,
        passthrough_label(settings.main_window_left_click_passthrough),
        true,
        settings.main_window_left_click_passthrough,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    MenuBuilder::new(app)
        .text(SHOW_MAIN_ID, "显示桌宠")
        .text(OPEN_SETTINGS_ID, "设置")
        .item(&passthrough)
        .separator()
        .text(QUIT_ID, "退出")
        .build()
        .map_err(|error| error.to_string())
}

pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let tray_menu = build_tray_menu(app)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
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

pub fn sync_tray(app: &AppHandle) -> Result<(), String> {
    let _ = app.remove_tray_by_id(TRAY_ID);
    create_tray(app)
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
        TOGGLE_PASSTHROUGH_ID => {
            let result: Result<(), String> = (|| {
                let settings_state = app
                    .try_state::<crate::settings::SettingsState>()
                    .ok_or_else(|| "找不到应用设置状态".to_string())?;
                let current = settings_state.get()?;
                let updated = settings_state.set_main_window_left_click_passthrough(
                    !current.main_window_left_click_passthrough,
                )?;
                windowing::apply_main_window_settings(app, &updated)?;
                settings::finish_update(app, updated)?;
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("无法切换左键透传: {error}");
                let _ = sync_tray(app);
            }
        }
        QUIT_ID => {
            if let Some(settings) = app.try_state::<crate::settings::SettingsState>() {
                let _ = settings.flush_pending_geometry();
            }
            app.exit(0)
        }
        _ => {}
    }
}
