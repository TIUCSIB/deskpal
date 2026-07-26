use tauri::{AppHandle, Emitter, State};

use crate::{
    parse_chat_shortcut,
    settings::{AppSettings, InfoMode, SavedPosition, SavedWindowBounds, SettingsState},
    sync_autostart, sync_chat_shortcut, windowing,
};

const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";

fn emit_settings(app: &AppHandle, settings: &AppSettings) {
    if let Err(error) = app.emit(SETTINGS_UPDATED_EVENT, settings) {
        eprintln!("无法同步应用设置事件: {error}");
    }
}

#[tauri::command]
pub fn load_app_settings(settings: State<'_, SettingsState>) -> Result<AppSettings, String> {
    settings.get()
}

#[tauri::command]
pub fn save_pet_scale(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    scale: f64,
) -> Result<AppSettings, String> {
    let updated = settings.set_pet_scale(scale)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn save_main_window_position(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    x: i32,
    y: i32,
) -> Result<AppSettings, String> {
    let updated = settings.save_main_position_throttled(SavedPosition { x, y })?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_info_mode(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    mode: InfoMode,
) -> Result<AppSettings, String> {
    let updated = settings.set_info_mode(mode)?;
    windowing::sync_info_window_visibility(&app)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_size_locked(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    locked: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_size_locked(locked)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_shortcut_enabled(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    let current = settings.get()?;
    let mut updated = settings.set_shortcut_enabled(enabled)?;
    let applied = sync_chat_shortcut(&app, &updated.chat_shortcut, enabled)?;
    if applied != enabled {
        updated = settings.set_shortcut_enabled(applied)?;
    }
    if !applied && current.shortcut_enabled != updated.shortcut_enabled {
        eprintln!("聊天快捷键未能按请求状态应用");
    }
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_chat_shortcut(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    shortcut: String,
) -> Result<AppSettings, String> {
    let trimmed = shortcut.trim();
    if trimmed.is_empty() {
        return Err("请输入快捷键".to_string());
    }
    parse_chat_shortcut(trimmed)?;

    let current = settings.get()?;
    let mut updated = settings.set_chat_shortcut(trimmed.to_string())?;
    if updated.shortcut_enabled {
        let applied = sync_chat_shortcut(&app, &updated.chat_shortcut, true)?;
        if !applied {
            let previous_shortcut = current.chat_shortcut;
            let _ = settings.set_chat_shortcut(previous_shortcut)?;
            updated = settings.set_shortcut_enabled(false)?;
        }
    }
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_launch_at_startup(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    sync_autostart(&app, enabled)?;
    let updated = settings.set_launch_at_startup(enabled)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_main_window_always_on_top(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_main_window_always_on_top(enabled)?;
    windowing::apply_main_window_settings(&app, &updated)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_main_window_show_in_taskbar(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_main_window_show_in_taskbar(enabled)?;
    windowing::apply_main_window_settings(&app, &updated)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn reset_main_window_position(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let updated = settings.reset_main_position()?;
    windowing::reset_main_window_position(&app)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn save_settings_window_bounds(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<AppSettings, String> {
    let updated = settings.save_settings_window_bounds_throttled(SavedWindowBounds { x, y, width, height })?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn reset_settings_window_bounds(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let updated = settings.reset_settings_window_bounds()?;
    windowing::reset_settings_window(&app)?;
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn reset_all_settings(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    sync_autostart(&app, false)?;
    let mut updated = settings.reset_all()?;
    windowing::restore_main_window_position(&app, None)?;
    windowing::apply_main_window_settings(&app, &updated)?;
    if updated.shortcut_enabled {
        let applied = sync_chat_shortcut(&app, &updated.chat_shortcut, true)?;
        if !applied {
            updated = settings.set_shortcut_enabled(false)?;
        }
    } else {
        let _ = sync_chat_shortcut(&app, &updated.chat_shortcut, false)?;
    }
    windowing::reset_settings_window(&app)?;
    windowing::sync_info_window_visibility(&app)?;
    emit_settings(&app, &updated);
    Ok(updated)
}
