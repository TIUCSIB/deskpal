use tauri::AppHandle;

use crate::windowing;

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
pub fn set_info_window_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    windowing::request_info_window_visibility(&app, visible)
}
