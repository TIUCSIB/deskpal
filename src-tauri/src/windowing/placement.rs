use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow};

const CHAT_OVERLAY_GAP: i32 = 4;
const INFO_OVERLAY_GAP: i32 = 10;
const OVERLAY_SAFE_MARGIN: i32 = 16;
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

pub fn clamp_position(x: i32, y: i32, size: PhysicalSize<u32>, area: Bounds) -> PhysicalPosition<i32> {
    let max_x = area.x + area.width.saturating_sub(size.width) as i32;
    let max_y = area.y + area.height.saturating_sub(size.height) as i32;
    PhysicalPosition::new(x.clamp(area.x, max_x), y.clamp(area.y, max_y))
}

fn rect_visible_area(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> u64 {
    let area_right = area.x + area.width as i32;
    let area_bottom = area.y + area.height as i32;
    let visible_width = (position.x + size.width as i32).min(area_right) - position.x.max(area.x);
    let visible_height = (position.y + size.height as i32).min(area_bottom) - position.y.max(area.y);
    if visible_width <= 0 || visible_height <= 0 {
        return 0;
    }
    visible_width as u64 * visible_height as u64
}

fn rect_overlap_area(
    first_position: PhysicalPosition<i32>,
    first_size: PhysicalSize<u32>,
    second_position: PhysicalPosition<i32>,
    second_size: PhysicalSize<u32>,
) -> u64 {
    let overlap_width =
        (first_position.x + first_size.width as i32).min(second_position.x + second_size.width as i32)
            - first_position.x.max(second_position.x);
    let overlap_height =
        (first_position.y + first_size.height as i32).min(second_position.y + second_size.height as i32)
            - first_position.y.max(second_position.y);
    if overlap_width <= 0 || overlap_height <= 0 {
        return 0;
    }
    overlap_width as u64 * overlap_height as u64
}

fn inset_bounds(area: Bounds, margin: i32) -> Bounds {
    let horizontal_margin = margin.max(0).min(area.width.saturating_sub(1) as i32 / 2);
    let vertical_margin = margin.max(0).min(area.height.saturating_sub(1) as i32 / 2);

    Bounds {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: area.width.saturating_sub((horizontal_margin * 2) as u32),
        height: area.height.saturating_sub((vertical_margin * 2) as u32),
    }
}

fn min_edge_clearance(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> i32 {
    let right = area.x + area.width as i32 - (position.x + size.width as i32);
    let bottom = area.y + area.height as i32 - (position.y + size.height as i32);
    (position.x - area.x)
        .min(position.y - area.y)
        .min(right)
        .min(bottom)
}

fn is_within_area(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> bool {
    let area_right = area.x + area.width as i32;
    let area_bottom = area.y + area.height as i32;
    position.x >= area.x
        && position.y >= area.y
        && position.x + size.width as i32 <= area_right
        && position.y + size.height as i32 <= area_bottom
}

#[derive(Clone, Copy)]
struct OverlayCandidate {
    position: PhysicalPosition<i32>,
    visible_area: u64,
    overlap_area: u64,
    full_fit: bool,
    safe_fit: bool,
    edge_clearance: i32,
    clamp_distance: u32,
    order: u8,
}

fn candidate_position(
    ideal_position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    area: Bounds,
    safe_area: Bounds,
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    order: u8,
) -> OverlayCandidate {
    let position = clamp_position(ideal_position.x, ideal_position.y, size, area);
    let clamp_x = (position.x - ideal_position.x).unsigned_abs();
    let clamp_y = (position.y - ideal_position.y).unsigned_abs();

    OverlayCandidate {
        position,
        visible_area: rect_visible_area(position, size, area),
        overlap_area: rect_overlap_area(position, size, main_position, main_size),
        full_fit: is_within_area(ideal_position, size, area),
        safe_fit: is_within_area(ideal_position, size, safe_area),
        edge_clearance: min_edge_clearance(position, size, area),
        clamp_distance: clamp_x + clamp_y,
        order,
    }
}

fn choose_position(candidates: [OverlayCandidate; 4]) -> PhysicalPosition<i32> {
    candidates
        .into_iter()
        .max_by_key(|candidate| {
            (
                u8::from(candidate.safe_fit),
                u8::from(candidate.full_fit),
                candidate.visible_area,
                u64::MAX - candidate.overlap_area,
                candidate.edge_clearance,
                u32::MAX - candidate.clamp_distance,
                u8::MAX - candidate.order,
            )
        })
        .expect("at least one candidate")
        .position
}

pub fn anchored_resize_position(
    position: PhysicalPosition<i32>,
    old_size: PhysicalSize<u32>,
    new_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let x = position.x + old_size.width as i32 / 2 - new_size.width as i32 / 2;
    let y = position.y + old_size.height as i32 - new_size.height as i32;
    clamp_position(x, y, new_size, area)
}

pub fn chat_position(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let centered_x = main_position.x + main_size.width as i32 / 2 - overlay_size.width as i32 / 2;
    let above_y = main_position.y - overlay_size.height as i32 - CHAT_OVERLAY_GAP;
    let below_y = main_position.y + main_size.height as i32 + CHAT_OVERLAY_GAP;
    let y = if above_y >= area.y { above_y } else { below_y };
    clamp_position(centered_x, y, overlay_size, area)
}

pub fn info_position(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
) -> PhysicalPosition<i32> {
    let centered_x = main_position.x + main_size.width as i32 / 2 - overlay_size.width as i32 / 2;
    let centered_y = main_position.y + main_size.height as i32 / 2 - overlay_size.height as i32 / 2;
    let right_x = main_position.x + main_size.width as i32 + INFO_OVERLAY_GAP;
    let left_x = main_position.x - overlay_size.width as i32 - INFO_OVERLAY_GAP;
    let above_y = main_position.y - overlay_size.height as i32 - INFO_OVERLAY_GAP;
    let below_y = main_position.y + main_size.height as i32 + INFO_OVERLAY_GAP;
    let safe_area = inset_bounds(area, OVERLAY_SAFE_MARGIN);

    let left_space = main_position.x - area.x;
    let right_space = area.x + area.width as i32 - (main_position.x + main_size.width as i32);
    let top_space = main_position.y - area.y;
    let bottom_space = area.y + area.height as i32 - (main_position.y + main_size.height as i32);

    let side_positions = if right_space >= left_space {
        [
            candidate_position(
                PhysicalPosition::new(right_x, centered_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                0,
            ),
            candidate_position(
                PhysicalPosition::new(left_x, centered_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                1,
            ),
        ]
    } else {
        [
            candidate_position(
                PhysicalPosition::new(left_x, centered_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                0,
            ),
            candidate_position(
                PhysicalPosition::new(right_x, centered_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                1,
            ),
        ]
    };

    let vertical_positions = if bottom_space >= top_space {
        [
            candidate_position(
                PhysicalPosition::new(centered_x, below_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                2,
            ),
            candidate_position(
                PhysicalPosition::new(centered_x, above_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                3,
            ),
        ]
    } else {
        [
            candidate_position(
                PhysicalPosition::new(centered_x, above_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                2,
            ),
            candidate_position(
                PhysicalPosition::new(centered_x, below_y),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                3,
            ),
        ]
    };

    choose_position([
        side_positions[0],
        side_positions[1],
        vertical_positions[0],
        vertical_positions[1],
    ])
}

pub fn default_main_position(size: PhysicalSize<u32>, area: Bounds) -> PhysicalPosition<i32> {
    let x = area.x + area.width as i32 - size.width as i32 - MAIN_WINDOW_RIGHT_MARGIN;
    let y = area.y + area.height as i32 - size.height as i32 - MAIN_WINDOW_BOTTOM_MARGIN;
    clamp_position(x, y, size, area)
}

pub fn current_work_area(window: &WebviewWindow) -> tauri::Result<Bounds> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| tauri::Error::AssetNotFound("monitor work area".into()))?;
    Ok(Bounds::from(monitor.work_area()))
}
