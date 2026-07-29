use tauri::{AppHandle, Emitter, Manager};

use crate::{
    feedback::{self, SystemFeedbackPayload},
    reminder::{self, ReminderActivity, ReminderHistoryState, ReminderPayload},
    windowing,
};

#[tauri::command]
pub fn resize_main_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    windowing::resize_main_window(&app, width, height)
}

#[tauri::command]
pub fn resize_info_window(app: AppHandle, scale: f64) -> Result<(), String> {
    windowing::resize_info_window(&app, scale)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    windowing::show_main_window(&app)
}

#[tauri::command]
pub fn show_startup_main_window(app: AppHandle) -> Result<(), String> {
    windowing::show_startup_main_window(&app)
}

#[tauri::command]
pub fn refresh_main_window_presentation(app: AppHandle) -> Result<(), String> {
    windowing::refresh_main_window_presentation(&app)
}

#[tauri::command]
pub fn toggle_chat_window(app: AppHandle) -> Result<(), String> {
    windowing::toggle_chat_window(&app)
}

#[tauri::command]
pub fn show_chat_window(app: AppHandle) -> Result<(), String> {
    windowing::show_chat_window(&app)
}

#[tauri::command]
pub fn hide_chat_window(app: AppHandle) -> Result<(), String> {
    windowing::hide_chat_window(&app)
}

#[tauri::command]
pub fn hide_settings_window(app: AppHandle) -> Result<(), String> {
    windowing::hide_settings_window(&app)
}

#[tauri::command]
pub fn hide_reminder_window(app: AppHandle) -> Result<(), String> {
    windowing::hide_reminder_window(&app)
}

#[tauri::command]
pub fn hide_system_feedback_window(app: AppHandle) -> Result<(), String> {
    windowing::hide_system_feedback_window(&app)
}

#[tauri::command]
pub fn active_system_feedback_payload(
    app: AppHandle,
) -> Result<Option<SystemFeedbackPayload>, String> {
    feedback::active_payload(&app)
}

#[tauri::command]
pub fn show_system_feedback(app: AppHandle, payload: SystemFeedbackPayload) -> Result<(), String> {
    feedback::show(&app, payload)
}

#[tauri::command]
pub fn dismiss_system_feedback(app: AppHandle, id: String) -> Result<(), String> {
    feedback::dismiss(&app, id)
}

#[tauri::command]
pub fn active_reminder_payload(app: AppHandle) -> Result<Option<ReminderPayload>, String> {
    reminder::active_payload(&app)
}

#[tauri::command]
pub fn dismiss_reminder_window(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::dismiss(&app, reminder_id)
}

#[tauri::command]
pub fn complete_reminder_window(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::complete(&app, reminder_id)
}

#[tauri::command]
pub fn snooze_reminder(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::snooze(&app, reminder_id)
}

#[tauri::command]
pub fn get_reminder_activity(
    history: tauri::State<'_, ReminderHistoryState>,
    include_all_events: Option<bool>,
) -> Result<ReminderActivity, String> {
    history.activity_with_events(include_all_events.unwrap_or(false))
}

#[tauri::command]
pub fn clear_reminder_activity(
    app: AppHandle,
    history: tauri::State<'_, ReminderHistoryState>,
) -> Result<(), String> {
    history.clear()?;
    app.emit("pet://reminder-activity-updated", history.activity()?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn pause_reminder_until_tomorrow(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::pause_until_tomorrow(&app, reminder_id)
}

#[tauri::command]
pub fn preview_reminder_window(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::preview(&app, reminder_id)
}

#[tauri::command]
pub fn set_info_window_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    windowing::request_info_window_visibility(&app, visible)
}

#[tauri::command]
pub fn forward_main_left_click(app: AppHandle) -> Result<(), String> {
    windowing::forward_main_left_click(&app)
}

#[tauri::command]
pub fn show_main_context_menu(app: AppHandle, x: f64, y: f64) -> Result<(), String> {
    windowing::show_context_menu(&app, x, y)
}

#[tauri::command]
pub fn hide_main_context_menu(app: AppHandle) -> Result<(), String> {
    windowing::hide_context_menu(&app)
}

#[tauri::command]
pub fn show_main_context_status(app: AppHandle) -> Result<(), String> {
    windowing::show_info_window_now(&app)
}

#[tauri::command]
pub fn pause_all_reminders_until_tomorrow(app: AppHandle) -> Result<(), String> {
    reminder::pause_all_until_tomorrow(&app)
}

#[tauri::command]
pub fn pause_enabled_reminder_until_tomorrow(
    app: AppHandle,
    reminder_id: String,
) -> Result<String, String> {
    reminder::pause_enabled_reminder_until_tomorrow(&app, reminder_id)
}

#[tauri::command]
pub fn show_reminders_paused_confirmation(app: AppHandle) -> Result<(), String> {
    feedback::show(&app, feedback::reminders_paused_confirmation())
}

#[tauri::command]
pub fn show_reminder_paused_confirmation(
    app: AppHandle,
    reminder_id: String,
) -> Result<(), String> {
    let settings = app
        .try_state::<crate::settings::SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    let reminder = settings
        .reminders
        .iter()
        .find(|item| item.id == reminder_id)
        .ok_or_else(|| "找不到该提醒".to_string())?;
    if !reminder.enabled || reminder.paused_until.is_none() {
        return Err("该提醒尚未暂停".to_string());
    }
    feedback::show(
        &app,
        feedback::reminder_paused_confirmation(&reminder.message),
    )
}

#[tauri::command]
pub fn show_main_settings_window(app: AppHandle) -> Result<(), String> {
    windowing::show_settings_window(&app)
}

#[tauri::command]
pub fn show_main_reminder_settings(app: AppHandle) -> Result<(), String> {
    windowing::show_settings_window(&app)?;
    app.get_webview_window(windowing::SETTINGS_WINDOW)
        .ok_or_else(|| "找不到设置窗口".to_string())?
        .emit("settings://focus-section", "reminder")
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn exit_application(app: AppHandle) {
    if let Some(settings) = app.try_state::<crate::settings::SettingsState>() {
        let _ = settings.flush_pending_geometry();
    }
    app.exit(0)
}
