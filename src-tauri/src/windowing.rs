mod guard;
mod overlay;
mod passthrough;
mod placement;
mod policy;
mod state;
mod zorder;

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

use crate::settings::{AppSettings, DEFAULT_SETTINGS_WINDOW_HEIGHT, DEFAULT_SETTINGS_WINDOW_WIDTH};
pub(crate) use guard::start_overlay_guard;
pub use overlay::{
    hide_chat_window, hide_context_menu, reposition_visible_overlays,
    request_info_window_visibility, show_chat_window, show_context_menu, show_info_window_now,
    sync_info_window_visibility, sync_overlay_visibility, sync_reminder_window_visibility,
    sync_system_feedback_window_visibility, toggle_chat_window,
};
pub use passthrough::forward_main_left_click;
use placement::{current_work_area, reclamped_main_position, resize_plan, restored_main_position};
pub use state::OverlayState;

pub const MAIN_WINDOW: &str = "main";
pub const CONTEXT_MENU_WINDOW: &str = "context-menu";
pub const CHAT_WINDOW: &str = "chat";
pub const INFO_WINDOW: &str = "info";
pub const SETTINGS_WINDOW: &str = "settings";
pub const REMINDER_WINDOW: &str = "reminder";
pub const SYSTEM_FEEDBACK_WINDOW: &str = "feedback";

const INFO_CONTENT_WIDTH: f64 = 232.0;
const INFO_CONTENT_HEIGHT: f64 = 158.0;
const INFO_WINDOW_SAFE_PADDING: f64 = 8.0;
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
    let area = current_work_area(&window).map_err(|error| error.to_string())?;
    let plan = resize_plan(old_position, old_size, (width, height), scale_factor, area)
        .ok_or_else(|| "窗口大小或缩放比例无效".to_string())?;

    window
        .set_size(logical_size)
        .map_err(|error| error.to_string())?;
    window
        .set_position(plan.position)
        .map_err(|error| error.to_string())?;
    reposition_visible_overlays(app);
    Ok(())
}

pub(crate) fn info_window_size(scale: f64) -> LogicalSize<f64> {
    let safe_scale = clamp_scale(scale).max(MIN_INFO_SCALE);
    LogicalSize::new(
        INFO_CONTENT_WIDTH * safe_scale + INFO_WINDOW_SAFE_PADDING,
        INFO_CONTENT_HEIGHT * safe_scale + INFO_WINDOW_SAFE_PADDING,
    )
}

pub fn resize_info_window(app: &AppHandle, scale: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(INFO_WINDOW)
        .ok_or_else(|| "找不到信息窗口".to_string())?;
    window
        .set_size(info_window_size(scale))
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
    let position = restored_main_position(saved_position, size, area);

    window
        .set_position(position)
        .map_err(|error| error.to_string())?;
    reposition_visible_overlays(app);
    Ok(())
}

