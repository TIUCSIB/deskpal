mod score;

use serde::Serialize;
use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow};

use score::choose_placement;

const BUBBLE_OVERLAY_GAP: i32 = 4;
const INFO_OVERLAY_GAP: i32 = 10;
const MAIN_WINDOW_RIGHT_MARGIN: i32 = 24;
const MAIN_WINDOW_BOTTOM_MARGIN: i32 = 36;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlaySide {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayPlacement {
    pub position: PhysicalPosition<i32>,
    pub side: OverlaySide,
}

pub fn clamp_position(
    x: i32,
    y: i32,
    size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let max_x = area.x + area.width.saturating_sub(size.width) as i32;
    let max_y = area.y + area.height.saturating_sub(size.height) as i32;
    PhysicalPosition::new(x.clamp(area.x, max_x), y.clamp(area.y, max_y))
}

pub fn bubble_placement(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> OverlayPlacement {
    let preferred_side = horizontal_priority(main_position, main_size, area);
    choose_placement(
        main_position,
        main_size,
        overlay_size,
        area,
        BUBBLE_OVERLAY_GAP,
        [
            OverlaySide::Above,
            OverlaySide::Below,
            preferred_side,
            opposite_horizontal_side(preferred_side),
        ],
    )
}

pub fn info_placement(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> OverlayPlacement {
    let first_side = horizontal_priority(main_position, main_size, area);
    let first_vertical = vertical_priority(main_position, main_size, area);
    choose_placement(
        main_position,
        main_size,
        overlay_size,
        area,
        INFO_OVERLAY_GAP,
        [
            first_side,
            opposite_horizontal_side(first_side),
            first_vertical,
            opposite_vertical_side(first_vertical),
        ],
    )
}

pub fn anchored_resize_position(
    position: PhysicalPosition<i32>,
    old_size: PhysicalSize<u32>,
    new_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    clamp_position(
        position.x + old_size.width as i32 / 2 - new_size.width as i32 / 2,
        position.y + old_size.height as i32 - new_size.height as i32,
        new_size,
        area,
    )
}

pub fn default_main_position(size: PhysicalSize<u32>, area: Bounds) -> PhysicalPosition<i32> {
    clamp_position(
        area.x + area.width as i32 - size.width as i32 - MAIN_WINDOW_RIGHT_MARGIN,
        area.y + area.height as i32 - size.height as i32 - MAIN_WINDOW_BOTTOM_MARGIN,
        size,
        area,
    )
}

pub fn current_work_area(window: &WebviewWindow) -> tauri::Result<Bounds> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| tauri::Error::AssetNotFound("monitor work area".into()))?;
    Ok(Bounds::from(monitor.work_area()))
}

fn horizontal_priority(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    area: Bounds,
) -> OverlaySide {
    let left_space = main_position.x - area.x;
    let right_space = area.x + area.width as i32 - (main_position.x + main_size.width as i32);
    if right_space >= left_space {
        OverlaySide::Right
    } else {
        OverlaySide::Left
    }
}

fn vertical_priority(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    area: Bounds,
) -> OverlaySide {
    let top_space = main_position.y - area.y;
    let bottom_space = area.y + area.height as i32 - (main_position.y + main_size.height as i32);
    if bottom_space >= top_space {
        OverlaySide::Below
    } else {
        OverlaySide::Above
    }
}

fn opposite_horizontal_side(side: OverlaySide) -> OverlaySide {
    match side {
        OverlaySide::Left => OverlaySide::Right,
        OverlaySide::Right => OverlaySide::Left,
        _ => unreachable!("horizontal direction is required"),
    }
}

fn opposite_vertical_side(side: OverlaySide) -> OverlaySide {
    match side {
        OverlaySide::Above => OverlaySide::Below,
        OverlaySide::Below => OverlaySide::Above,
        _ => unreachable!("vertical direction is required"),
    }
}
