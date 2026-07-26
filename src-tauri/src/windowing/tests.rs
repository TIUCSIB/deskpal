use super::placement::{
    anchored_resize_position, chat_position, info_position, Bounds,
};
use super::{clamp_scale, MAX_PET_SCALE, MIN_PET_SCALE};
use tauri::{PhysicalPosition, PhysicalSize};

const AREA: Bounds = Bounds {
    x: 0,
    y: 0,
    width: 1920,
    height: 1040,
};

#[test]
fn scale_is_limited_to_supported_range() {
    assert_eq!(clamp_scale(0.2), MIN_PET_SCALE);
    assert_eq!(clamp_scale(0.9), 0.9);
    assert_eq!(clamp_scale(2.0), MAX_PET_SCALE);
    assert_eq!(clamp_scale(f64::NAN), 1.0);
    assert_eq!(clamp_scale(f64::INFINITY), 1.0);
}

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

    assert_eq!(position, PhysicalPosition::new(0, 234));
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

#[test]
fn info_prefers_left_when_right_overflows() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 500,
        height: 400,
    };
    let position = info_position(
        PhysicalPosition::new(350, 150),
        PhysicalSize::new(120, 100),
        PhysicalSize::new(220, 140),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(120, 130));
}

#[test]
fn info_prefers_below_when_it_has_more_room() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 500,
        height: 400,
    };
    let position = info_position(
        PhysicalPosition::new(150, 150),
        PhysicalSize::new(120, 100),
        PhysicalSize::new(300, 120),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(60, 260));
}

#[test]
fn info_large_pet_near_right_edge_prefers_left_without_touching_edge() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 1280,
        height: 720,
    };
    let position = info_position(
        PhysicalPosition::new(1040, 360),
        PhysicalSize::new(220, 260),
        PhysicalSize::new(240, 180),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(790, 400));
}

#[test]
fn info_large_pet_near_left_edge_prefers_right_without_touching_edge() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 1280,
        height: 720,
    };
    let position = info_position(
        PhysicalPosition::new(20, 360),
        PhysicalSize::new(220, 260),
        PhysicalSize::new(240, 180),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(250, 400));
}

#[test]
fn info_prefers_less_edge_hugging_when_all_candidates_need_clamp() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 360,
        height: 260,
    };
    let position = info_position(
        PhysicalPosition::new(120, 90),
        PhysicalSize::new(120, 120),
        PhysicalSize::new(280, 180),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(40, 0));
}

#[test]
fn info_prefers_above_when_bottom_space_is_worse() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 520,
        height: 360,
    };
    let position = info_position(
        PhysicalPosition::new(180, 220),
        PhysicalSize::new(120, 110),
        PhysicalSize::new(260, 120),
        area,
    );

    assert_eq!(position, PhysicalPosition::new(110, 90));
}