pub fn reclamp_main_window_position(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let area = current_work_area(&window).map_err(|error| error.to_string())?;
    let clamped = reclamped_main_position(position, size, area);

    if clamped != position {
        window
            .set_position(clamped)
            .map_err(|error| error.to_string())?;
    }
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
    #[cfg(target_os = "windows")]
    window
        .set_ignore_cursor_events(false)
        .map_err(|error| error.to_string())?;
    // 浮窗始终跟随桌宠置顶，避免设置变更后与主窗口层级脱节
    reinforce_overlay_windows_topmost_enabled(app, settings.main_window_always_on_top);
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

/** 全部浮窗标签 —— 需要保持置顶、但不参与互斥仲裁的窗口。 */
pub const OVERLAY_WINDOW_LABELS: [&str; 5] = [
    CONTEXT_MENU_WINDOW,
    CHAT_WINDOW,
    INFO_WINDOW,
    REMINDER_WINDOW,
    SYSTEM_FEEDBACK_WINDOW,
];

/**
 * 刷新窗口呈现并重新置于顶层。
 *
 * 相比单纯的 `show()`，此函数会显式恢复 topmost 标记并重排 Z 序，
 * 用于窗口创建、主窗口重新呈现以及系统级事件之后。
 */
pub(super) fn refresh_window_presentation(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    zorder::ensure_on_top(window)
}

/**
 * 为单个浮窗重新施加置顶状态（窗口显示或失去层级后调用）。
 *
 * 走不抖动的路径：本函数在每次浮窗显示时都会执行（悬停驱动，可达较高频率），
 * 位置写入会带来无谓开销与 1 物理像素的闪烁风险。
 */
pub(super) fn reinforce_window_topmost(window: &tauri::WebviewWindow) -> Result<(), String> {
    zorder::assert_on_top(window)
}

/**
 * 读取用户的窗口置顶偏好。
 *
 * 返回 `None` 表示设置状态尚未就绪 —— 调用方应按"未就绪则保持既有默认
 * 行为"处理。该偏好是浮窗置顶的唯一权威来源：用户在设置中关闭置顶后，
 * 所有浮窗层级维护（显示时置顶、守护修复）都必须停止介入。
 */
pub(super) fn topmost_preference(app: &AppHandle) -> Option<bool> {
    app.try_state::<crate::settings::SettingsState>()?
        .get()
        .ok()
        .map(|settings| settings.main_window_always_on_top)
}

/**
 * 按用户偏好将窗口置于顶层。
 *
 * 设置未就绪时沿用默认置顶行为，就绪后以设置值为准。所有"显示窗口后
 * 需要置顶"的调用点都应经由此函数，以保证置顶策略只有一个判定入口。
 * 置顶失败不向上传播 —— 层级维护属于尽力而为的增强，不应阻断窗口显隐。
 */
pub(super) fn apply_topmost_per_preference(app: &AppHandle, window: &tauri::WebviewWindow) {
    if !topmost_preference(app).unwrap_or(true) {
        return;
    }
    let label = window.label().to_string();
    if let Err(error) = reinforce_window_topmost(window) {
        eprintln!("无法将 {label} 窗口置于顶层: {error}");
    }
}

/**
 * 对所有已显示的浮窗重新施加置顶状态。
 *
 * 该函数是层级问题的确定性兜底：DPI 变化、显示器热插拔、资源管理器重启、
 * 或跨进程 topmost 竞争导致层级漂移后，统一修复。
 */
pub fn reinforce_overlay_windows_topmost(app: &AppHandle) {
    reinforce_overlay_windows_topmost_enabled(app, true);
}

/** 按指定置顶状态刷新所有已显示的浮窗层级。 */
fn reinforce_overlay_windows_topmost_enabled(app: &AppHandle, topmost: bool) {
    let windows = OVERLAY_WINDOW_LABELS
        .iter()
        .filter_map(|label| {
            let window = app.get_webview_window(label)?;
            window.is_visible().ok().filter(|visible| *visible)?;
            Some((*label, window))
        })
        .collect::<Vec<_>>();
    if !topmost {
        for (label, window) in &windows {
            if let Err(error) = window.set_always_on_top(false) {
                eprintln!("无法取消 {label} 窗口置顶: {error}");
            }
        }
        return;
    }
    for (label, error) in zorder::ensure_all_on_top(windows) {
        eprintln!("无法恢复 {label} 窗口层级: {error}");
    }
}

fn present_main_window(app: &AppHandle, focus: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let _ = window.unminimize();
    window.show().map_err(|error| error.to_string())?;
    // 先让桌宠回到 topmost 层顶部，再同步浮窗，确保浮窗排在桌宠之上
    if let Err(error) = zorder::ensure_on_top(&window) {
        eprintln!("无法恢复桌宠窗口层级: {error}");
    }
    reposition_visible_overlays(app);
    sync_info_window_visibility(app)?;
    sync_reminder_window_visibility(app)?;
    sync_system_feedback_window_visibility(app)?;
    if focus {
        let _ = window.set_focus();
    }
    Ok(())
}

pub fn show_startup_main_window(app: &AppHandle) -> Result<(), String> {
    present_main_window(app, false)
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        restore_main_window_position(app, window.outer_position().ok())?;
    }
    present_main_window(app, true)
}

pub fn refresh_main_window_presentation(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    refresh_window_presentation(&window)
}

pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    let _ = present_main_window(app, false);
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
        .map_err(|error| error.to_string())?;
    sync_overlay_visibility(app)
}

pub fn hide_system_feedback_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(SYSTEM_FEEDBACK_WINDOW)
        .ok_or_else(|| "找不到系统反馈窗口".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "windowing/tests.rs"]
mod tests;
