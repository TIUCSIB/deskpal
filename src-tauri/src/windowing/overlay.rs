use tauri::{AppHandle, Emitter, Manager};

use crate::{
    reminder::ReminderState,
    settings::{InfoMode, SettingsState},
};

use super::{
    placement::{bubble_placement, current_work_area, info_placement, OverlayPlacement},
    OverlayState, CHAT_WINDOW, INFO_WINDOW, MAIN_WINDOW, REMINDER_WINDOW,
};

const OVERLAY_PRESENT_EVENT: &str = "overlay://present";

pub fn reposition_visible_overlays(app: &AppHandle) {
    for label in [CHAT_WINDOW, INFO_WINDOW, REMINDER_WINDOW] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        if !window.is_visible().unwrap_or(false) {
            continue;
        }
        if let Err(error) = reposition_overlay(app, label) {
            eprintln!("无法重新定位 {label} 窗口: {error}");
        }
    }
}

pub fn toggle_chat_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        window.hide().map_err(|error| error.to_string())?;
        sync_info_window_visibility(app)?;
        sync_reminder_window_visibility(app)?;
        return Ok(());
    }

    show_chat_window(app)
}

pub fn show_chat_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?;
    if !window.is_visible().map_err(|error| error.to_string())? {
        present_overlay(app, CHAT_WINDOW)?;
        sync_info_window_visibility(app)?;
        sync_reminder_window_visibility(app)?;
    }
    window.set_focus().map_err(|error| error.to_string())?;
    window
        .emit("chat://focus-input", ())
        .map_err(|error| error.to_string())
}

pub fn hide_chat_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?
        .hide()
        .map_err(|error| error.to_string())?;
    sync_info_window_visibility(app)?;
    sync_reminder_window_visibility(app)
}

pub fn request_info_window_visibility(app: &AppHandle, visible: bool) -> Result<(), String> {
    if let Some(state) = app.try_state::<OverlayState>() {
        state.set_info_requested_visible(visible)?;
    }
    sync_info_window_visibility(app)
}

pub fn show_info_window_now(app: &AppHandle) -> Result<(), String> {
    if chat_window_visible(app) {
        return Ok(());
    }
    present_overlay(app, INFO_WINDOW)
}

pub fn sync_info_window_visibility(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(INFO_WINDOW)
        .ok_or_else(|| "找不到信息窗口".to_string())?;
    let should_show = match current_info_mode(app) {
        InfoMode::Hidden => false,
        InfoMode::Always => !chat_window_visible(app),
        InfoMode::Auto => info_requested_visible(app) && !chat_window_visible(app),
    };
    if !should_show {
        return window.hide().map_err(|error| error.to_string());
    }
    present_overlay(app, INFO_WINDOW)
}

pub fn sync_reminder_window_visibility(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(REMINDER_WINDOW)
        .ok_or_else(|| "找不到提醒窗口".to_string())?;
    if !reminder_active(app) || chat_window_visible(app) {
        return window.hide().map_err(|error| error.to_string());
    }
    present_overlay(app, REMINDER_WINDOW)?;
    window.set_focus().map_err(|error| error.to_string())
}

fn current_info_mode(app: &AppHandle) -> InfoMode {
    app.try_state::<SettingsState>()
        .and_then(|settings| settings.get().ok())
        .map(|settings| settings.info_mode)
        .unwrap_or(InfoMode::Auto)
}

fn chat_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window(CHAT_WINDOW)
        .is_some_and(|window| window.is_visible().unwrap_or(false))
}

fn info_requested_visible(app: &AppHandle) -> bool {
    app.try_state::<OverlayState>()
        .map(|state| state.info_requested_visible())
        .unwrap_or(false)
}

fn reminder_active(app: &AppHandle) -> bool {
    app.try_state::<ReminderState>()
        .and_then(|state| state.active_payload().ok())
        .flatten()
        .is_some()
}

pub(super) fn reposition_overlay(app: &AppHandle, label: &str) -> Result<OverlayPlacement, String> {
    let main = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let overlay = app
        .get_webview_window(label)
        .ok_or_else(|| format!("找不到 {label} 窗口"))?;
    let main_position = main.outer_position().map_err(|error| error.to_string())?;
    let main_size = main.outer_size().map_err(|error| error.to_string())?;
    let overlay_size = overlay.outer_size().map_err(|error| error.to_string())?;
    let area = current_work_area(&main).map_err(|error| error.to_string())?;
    let placement = match label {
        CHAT_WINDOW | REMINDER_WINDOW => {
            bubble_placement(main_position, main_size, overlay_size, area)
        }
        INFO_WINDOW => info_placement(main_position, main_size, overlay_size, area),
        _ => return Err(format!("不支持定位窗口 {label}")),
    };
    overlay
        .set_position(placement.position)
        .map_err(|error| error.to_string())?;
    Ok(placement)
}

fn present_overlay(app: &AppHandle, label: &str) -> Result<(), String> {
    let placement = reposition_overlay(app, label)?;
    let overlay = app
        .get_webview_window(label)
        .ok_or_else(|| format!("找不到 {label} 窗口"))?;
    let was_visible = overlay.is_visible().map_err(|error| error.to_string())?;
    overlay.show().map_err(|error| error.to_string())?;
    if was_visible {
        return Ok(());
    }
    overlay
        .emit(OVERLAY_PRESENT_EVENT, placement.side)
        .map_err(|error| error.to_string())
}
