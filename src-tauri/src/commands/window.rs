use tauri::{AppHandle, Emitter};

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
pub fn show_reminders_paused_confirmation(app: AppHandle) -> Result<(), String> {
    feedback::show(&app, feedback::reminders_paused_confirmation())
}

#[tauri::command]
pub fn show_main_settings_window(app: AppHandle) -> Result<(), String> {
    windowing::show_settings_window(&app)
}

#[tauri::command]
pub fn exit_application(app: AppHandle) {
    app.exit(0)
}
