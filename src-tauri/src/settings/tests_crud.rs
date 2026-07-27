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
fn reset_all_restores_defaults() {
    let state = test_state("reset-all");
    state.set_pet_scale(1.1).expect("set scale");
    state
        .set_pet_role("broom-witch".to_string())
        .expect("set pet role");
    state
        .create_reminder(interval_input("起来接水", 20))
        .expect("create reminder");

    let reset = state.reset_all().expect("reset all");

    assert_eq!(reset.pet_scale, DEFAULT_PET_SCALE);
    assert_eq!(reset.pet_role, DEFAULT_PET_ROLE);
    assert!(reset.reminders.is_empty());
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
fn updating_and_deleting_reminders_only_affects_target_item() {
    let state = test_state("reminder-crud");
    let first = state
        .create_reminder(interval_input("喝水", 30))
        .expect("create first")
        .reminders
        .into_iter()
        .next()
        .expect("first reminder");
    let second = state
        .create_reminder(interval_input("起身活动", 45))
        .expect("create second")
        .reminders
        .into_iter()
        .last()
        .expect("second reminder");

    let updated = state
        .update_reminder(Reminder {
            message: "休息一下".to_string(),
            ..first.clone()
        })
        .expect("update first");
    assert_eq!(updated.reminders[0].message, "休息一下");
    assert_eq!(updated.reminders[1].message, second.message);

    let deleted = state
        .delete_reminder(first.id)
        .expect("delete first reminder");
    assert_eq!(deleted.reminders.len(), 1);
    assert_eq!(deleted.reminders[0].id, second.id);
    let _ = fs::remove_file(state.path);
}

#[test]
fn pause_enabled_reminders_leaves_disabled_reminders_unchanged() {
    let state = test_state("pause-enabled-reminders");
    let first = state
        .create_reminder(interval_input("喝水", 30))
        .expect("create first reminder")
        .reminders
        .into_iter()
        .next()
        .expect("first reminder");
    let second = state
        .create_reminder(interval_input("休息", 45))
        .expect("create second reminder")
        .reminders
        .into_iter()
        .last()
        .expect("second reminder");
    state
        .set_reminder_enabled(second.id.clone(), false)
        .expect("disable second reminder");

    let updated = state
        .pause_enabled_reminders_until("2030-01-01T00:00:00+00:00".to_string())
        .expect("pause enabled reminders");

    assert_eq!(updated.reminders[0].id, first.id);
    let paused_until = updated.reminders[0]
        .paused_until
        .as_deref()
        .expect("enabled reminder is paused");
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(paused_until).expect("stored pause time"),
        chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00+00:00")
            .expect("expected pause time")
    );
    assert!(!updated.reminders[1].enabled);
    assert_eq!(updated.reminders[1].paused_until, None);
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

#[test]
fn legacy_single_reminder_migrates_to_list() {
    let legacy = r#"{
      "pet_scale": 0.9,
      "reminder": {
        "enabled": true,
        "message": "起来活动",
        "interval_minutes": 45,
        "snooze_minutes": 10
      }
    }"#;
    let settings = parse_settings(legacy).expect("legacy settings parse");

    assert_eq!(settings.reminders.len(), 1);
    assert_eq!(settings.reminders[0].id, "legacy-default");
    assert!(settings.reminders[0].enabled);
    assert_eq!(settings.reminders[0].message, "起来活动");
}

#[test]
fn current_empty_reminders_list_does_not_restore_legacy_reminder() {
    let settings = parse_settings(
        r#"{
          "reminders": [],
          "reminder": {
            "enabled": true,
            "message": "旧提醒",
            "interval_minutes": 30,
            "snooze_minutes": 5
          }
        }"#,
    )
    .expect("current settings parse");

    assert!(settings.reminders.is_empty());
}
