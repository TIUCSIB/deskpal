use tauri::{
    menu::{MenuBuilder, MenuEvent, SubmenuBuilder},
    AppHandle, Emitter, LogicalPosition, Manager, WebviewWindow,
};

use crate::{
    settings::{AppSettings, DEFAULT_PET_SCALE, InfoMode, SettingsState},
    sync_chat_shortcut, windowing,
};

pub const CHAT_ID: &str = "chat";
pub const INFO_AUTO_ID: &str = "info-auto";
pub const INFO_ALWAYS_ID: &str = "info-always";
pub const INFO_HIDDEN_ID: &str = "info-hidden";
pub const SIZE_SMALL_ID: &str = "size-small";
pub const SIZE_MEDIUM_ID: &str = "size-medium";
pub const SIZE_LARGE_ID: &str = "size-large";
pub const LOCK_SIZE_ID: &str = "lock-size";
pub const RESET_POSITION_ID: &str = "reset-position";
pub const RESET_SCALE_ID: &str = "reset-scale";
pub const TOGGLE_SHORTCUT_ID: &str = "toggle-shortcut";
pub const SHOW_MAIN_ID: &str = "show-main";
pub const QUIT_ID: &str = "quit";
pub const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";

fn label_with_check(label: &str, checked: bool) -> String {
    if checked {
        format!("✓ {label}")
    } else {
        label.to_string()
    }
}

fn current_settings(app: &AppHandle) -> AppSettings {
    app.try_state::<SettingsState>()
        .and_then(|settings| settings.get().ok())
        .unwrap_or_default()
}

pub fn show_context_menu(
    app: &AppHandle,
    window: &WebviewWindow,
    x: f64,
    y: f64,
    scale: f64,
) -> Result<(), String> {
    if let Some(info) = app.get_webview_window(windowing::INFO_WINDOW) {
        let _ = info.hide();
    }
    let settings = current_settings(app);
    let info_menu = SubmenuBuilder::new(app, "信息窗")
        .text(
            INFO_AUTO_ID,
            label_with_check("自动显示", settings.info_mode == InfoMode::Auto),
        )
        .text(
            INFO_ALWAYS_ID,
            label_with_check("始终显示", settings.info_mode == InfoMode::Always),
        )
        .text(
            INFO_HIDDEN_ID,
            label_with_check("隐藏", settings.info_mode == InfoMode::Hidden),
        )
        .build()
        .map_err(|error| error.to_string())?;
    let size_menu = SubmenuBuilder::new(app, "大小")
        .text(SIZE_SMALL_ID, label_with_check("小", (scale - 0.45).abs() < 0.05))
        .text(SIZE_MEDIUM_ID, label_with_check("中", (scale - 0.85).abs() < 0.05))
        .text(SIZE_LARGE_ID, label_with_check("大", (scale - 1.2).abs() < 0.05))
        .build()
        .map_err(|error| error.to_string())?;
    let menu = MenuBuilder::new(app)
        .text(CHAT_ID, "聊天")
        .item(&info_menu)
        .item(&size_menu)
        .text(
            LOCK_SIZE_ID,
            label_with_check("锁定大小", settings.size_locked),
        )
        .text(RESET_SCALE_ID, "恢复默认大小")
        .text(RESET_POSITION_ID, "重置位置")
        .text(
            TOGGLE_SHORTCUT_ID,
            label_with_check("聊天快捷键", settings.shortcut_enabled),
        )
        .separator()
        .text(QUIT_ID, "退出")
        .build()
        .map_err(|error| error.to_string())?;

    window
        .popup_menu_at(&menu, LogicalPosition::new(x, y))
        .map_err(|error| error.to_string())
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        CHAT_ID => {
            if let Err(error) = windowing::toggle_chat_window(app) {
                eprintln!("无法切换聊天窗口: {error}");
            }
        }
        INFO_AUTO_ID => update_info_mode(app, InfoMode::Auto),
        INFO_ALWAYS_ID => update_info_mode(app, InfoMode::Always),
        INFO_HIDDEN_ID => update_info_mode(app, InfoMode::Hidden),
        SIZE_SMALL_ID => update_scale(app, 0.45, true),
        SIZE_MEDIUM_ID => update_scale(app, 0.85, true),
        SIZE_LARGE_ID => update_scale(app, 1.2, true),
        LOCK_SIZE_ID => toggle_size_lock(app),
        RESET_SCALE_ID => update_scale(app, DEFAULT_PET_SCALE, false),
        RESET_POSITION_ID => reset_position(app),
        TOGGLE_SHORTCUT_ID => toggle_shortcut(app),
        SHOW_MAIN_ID => {
            if let Err(error) = windowing::show_main_window(app) {
                eprintln!("无法显示桌宠窗口: {error}");
            }
        }
        QUIT_ID => app.exit(0),
        _ => {}
    }
}

