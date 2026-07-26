use std::{fs, path::PathBuf, sync::Mutex, time::SystemTime};

use super::*;

fn temp_settings_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("table-pet-{name}-{unique}.json"))
}

fn test_state(name: &str) -> SettingsState {
    SettingsState {
        path: temp_settings_path(name),
        inner: Mutex::new(SettingsData {
            settings: AppSettings::default(),
            last_position_save: None,
            last_settings_window_save: None,
        }),
    }
}

#[test]
fn default_settings_match_expected_runtime_defaults() {
    let settings = AppSettings::default();

    assert_eq!(settings.main_position, None);
    assert_eq!(settings.settings_window_bounds, None);
    assert_eq!(settings.pet_scale, DEFAULT_PET_SCALE);
    assert_eq!(settings.info_mode, InfoMode::Auto);
    assert!(!settings.size_locked);
    assert!(settings.shortcut_enabled);
    assert!(!settings.launch_at_startup);
    assert!(settings.main_window_always_on_top);
    assert!(!settings.main_window_show_in_taskbar);
    assert_eq!(settings.chat_shortcut, DEFAULT_CHAT_SHORTCUT);
}

#[test]
fn reset_all_restores_defaults() {
    let state = test_state("reset-all");

    state.set_pet_scale(1.1).expect("set scale");
    state.set_info_mode(InfoMode::Hidden).expect("set mode");
    state.set_shortcut_enabled(false).expect("set shortcut state");
    state.set_chat_shortcut("Ctrl+Shift+P".to_string()).expect("set shortcut");

    let reset = state.reset_all().expect("reset all");

    assert_eq!(reset.pet_scale, DEFAULT_PET_SCALE);
    assert_eq!(reset.info_mode, InfoMode::Auto);
    assert!(reset.shortcut_enabled);
    assert_eq!(reset.chat_shortcut, DEFAULT_CHAT_SHORTCUT);
    let _ = fs::remove_file(state.path);
}

#[test]
fn settings_window_bounds_are_clamped_before_persisting() {
    let state = test_state("window-bounds");

    let updated = state
        .save_settings_window_bounds_throttled(SavedWindowBounds {
            x: 12,
            y: 34,
            width: 12,
            height: 24,
        })
        .expect("save bounds");
    let bounds = updated.settings_window_bounds.expect("saved bounds");

    assert_eq!(bounds.x, 12);
    assert_eq!(bounds.y, 34);
    assert_eq!(bounds.width, MIN_SETTINGS_WINDOW_WIDTH);
    assert_eq!(bounds.height, MIN_SETTINGS_WINDOW_HEIGHT);
    let _ = fs::remove_file(state.path);
}

#[test]
fn reset_settings_window_bounds_clears_only_window_bounds() {
    let state = test_state("reset-window-bounds");

    state
        .save_settings_window_bounds_throttled(SavedWindowBounds {
            x: 1,
            y: 2,
            width: 800,
            height: 700,
        })
        .expect("save bounds");
    state.set_pet_scale(1.05).expect("set scale");

    let updated = state
        .reset_settings_window_bounds()
        .expect("reset window bounds");

    assert_eq!(updated.settings_window_bounds, None);
    assert_eq!(updated.pet_scale, 1.05);
    let _ = fs::remove_file(state.path);
}
