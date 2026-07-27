use tauri::AppHandle;

use crate::{
    reminder::{self, ReminderPayload},
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
pub fn active_reminder_payload(app: AppHandle) -> Result<Option<ReminderPayload>, String> {
    reminder::active_payload(&app)
}

#[tauri::command]
pub fn dismiss_reminder_window(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::dismiss(&app, reminder_id)
}

#[tauri::command]
pub fn snooze_reminder(app: AppHandle, reminder_id: String) -> Result<(), String> {
    reminder::snooze(&app, reminder_id)
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
