use tauri::{AppHandle, Emitter, Manager};

use crate::{
    feedback::SystemFeedbackState,
    reminder::ReminderState,
    settings::{InfoMode, SettingsState},
};

use super::{
    placement::{
        bubble_placement, context_menu_position, current_work_area, info_placement,
        OverlayPlacement,
    },
    policy::{overlay_winner, OverlayWinner},
    OverlayState, CHAT_WINDOW, CONTEXT_MENU_WINDOW, INFO_WINDOW, MAIN_WINDOW, REMINDER_WINDOW,
    SYSTEM_FEEDBACK_WINDOW,
};

const OVERLAY_PRESENT_EVENT: &str = "overlay://present";

pub fn reposition_visible_overlays(app: &AppHandle) {
    for label in [
        CHAT_WINDOW,
        INFO_WINDOW,
        REMINDER_WINDOW,
        SYSTEM_FEEDBACK_WINDOW,
    ] {
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

pub fn hide_context_menu(app: &AppHandle) -> Result<(), String> {
    hide_window(app, CONTEXT_MENU_WINDOW)?;
    sync_overlay_visibility(app)
}

pub fn show_context_menu(app: &AppHandle, x: f64, y: f64) -> Result<(), String> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return Err("右键菜单坐标无效".to_string());
    }
    hide_window(app, CHAT_WINDOW)?;
    hide_window(app, REMINDER_WINDOW)?;
    hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
    hide_window(app, INFO_WINDOW)?;

    let main = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let menu = app
        .get_webview_window(CONTEXT_MENU_WINDOW)
        .ok_or_else(|| "找不到右键菜单窗口".to_string())?;
    let main_position = main.outer_position().map_err(|error| error.to_string())?;
    let scale_factor = main.scale_factor().map_err(|error| error.to_string())?;
    let menu_size = menu.outer_size().map_err(|error| error.to_string())?;
    let area = current_work_area(&main).map_err(|error| error.to_string())?;
    let position = context_menu_position(main_position, scale_factor, x, y, menu_size, area);

    menu.set_position(position)
        .map_err(|error| error.to_string())?;
    menu.show().map_err(|error| error.to_string())?;
    menu.set_focus().map_err(|error| error.to_string())?;
    menu.emit("context-menu://focus", ())
        .map_err(|error| error.to_string())
}

pub fn toggle_chat_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        window.hide().map_err(|error| error.to_string())?;
        return sync_overlay_visibility(app);
    }

    show_chat_window(app)
}

pub fn show_chat_window(app: &AppHandle) -> Result<(), String> {
    hide_window(app, CONTEXT_MENU_WINDOW)?;
    let window = app
        .get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?;
    if !window.is_visible().map_err(|error| error.to_string())? {
        present_overlay(app, CHAT_WINDOW)?;
    }
    sync_overlay_visibility(app)?;
    window.set_focus().map_err(|error| error.to_string())?;
    window
        .emit("chat://focus-input", ())
        .map_err(|error| error.to_string())
}

pub fn hide_chat_window(app: &AppHandle) -> Result<(), String> {
    hide_window(app, CHAT_WINDOW)?;
    sync_overlay_visibility(app)
}

pub fn request_info_window_visibility(app: &AppHandle, visible: bool) -> Result<(), String> {
    if let Some(state) = app.try_state::<OverlayState>() {
        state.set_info_requested_visible(visible)?;
    }
    sync_overlay_visibility(app)
}

pub fn show_info_window_now(app: &AppHandle) -> Result<(), String> {
    hide_window(app, CONTEXT_MENU_WINDOW)?;
    if matches!(
        overlay_winner(
            false,
            chat_window_visible(app),
            reminder_active(app),
            system_feedback_active(app),
            true,
        ),
        OverlayWinner::Info
    ) {
        return present_overlay(app, INFO_WINDOW);
    }
    sync_overlay_visibility(app)
}

pub fn sync_info_window_visibility(app: &AppHandle) -> Result<(), String> {
    sync_overlay_visibility(app)
}

pub fn sync_reminder_window_visibility(app: &AppHandle) -> Result<(), String> {
    sync_overlay_visibility(app)
}

pub fn sync_system_feedback_window_visibility(app: &AppHandle) -> Result<(), String> {
    sync_overlay_visibility(app)
}

pub fn sync_overlay_visibility(app: &AppHandle) -> Result<(), String> {
    match overlay_winner(
        context_menu_visible(app),
        chat_window_visible(app),
        reminder_active(app),
        system_feedback_active(app),
        info_should_show(app),
    ) {
        OverlayWinner::ContextMenu => hide_lower_overlays(app),
        OverlayWinner::Chat => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            hide_window(app, INFO_WINDOW)
        }
        OverlayWinner::Reminder => {
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            hide_window(app, INFO_WINDOW)?;
            present_reminder_overlay(app)
        }
        OverlayWinner::Feedback => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, INFO_WINDOW)?;
            present_overlay(app, SYSTEM_FEEDBACK_WINDOW)
        }
        OverlayWinner::Info => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            present_overlay(app, INFO_WINDOW)
        }
        OverlayWinner::None => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            hide_window(app, INFO_WINDOW)
        }
    }
}

fn hide_lower_overlays(app: &AppHandle) -> Result<(), String> {
    hide_window(app, CHAT_WINDOW)?;
    hide_window(app, REMINDER_WINDOW)?;
    hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
    hide_window(app, INFO_WINDOW)
}

fn present_reminder_overlay(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(REMINDER_WINDOW)
        .ok_or_else(|| "找不到提醒窗口".to_string())?;
    let was_visible = window.is_visible().map_err(|error| error.to_string())?;
    present_overlay(app, REMINDER_WINDOW)?;
    if !was_visible {
        window.set_focus().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn hide_window(app: &AppHandle, label: &str) -> Result<(), String> {
    app.get_webview_window(label)
        .ok_or_else(|| format!("找不到 {label} 窗口"))?
        .hide()
        .map_err(|error| error.to_string())
}

fn current_info_mode(app: &AppHandle) -> InfoMode {
    app.try_state::<SettingsState>()
        .and_then(|settings| settings.get().ok())
        .map(|settings| settings.info_mode)
        .unwrap_or(InfoMode::Auto)
}

fn info_should_show(app: &AppHandle) -> bool {
    match current_info_mode(app) {
        InfoMode::Hidden => false,
        InfoMode::Always => true,
        InfoMode::Auto => info_requested_visible(app),
    }
}

fn context_menu_visible(app: &AppHandle) -> bool {
    app.get_webview_window(CONTEXT_MENU_WINDOW)
        .is_some_and(|window| window.is_visible().unwrap_or(false))
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

fn system_feedback_active(app: &AppHandle) -> bool {
    app.try_state::<SystemFeedbackState>()
        .and_then(|state| state.active_payload().ok())
        .flatten()
        .is_some()
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
        CHAT_WINDOW | REMINDER_WINDOW | SYSTEM_FEEDBACK_WINDOW => {
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
