use tauri::{AppHandle, Emitter, Manager};

use crate::{
    feedback::SystemFeedbackState,
    reminder::ReminderState,
    settings::{InfoMode, SettingsState},
};

use super::{
    apply_topmost_per_preference,
    placement::{
        bubble_placement, context_menu_position, current_work_area, info_placement,
        OverlayPlacement,
    },
    policy::{overlay_winner, OverlayWinner},
    OverlayState, CHAT_WINDOW, CONTEXT_MENU_WINDOW, INFO_WINDOW, MAIN_WINDOW, REMINDER_WINDOW,
    SYSTEM_FEEDBACK_WINDOW,
};

const OVERLAY_PRESENT_EVENT: &str = "overlay://present";
/** 浮窗因更高优先级浮窗仲裁而被隐藏时，通知桌宠修正前端的 hover 去重状态。 */
const OVERLAY_SUPPRESSED_EVENT: &str = "overlay://suppressed";

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
    // 右键菜单即将显示并抢占信息窗，需通知前端重置 hover 去重
    preempt_info_window(app)?;

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
    apply_topmost_per_preference(app, &menu);
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
        // Chat / Reminder / Feedback 胜出：信息窗是被动挤出，需通知前端重置去重
        OverlayWinner::Chat => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            preempt_info_window(app)
        }
        OverlayWinner::Reminder => {
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            preempt_info_window(app)?;
            present_reminder_overlay(app)
        }
        OverlayWinner::Feedback => {
            hide_window(app, REMINDER_WINDOW)?;
            preempt_info_window(app)?;
            present_overlay(app, SYSTEM_FEEDBACK_WINDOW)
        }
        OverlayWinner::Info => {
            hide_window(app, REMINDER_WINDOW)?;
            hide_window(app, SYSTEM_FEEDBACK_WINDOW)?;
            present_overlay(app, INFO_WINDOW)
        }
        // None 表示 info_should_show() 为假（如信息模式设为"隐藏"）：
        // 前端去重状态与用户意图一致，不通知，避免每次移动都徒劳请求显示
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
    preempt_info_window(app)
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

/**
 * 隐藏浮窗的原因，决定是否需要通知桌宠窗口重置 hover 去重状态。
 *
 * 前端 `Pet.vue` 用闭包变量 `hoveringPetPixel` 去重 hover 事件：一旦记录了
 * "指针在宠物上"，后续 `pointermove` 就不再上报。原生层若在此时隐藏了信息窗，
 * 前端状态便与原生状态漂移，表现为"移回去也不显示"。
 *
 * 因此只有**被更高优先级浮窗抢占**这一种情况需要通知；用户主动隐藏、或
 * 信息模式本身为"隐藏"时，前端去重状态是正确的，通知反而会造成无谓往返。
 */
#[derive(Clone, Copy, PartialEq, Eq)]
enum HideReason {
    /// 被更高优先级浮窗抢占 —— 需通知前端重置 hover 去重。
    Preempted,
    /// 调用方刻意隐藏（用户操作、模式为隐藏、清理同级浮窗）—— 不通知。
    Intentional,
}

fn hide_window(app: &AppHandle, label: &str) -> Result<(), String> {
    hide_window_with_reason(app, label, HideReason::Intentional)
}

/**
 * 隐藏指定浮窗，并按 `reason` 决定是否通知桌宠窗口。
 *
 * 仅对信息窗发送通知：其他浮窗的显隐不由 hover 驱动，与前端去重状态无关；
 * 这也避免了"隐藏 → 通知 → 重新请求显示 → 再次隐藏"的循环。前端收到通知
 * 后只做状态重置，需等下一次真实指针移动才会重新请求显示。
 */
fn hide_window_with_reason(app: &AppHandle, label: &str, reason: HideReason) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("找不到 {label} 窗口"))?;
    let was_visible = window.is_visible().unwrap_or(false);
    window.hide().map_err(|error| error.to_string())?;
    if was_visible && label == INFO_WINDOW && reason == HideReason::Preempted {
        notify_overlay_suppressed(app, label);
    }
    Ok(())
}

/** 隐藏因优先级仲裁而落败的信息窗，并通知前端重置 hover 去重状态。 */
fn preempt_info_window(app: &AppHandle) -> Result<(), String> {
    hide_window_with_reason(app, INFO_WINDOW, HideReason::Preempted)
}

/** 告知桌宠窗口：某个浮窗被仲裁隐藏，请重置 hover 去重状态。 */
fn notify_overlay_suppressed(app: &AppHandle, label: &str) {
    let Some(main) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    if let Err(error) = main.emit_to(MAIN_WINDOW, OVERLAY_SUPPRESSED_EVENT, label) {
        eprintln!("无法通知桌宠窗口浮窗已隐藏: {error}");
    }
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

/**
 * 显示浮窗并将其置于顶层。
 *
 * `was_hidden` 与 `resent` 的区分很关键：
 * - `was_hidden` 为真时才发送 `overlay://present`，避免每次 hover 重播入场动画；
 * - 无论窗口此前是否可见，都必须重新施加 topmost —— 这正是"信息窗被浏览器
 *   盖住"的修复点。窗口可见不代表它处于正确的 Z 序位置。
 *
 * 置顶受用户偏好约束：设置未就绪（`None`）时保持默认置顶，就绪后以设置值
 * 为准。用户关闭置顶时不得再次强制施加，否则设置项将形同虚设。
 */
fn present_overlay(app: &AppHandle, label: &str) -> Result<(), String> {
    let placement = reposition_overlay(app, label)?;
    let overlay = app
        .get_webview_window(label)
        .ok_or_else(|| format!("找不到 {label} 窗口"))?;
    let was_hidden = !overlay.is_visible().map_err(|error| error.to_string())?;
    overlay.show().map_err(|error| error.to_string())?;
    apply_topmost_per_preference(app, &overlay);
    if was_hidden {
        overlay
            .emit(OVERLAY_PRESENT_EVENT, placement.side)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