fn update_info_mode(app: &AppHandle, mode: InfoMode) {
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    match settings.set_info_mode(mode) {
        Ok(updated) => {
            let visible = updated.info_mode == InfoMode::Always;
            if let Err(error) = windowing::set_info_window_visible(app, visible) {
                eprintln!("无法更新信息窗口模式: {error}");
            }
            emit_settings(app, &updated);
        }
        Err(error) => eprintln!("无法保存信息窗口模式: {error}"),
    }
}

fn update_scale(app: &AppHandle, scale: f64, respect_lock: bool) {
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    let current = match settings.get() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("无法读取应用设置: {error}");
            return;
        }
    };
    if respect_lock && current.size_locked {
        return;
    }
    match settings.set_pet_scale(scale) {
        Ok(updated) => {
            emit_scale(app, scale);
            emit_settings(app, &updated);
        }
        Err(error) => eprintln!("无法保存桌宠缩放: {error}"),
    }
}

fn toggle_size_lock(app: &AppHandle) {
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    let next = match settings.get() {
        Ok(current) => !current.size_locked,
        Err(error) => {
            eprintln!("无法读取应用设置: {error}");
            return;
        }
    };
    match settings.set_size_locked(next) {
        Ok(updated) => emit_settings(app, &updated),
        Err(error) => eprintln!("无法更新锁定大小设置: {error}"),
    }
}

fn reset_position(app: &AppHandle) {
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    match settings.reset_main_position() {
        Ok(updated) => {
            if let Err(error) = windowing::reset_main_window_position(app) {
                eprintln!("无法重置桌宠位置: {error}");
            }
            emit_settings(app, &updated);
        }
        Err(error) => eprintln!("无法重置桌宠位置设置: {error}"),
    }
}

fn toggle_shortcut(app: &AppHandle) {
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    let target = match settings.get() {
        Ok(current) => !current.shortcut_enabled,
        Err(error) => {
            eprintln!("无法读取快捷键设置: {error}");
            return;
        }
    };
    let enabled = match sync_chat_shortcut(app, target) {
        Ok(enabled) => enabled,
        Err(error) => {
            eprintln!("无法切换聊天快捷键: {error}");
            return;
        }
    };
    match settings.set_shortcut_enabled(enabled) {
        Ok(updated) => {
            if target && !enabled {
                eprintln!("聊天快捷键 Ctrl+Alt+D 已被占用，已自动关闭该功能");
            }
            emit_settings(app, &updated)
        }
        Err(error) => eprintln!("无法更新快捷键设置: {error}"),
    }
}

fn emit_scale(app: &AppHandle, scale: f64) {
    let Some(main) = app.get_webview_window(windowing::MAIN_WINDOW) else {
        return;
    };
    if let Err(error) = main.emit("pet://set-scale", scale) {
        eprintln!("无法发送桌宠缩放事件: {error}");
    }
}

fn emit_settings(app: &AppHandle, settings: &AppSettings) {
    let Some(main) = app.get_webview_window(windowing::MAIN_WINDOW) else {
        return;
    };
    if let Err(error) = main.emit(SETTINGS_UPDATED_EVENT, settings) {
        eprintln!("无法同步应用设置事件: {error}");
    }
}
