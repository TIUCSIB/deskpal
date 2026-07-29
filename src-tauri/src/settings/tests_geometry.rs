use std::{fs, path::PathBuf, sync::Mutex, time::SystemTime};

use super::normalize::parse_settings;
use super::state::SettingsData;
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

fn interval_input(message: &str, minutes: u32) -> ReminderInput {
    ReminderInput {
        id: None,
        message: message.to_string(),
        schedule: ReminderSchedule::Interval {
            interval_minutes: minutes,
        },
        snooze_minutes: DEFAULT_REMINDER_SNOOZE_MINUTES,
    }
}

#[test]
fn flush_pending_geometry_persists_recent_throttled_updates() {
    let state = test_state("flush-pending-geometry");
    let first = state
        .save_main_position_throttled(SavedPosition { x: 12, y: 34 })
        .expect("save main position");
    assert_eq!(first.main_position, Some(SavedPosition { x: 12, y: 34 }));

    let second = state
        .save_settings_window_bounds_throttled(SavedWindowBounds {
            x: 20,
            y: 40,
            width: 480,
            height: 540,
        })
        .expect("save settings bounds");
    assert_eq!(
        second.settings_window_bounds,
        Some(SavedWindowBounds {
            x: 20,
            y: 40,
            width: 480,
            height: 540,
        })
    );

    let flushed = state
        .flush_pending_geometry()
        .expect("flush pending geometry");
    let saved = fs::read_to_string(&state.path).expect("read saved settings");
    let parsed = parse_settings(&saved).expect("parse saved settings");

    assert_eq!(flushed.main_position, Some(SavedPosition { x: 12, y: 34 }));
    assert_eq!(
        flushed.settings_window_bounds,
        Some(SavedWindowBounds {
            x: 20,
            y: 40,
            width: 480,
            height: 540,
        })
    );
    assert_eq!(parsed.main_position, flushed.main_position);
    assert_eq!(
        parsed.settings_window_bounds,
        flushed.settings_window_bounds
    );
    let _ = fs::remove_file(state.path);
}

#[test]
fn creating_duplicate_reminders_assigns_unique_ids() {
    let state = test_state("reminder-ids");
    state
        .create_reminder(ReminderInput {
            id: Some("water".to_string()),
            ..interval_input("喝水", 30)
        })
        .expect("create first reminder");
    let updated = state
        .create_reminder(ReminderInput {
            id: Some("water".to_string()),
            ..interval_input("休息", 45)
        })
        .expect("create second reminder");

    assert_eq!(updated.reminders[0].id, "water");
    assert_eq!(updated.reminders[1].id, "water-2");
    let _ = fs::remove_file(state.path);
}
