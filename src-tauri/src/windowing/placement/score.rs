use tauri::{PhysicalPosition, PhysicalSize};

use super::{clamp_position, Bounds, OverlayPlacement, OverlaySide};

const OVERLAY_SAFE_MARGIN: i32 = 16;

#[derive(Clone, Copy)]
struct Candidate {
    placement: OverlayPlacement,
    visible_area: u64,
    overlap_area: u64,
    full_fit: bool,
    safe_fit: bool,
    edge_clearance: i32,
    clamp_distance: u32,
    order: u8,
}

pub(super) fn choose_placement(
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
    gap: i32,
    order: [OverlaySide; 4],
) -> OverlayPlacement {
    let centered_x = main_position.x + main_size.width as i32 / 2 - overlay_size.width as i32 / 2;
    let centered_y = main_position.y + main_size.height as i32 / 2 - overlay_size.height as i32 / 2;
    let safe_area = inset_bounds(area, OVERLAY_SAFE_MARGIN);
    let ideal = |side| match side {
        OverlaySide::Above => PhysicalPosition::new(
            centered_x,
            main_position.y - overlay_size.height as i32 - gap,
        ),
        OverlaySide::Below => {
            PhysicalPosition::new(centered_x, main_position.y + main_size.height as i32 + gap)
        }
        OverlaySide::Left => PhysicalPosition::new(
            main_position.x - overlay_size.width as i32 - gap,
            centered_y,
        ),
        OverlaySide::Right => {
            PhysicalPosition::new(main_position.x + main_size.width as i32 + gap, centered_y)
        }
    };

    [0, 1, 2, 3]
        .map(|index| {
            candidate(
                order[index],
                ideal(order[index]),
                overlay_size,
                area,
                safe_area,
                main_position,
                main_size,
                index as u8,
            )
        })
        .into_iter()
        .max_by_key(score)
        .expect("overlay candidates exist")
        .placement
}

fn candidate(
    side: OverlaySide,
    ideal: PhysicalPosition<i32>,
    overlay_size: PhysicalSize<u32>,
    area: Bounds,
    safe_area: Bounds,
    main_position: PhysicalPosition<i32>,
    main_size: PhysicalSize<u32>,
    order: u8,
) -> Candidate {
    let position = clamp_position(ideal.x, ideal.y, overlay_size, area);
    Candidate {
        placement: OverlayPlacement { position, side },
        visible_area: visible_area(position, overlay_size, area),
        overlap_area: overlap_area(position, overlay_size, main_position, main_size),
        full_fit: within_area(ideal, overlay_size, area),
        safe_fit: within_area(ideal, overlay_size, safe_area),
        edge_clearance: edge_clearance(position, overlay_size, area),
        clamp_distance: (position.x - ideal.x).unsigned_abs()
            + (position.y - ideal.y).unsigned_abs(),
        order,
    }
}

fn score(candidate: &Candidate) -> (u8, u8, u64, u64, i32, u32, u8) {
    (
        u8::from(candidate.safe_fit),
        u8::from(candidate.full_fit),
        candidate.visible_area,
        u64::MAX - candidate.overlap_area,
        candidate.edge_clearance,
        u32::MAX - candidate.clamp_distance,
        u8::MAX - candidate.order,
    )
}

fn visible_area(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> u64 {
    let right = area.x + area.width as i32;
    let bottom = area.y + area.height as i32;
    let width = (position.x + size.width as i32).min(right) - position.x.max(area.x);
    let height = (position.y + size.height as i32).min(bottom) - position.y.max(area.y);
    if width <= 0 || height <= 0 {
        0
    } else {
        width as u64 * height as u64
    }
}

fn overlap_area(
    first_position: PhysicalPosition<i32>,
    first_size: PhysicalSize<u32>,
    second_position: PhysicalPosition<i32>,
    second_size: PhysicalSize<u32>,
) -> u64 {
    let width = (first_position.x + first_size.width as i32)
        .min(second_position.x + second_size.width as i32)
        - first_position.x.max(second_position.x);
    let height = (first_position.y + first_size.height as i32)
        .min(second_position.y + second_size.height as i32)
        - first_position.y.max(second_position.y);
    if width <= 0 || height <= 0 {
        0
    } else {
        width as u64 * height as u64
    }
}

fn inset_bounds(area: Bounds, margin: i32) -> Bounds {
    let horizontal = margin.max(0).min(area.width.saturating_sub(1) as i32 / 2);
    let vertical = margin.max(0).min(area.height.saturating_sub(1) as i32 / 2);
    Bounds {
        x: area.x + horizontal,
        y: area.y + vertical,
        width: area.width.saturating_sub((horizontal * 2) as u32),
        height: area.height.saturating_sub((vertical * 2) as u32),
    }
}

fn within_area(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> bool {
    position.x >= area.x
        && position.y >= area.y
        && position.x + size.width as i32 <= area.x + area.width as i32
        && position.y + size.height as i32 <= area.y + area.height as i32
}

fn edge_clearance(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, area: Bounds) -> i32 {
    let right = area.x + area.width as i32 - (position.x + size.width as i32);
    let bottom = area.y + area.height as i32 - (position.y + size.height as i32);
    (position.x - area.x)
        .min(position.y - area.y)
        .min(right)
        .min(bottom)
}
