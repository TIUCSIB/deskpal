use tauri::{AppHandle, Emitter, State};

use crate::{
    parse_chat_shortcut, reminder, role_packs,
    settings::{
        AppSettings, InfoMode, QuietHours, Reminder, ReminderInput, SavedPosition,
        SavedWindowBounds, SettingsState,
    },
    sync_autostart, sync_chat_shortcut, windowing,
};

const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";

pub(super) fn emit_settings(app: &AppHandle, settings: &AppSettings) {
    if let Err(error) = app.emit(SETTINGS_UPDATED_EVENT, settings) {
        eprintln!("无法同步应用设置事件: {error}");
    }
}

pub(super) fn finish_update(app: &AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    reminder::sync_from_settings(app)?;
    emit_settings(app, &settings);
    Ok(settings)
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
    finish_update(&app, settings.set_pet_scale(scale)?)
}

#[tauri::command]
pub fn set_pet_role(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    role: String,
) -> Result<AppSettings, String> {
    if !role_packs::is_valid_role(&app, &role) {
        return Err("未安装或不受支持的角色。".to_string());
    }
    finish_update(&app, settings.set_validated_pet_role(role)?)
}

#[tauri::command]
pub fn save_main_window_position(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    x: i32,
    y: i32,
) -> Result<AppSettings, String> {
    finish_update(
        &app,
        settings.save_main_position_throttled(SavedPosition { x, y })?,
    )
}

#[tauri::command]
pub fn set_info_mode(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    mode: InfoMode,
) -> Result<AppSettings, String> {
    let updated = settings.set_info_mode(mode)?;
    windowing::sync_info_window_visibility(&app)?;
    finish_update(&app, updated)
}

#[tauri::command]
pub fn set_size_locked(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    locked: bool,
) -> Result<AppSettings, String> {
    finish_update(&app, settings.set_size_locked(locked)?)
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
    finish_update(&app, updated)
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
    if updated.shortcut_enabled && !sync_chat_shortcut(&app, &updated.chat_shortcut, true)? {
        let _ = settings.set_chat_shortcut(current.chat_shortcut)?;
        updated = settings.set_shortcut_enabled(false)?;
    }
    finish_update(&app, updated)
}

#[tauri::command]
pub fn set_launch_at_startup(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    sync_autostart(&app, enabled)?;
    finish_update(&app, settings.set_launch_at_startup(enabled)?)
}

#[tauri::command]
pub fn set_main_window_always_on_top(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_main_window_always_on_top(enabled)?;
    windowing::apply_main_window_settings(&app, &updated)?;
    finish_update(&app, updated)
}

#[tauri::command]
pub fn set_main_window_show_in_taskbar(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    enabled: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_main_window_show_in_taskbar(enabled)?;
    windowing::apply_main_window_settings(&app, &updated)?;
    finish_update(&app, updated)
}

#[tauri::command]
pub fn set_reminder_quiet_hours(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    quiet_hours: QuietHours,
) -> Result<AppSettings, String> {
    finish_update(&app, settings.set_quiet_hours(quiet_hours)?)
}

#[tauri::command]
pub fn create_reminder(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    input: ReminderInput,
) -> Result<AppSettings, String> {
    finish_update(&app, settings.create_reminder(input)?)
}

#[tauri::command]
pub fn update_reminder(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    reminder: Reminder,
) -> Result<AppSettings, String> {
    finish_update(&app, settings.update_reminder(reminder)?)
}

#[tauri::command]
pub fn delete_reminder(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    id: String,
) -> Result<AppSettings, String> {
    let updated = settings.delete_reminder(id.clone())?;
    if let Err(error) = reminder::remove_reminder(&app, &id) {
        eprintln!("删除提醒后的运行状态同步失败: {error}");
    }
    if let Err(error) = reminder::sync_from_settings(&app) {
        eprintln!("删除提醒后的调度同步失败: {error}");
    }
    emit_settings(&app, &updated);
    Ok(updated)
}

#[tauri::command]
pub fn set_reminder_enabled(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    id: String,
    enabled: bool,
) -> Result<AppSettings, String> {
    let updated = settings.set_reminder_enabled(id.clone(), enabled)?;
    if !enabled {
        reminder::remove_reminder(&app, &id)?;
    }
    finish_update(&app, updated)
}

#[tauri::command]
pub fn resume_reminder(
    app: AppHandle,
    settings: State<'_, SettingsState>,
    id: String,
) -> Result<AppSettings, String> {
    finish_update(&app, settings.set_reminder_pause(id, None)?)
}

#[tauri::command]
pub fn reset_main_window_position(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let updated = settings.reset_main_position()?;
    windowing::reset_main_window_position(&app)?;
    finish_update(&app, updated)
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
    finish_update(
        &app,
        settings.save_settings_window_bounds_throttled(SavedWindowBounds {
            x,
            y,
            width,
            height,
        })?,
    )
}

#[tauri::command]
pub fn reset_settings_window_bounds(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let updated = settings.reset_settings_window_bounds()?;
    windowing::reset_settings_window(&app)?;
    finish_update(&app, updated)
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
        if !sync_chat_shortcut(&app, &updated.chat_shortcut, true)? {
            updated = settings.set_shortcut_enabled(false)?;
        }
    } else {
        let _ = sync_chat_shortcut(&app, &updated.chat_shortcut, false)?;
    }
    windowing::reset_settings_window(&app)?;
    windowing::sync_info_window_visibility(&app)?;
    finish_update(&app, updated)
}
