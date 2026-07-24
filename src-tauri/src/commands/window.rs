use tauri::{AppHandle, WebviewWindow};

use crate::{menu, windowing};

#[tauri::command]
pub fn resize_main_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    windowing::resize_main_window(&app, width, height)
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
pub fn set_info_window_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    windowing::set_info_window_visible(&app, visible)
}

#[tauri::command]
pub fn show_context_menu(
    app: AppHandle,
    window: WebviewWindow,
    x: f64,
    y: f64,
    scale: f64,
) -> Result<(), String> {
    menu::show_context_menu(&app, &window, x, y, scale)
}
