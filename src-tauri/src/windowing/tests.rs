use super::placement::{
    anchored_resize_position, bubble_placement, info_placement, Bounds, OverlaySide,
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
    let position = bubble_placement(
        PhysicalPosition::new(300, 30),
        PhysicalSize::new(160, 200),
        PhysicalSize::new(340, 180),
        AREA,
    );

    assert_eq!(position.position, PhysicalPosition::new(210, 234));
}

#[test]
fn bubble_prefers_above_when_it_fits() {
    let placement = bubble_placement(
        PhysicalPosition::new(800, 700),
        PhysicalSize::new(160, 200),
        PhysicalSize::new(340, 180),
        AREA,
    );

    assert_eq!(placement.side, OverlaySide::Above);
    assert_eq!(placement.position, PhysicalPosition::new(710, 516));
}

#[test]
fn bubble_falls_back_to_right_when_vertical_space_is_unavailable() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 1000,
        height: 300,
    };
    let placement = bubble_placement(
        PhysicalPosition::new(300, 90),
        PhysicalSize::new(160, 120),
        PhysicalSize::new(300, 220),
        area,
    );

    assert_eq!(placement.side, OverlaySide::Right);
    assert_eq!(placement.position, PhysicalPosition::new(464, 40));
}

#[test]
fn info_placement_keeps_negative_monitor_coordinates() {
    let area = Bounds {
        x: -1600,
        y: -200,
        width: 1600,
        height: 900,
    };
    let placement = info_placement(
        PhysicalPosition::new(-220, 260),
        PhysicalSize::new(160, 200),
        PhysicalSize::new(240, 180),
        area,
    );

    assert_eq!(placement.side, OverlaySide::Left);
    assert_eq!(placement.position, PhysicalPosition::new(-470, 270));
}

#[test]
fn bubble_clamps_an_oversized_overlay_to_the_work_area() {
    let area = Bounds {
        x: -400,
        y: 100,
        width: 500,
        height: 300,
    };
    let placement = bubble_placement(
        PhysicalPosition::new(-200, 180),
        PhysicalSize::new(100, 100),
        PhysicalSize::new(700, 400),
        area,
    );

    assert_eq!(placement.position, PhysicalPosition::new(-400, 100));
}

#[test]
fn info_moves_to_left_at_right_edge() {
    let position = info_placement(
        PhysicalPosition::new(1760, 600),
        PhysicalSize::new(160, 200),
        PhysicalSize::new(240, 180),
        AREA,
    );

    assert_eq!(position.position, PhysicalPosition::new(1510, 610));
}

#[test]
fn info_prefers_left_when_right_overflows() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 500,
        height: 400,
    };
    let position = info_placement(
        PhysicalPosition::new(350, 150),
        PhysicalSize::new(120, 100),
        PhysicalSize::new(220, 140),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(120, 130));
}

#[test]
fn info_prefers_below_when_it_has_more_room() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 500,
        height: 400,
    };
    let position = info_placement(
        PhysicalPosition::new(150, 150),
        PhysicalSize::new(120, 100),
        PhysicalSize::new(300, 120),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(60, 260));
}

#[test]
fn info_large_pet_near_right_edge_prefers_left_without_touching_edge() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 1280,
        height: 720,
    };
    let position = info_placement(
        PhysicalPosition::new(1040, 360),
        PhysicalSize::new(220, 260),
        PhysicalSize::new(240, 180),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(790, 400));
}

#[test]
fn info_large_pet_near_left_edge_prefers_right_without_touching_edge() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 1280,
        height: 720,
    };
    let position = info_placement(
        PhysicalPosition::new(20, 360),
        PhysicalSize::new(220, 260),
        PhysicalSize::new(240, 180),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(250, 400));
}

#[test]
fn info_prefers_less_edge_hugging_when_all_candidates_need_clamp() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 360,
        height: 260,
    };
    let position = info_placement(
        PhysicalPosition::new(120, 90),
        PhysicalSize::new(120, 120),
        PhysicalSize::new(280, 180),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(40, 0));
}

#[test]
fn info_prefers_above_when_bottom_space_is_worse() {
    let area = Bounds {
        x: 0,
        y: 0,
        width: 520,
        height: 360,
    };
    let position = info_placement(
        PhysicalPosition::new(180, 220),
        PhysicalSize::new(120, 110),
        PhysicalSize::new(260, 120),
        area,
    );

    assert_eq!(position.position, PhysicalPosition::new(110, 90));
}
