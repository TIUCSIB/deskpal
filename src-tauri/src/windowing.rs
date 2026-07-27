mod overlay;
mod placement;
mod state;

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

use crate::settings::{AppSettings, DEFAULT_SETTINGS_WINDOW_HEIGHT, DEFAULT_SETTINGS_WINDOW_WIDTH};
pub use overlay::{
    hide_chat_window, reposition_visible_overlays, request_info_window_visibility,
    sync_info_window_visibility, sync_reminder_window_visibility, toggle_chat_window,
};
use placement::{anchored_resize_position, current_work_area, default_main_position};
pub use state::OverlayState;

pub const MAIN_WINDOW: &str = "main";
pub const CHAT_WINDOW: &str = "chat";
pub const INFO_WINDOW: &str = "info";
pub const SETTINGS_WINDOW: &str = "settings";
pub const REMINDER_WINDOW: &str = "reminder";

const INFO_BASE_WIDTH: f64 = 240.0;
const INFO_BASE_HEIGHT: f64 = 144.0;
const MIN_PET_SCALE: f64 = 0.45;
const MAX_PET_SCALE: f64 = 1.2;
const MIN_INFO_SCALE: f64 = 0.78;

pub fn clamp_scale(scale: f64) -> f64 {
    if scale.is_finite() {
        scale.clamp(MIN_PET_SCALE, MAX_PET_SCALE)
    } else {
        1.0
    }
}

pub fn resize_main_window(app: &AppHandle, width: f64, height: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let old_position = window.outer_position().map_err(|error| error.to_string())?;
    let old_size = window.outer_size().map_err(|error| error.to_string())?;
    let scale_factor = window.scale_factor().map_err(|error| error.to_string())?;
    let logical_size = LogicalSize::new(width, height);
    let new_size = logical_size.to_physical::<u32>(scale_factor);
    let area = current_work_area(&window).map_err(|error| error.to_string())?;
    let new_position = anchored_resize_position(old_position, old_size, new_size, area);

    window
        .set_size(logical_size)
        .map_err(|error| error.to_string())?;
    window
        .set_position(new_position)
        .map_err(|error| error.to_string())?;
    reposition_visible_overlays(app);
    Ok(())
}

pub fn resize_info_window(app: &AppHandle, scale: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(INFO_WINDOW)
        .ok_or_else(|| "找不到信息窗口".to_string())?;
    let safe_scale = clamp_scale(scale).max(MIN_INFO_SCALE);
    window
        .set_size(LogicalSize::new(
            INFO_BASE_WIDTH * safe_scale,
            INFO_BASE_HEIGHT * safe_scale,
        ))
        .map_err(|error| error.to_string())?;
    if window.is_visible().unwrap_or(false) {
        overlay::reposition_overlay(app, INFO_WINDOW)?;
    }
    Ok(())
}

pub fn restore_main_window_position(
    app: &AppHandle,
    saved_position: Option<PhysicalPosition<i32>>,
) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let area = current_work_area(&window).map_err(|error| error.to_string())?;
    let position = saved_position
        .map(|saved| placement::clamp_position(saved.x, saved.y, size, area))
        .unwrap_or_else(|| default_main_position(size, area));

    window
        .set_position(position)
        .map_err(|error| error.to_string())?;
    reposition_visible_overlays(app);
    Ok(())
}

pub fn reset_main_window_position(app: &AppHandle) -> Result<(), String> {
    restore_main_window_position(app, None)
}

pub fn apply_main_window_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    window
        .set_always_on_top(settings.main_window_always_on_top)
        .map_err(|error| error.to_string())?;
    window
        .set_skip_taskbar(!settings.main_window_show_in_taskbar)
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn reset_settings_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SETTINGS_WINDOW)
        .ok_or_else(|| "找不到设置窗口".to_string())?;
    window
        .set_size(LogicalSize::new(
            DEFAULT_SETTINGS_WINDOW_WIDTH as f64,
            DEFAULT_SETTINGS_WINDOW_HEIGHT as f64,
        ))
        .map_err(|error| error.to_string())?;
    window.center().map_err(|error| error.to_string())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let saved_position = window.outer_position().ok();
    restore_main_window_position(app, saved_position)?;
    window.show().map_err(|error| error.to_string())?;
    reposition_visible_overlays(app);
    sync_info_window_visibility(app)?;
    sync_reminder_window_visibility(app)?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SETTINGS_WINDOW)
        .ok_or_else(|| "找不到设置窗口".to_string())?;
    let _ = window.unminimize();
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn hide_settings_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(SETTINGS_WINDOW)
        .ok_or_else(|| "找不到设置窗口".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

pub fn hide_reminder_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(REMINDER_WINDOW)
        .ok_or_else(|| "找不到提醒窗口".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "windowing/tests.rs"]
mod tests;
