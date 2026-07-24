use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalRect, PhysicalSize,
    WebviewWindow,
};

pub const MAIN_WINDOW: &str = "main";
pub const CHAT_WINDOW: &str = "chat";
pub const INFO_WINDOW: &str = "info";

const OVERLAY_GAP: i32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl From<&PhysicalRect<i32, u32>> for Bounds {
    fn from(rect: &PhysicalRect<i32, u32>) -> Self {
        Self {
            x: rect.position.x,
            y: rect.position.y,
            width: rect.size.width,
            height: rect.size.height,
        }
    }
}

fn clamp_position(x: i32, y: i32, size: PhysicalSize<u32>, area: Bounds) -> PhysicalPosition<i32> {
    let max_x = area.x + area.width.saturating_sub(size.width) as i32;
    let max_y = area.y + area.height.saturating_sub(size.height) as i32;

    PhysicalPosition::new(x.clamp(area.x, max_x), y.clamp(area.y, max_y))
}

fn anchored_resize_position(
    position: PhysicalPosition<i32>,
    old_size: PhysicalSize<u32>,
    new_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let x = position.x + old_size.width as i32 / 2 - new_size.width as i32 / 2;
    let y = position.y + old_size.height as i32 - new_size.height as i32;
    clamp_position(x, y, new_size, area)
}

fn chat_position(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let centered_x = main_position.x + main_size.width as i32 / 2 - overlay_size.width as i32 / 2;
    let above_y = main_position.y - overlay_size.height as i32 - OVERLAY_GAP;
    let below_y = main_position.y + main_size.height as i32 + OVERLAY_GAP;
    let y = if above_y >= area.y { above_y } else { below_y };

    clamp_position(centered_x, y, overlay_size, area)
}

fn info_position(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let right_x = main_position.x + main_size.width as i32 + OVERLAY_GAP;
    let left_x = main_position.x - overlay_size.width as i32 - OVERLAY_GAP;
    let area_right = area.x + area.width as i32;
    let x = if right_x + overlay_size.width as i32 <= area_right {
        right_x
    } else {
        left_x
    };
    let centered_y = main_position.y + main_size.height as i32 / 2 - overlay_size.height as i32 / 2;

    clamp_position(x, centered_y, overlay_size, area)
}

fn current_work_area(window: &WebviewWindow) -> tauri::Result<Bounds> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| tauri::Error::AssetNotFound("monitor work area".into()))?;

    Ok(Bounds::from(monitor.work_area()))
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

fn position_overlay(app: &AppHandle, label: &str) -> Result<(), String> {
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
    let position = match label {
        CHAT_WINDOW => chat_position(main_position, main_size, overlay_size, area),
        INFO_WINDOW => info_position(main_position, main_size, overlay_size, area),
        _ => return Err(format!("不支持定位窗口 {label}")),
    };

    overlay
        .set_position(position)
        .map_err(|error| error.to_string())
}

pub fn reposition_visible_overlays(app: &AppHandle) {
    for label in [CHAT_WINDOW, INFO_WINDOW] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        if window.is_visible().unwrap_or(false) {
            if let Err(error) = position_overlay(app, label) {
                eprintln!("无法重新定位 {label} 窗口: {error}");
            }
        }
    }
}

pub fn toggle_chat_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?;

    if window.is_visible().map_err(|error| error.to_string())? {
        window.hide().map_err(|error| error.to_string())?;
        return Ok(());
    }

    if let Some(info) = app.get_webview_window(INFO_WINDOW) {
        let _ = info.hide();
    }
    position_overlay(app, CHAT_WINDOW)?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window
        .emit("chat://focus-input", ())
        .map_err(|error| error.to_string())
}

pub fn hide_chat_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(CHAT_WINDOW)
        .ok_or_else(|| "找不到聊天窗口".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

pub fn set_info_window_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(INFO_WINDOW)
        .ok_or_else(|| "找不到信息窗口".to_string())?;

    if visible {
        if app
            .get_webview_window(CHAT_WINDOW)
            .is_some_and(|chat| chat.is_visible().unwrap_or(false))
        {
            return Ok(());
        }
        position_overlay(app, INFO_WINDOW)?;
        window.show().map_err(|error| error.to_string())
    } else {
        window.hide().map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Bounds = Bounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    };

    #[test]
    fn resize_keeps_bottom_center_anchor() {
        let position = anchored_resize_position(
            PhysicalPosition::new(800, 700),
            PhysicalSize::new(200, 220),
            PhysicalSize::new(300, 330),
            AREA,
        );

        assert_eq!(position, PhysicalPosition::new(750, 590));
    }

    #[test]
    fn chat_moves_below_when_top_has_no_room() {
        let position = chat_position(
            PhysicalPosition::new(20, 30),
            PhysicalSize::new(160, 200),
            PhysicalSize::new(340, 180),
            AREA,
        );

        assert_eq!(position, PhysicalPosition::new(0, 240));
    }

    #[test]
    fn info_moves_to_left_at_right_edge() {
        let position = info_position(
            PhysicalPosition::new(1760, 600),
            PhysicalSize::new(160, 200),
            PhysicalSize::new(240, 180),
            AREA,
        );

        assert_eq!(position, PhysicalPosition::new(1510, 610));
    }
}
