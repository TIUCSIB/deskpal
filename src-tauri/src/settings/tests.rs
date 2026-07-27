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
fn default_settings_match_expected_runtime_defaults() {
    let settings = AppSettings::default();

    assert_eq!(settings.main_position, None);
    assert_eq!(settings.settings_window_bounds, None);
    assert_eq!(settings.pet_scale, DEFAULT_PET_SCALE);
    assert_eq!(settings.pet_role, DEFAULT_PET_ROLE);
    assert_eq!(settings.info_mode, InfoMode::Auto);
    assert!(!settings.size_locked);
    assert!(settings.shortcut_enabled);
    assert!(!settings.launch_at_startup);
    assert!(settings.main_window_always_on_top);
    assert!(!settings.main_window_show_in_taskbar);
    assert_eq!(settings.chat_shortcut, DEFAULT_CHAT_SHORTCUT);
    assert!(settings.reminders.is_empty());
}

#[test]
fn reset_all_restores_defaults() {
    let state = test_state("reset-all");

    state.set_pet_scale(1.1).expect("set scale");
    state
        .set_pet_role("broom-witch".to_string())
        .expect("set pet role");
    state.set_info_mode(InfoMode::Hidden).expect("set mode");
    state
        .set_shortcut_enabled(false)
        .expect("set shortcut state");
    state
        .set_chat_shortcut("Ctrl+Shift+P".to_string())
        .expect("set shortcut");
    state
        .create_reminder(interval_input("起来接水", 20))
        .expect("create reminder");

    let reset = state.reset_all().expect("reset all");

    assert_eq!(reset.pet_scale, DEFAULT_PET_SCALE);
    assert_eq!(reset.pet_role, DEFAULT_PET_ROLE);
    assert_eq!(reset.info_mode, InfoMode::Auto);
    assert!(reset.shortcut_enabled);
    assert_eq!(reset.chat_shortcut, DEFAULT_CHAT_SHORTCUT);
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

#[test]
fn invalid_pet_role_falls_back_to_default() {
    let state = test_state("pet-role-normalize");

    let updated = state
        .set_pet_role("unknown-role".to_string())
        .expect("normalize role");

    assert_eq!(updated.pet_role, DEFAULT_PET_ROLE);
    let _ = fs::remove_file(state.path);
}

#[test]
fn reminders_normalize_schedule_and_message() {
    let state = test_state("reminder-normalize");

    let created = state
        .create_reminder(ReminderInput {
            id: Some("daily".to_string()),
            message: "   ".to_string(),
            schedule: ReminderSchedule::FixedTime {
                time: "32:99".to_string(),
            },
            snooze_minutes: 0,
        })
        .expect("create reminder");
    let reminder = created.reminders.first().expect("saved reminder");

    assert_eq!(reminder.message, DEFAULT_REMINDER_MESSAGE);
    assert_eq!(reminder.snooze_minutes, 1);
    assert_eq!(
        reminder.schedule,
        ReminderSchedule::FixedTime {
            time: "09:00".to_string()
        }
    );
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

    assert_eq!(updated.reminders.len(), 2);
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
    let stored: StoredSettings = serde_json::from_str(legacy).expect("legacy settings parse");
    let mut settings = migrate_stored_settings(stored, false);
    settings.reminders = normalize_reminders(settings.reminders);

    assert_eq!(settings.reminders.len(), 1);
    assert_eq!(settings.reminders[0].id, LEGACY_REMINDER_ID);
    assert!(settings.reminders[0].enabled);
    assert_eq!(settings.reminders[0].message, "起来活动");
    assert_eq!(
        settings.reminders[0].schedule,
        ReminderSchedule::Interval {
            interval_minutes: 45
        }
    );
}

#[test]
fn current_empty_reminders_list_does_not_restore_legacy_reminder() {
    let stored: StoredSettings = serde_json::from_str(
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

    let settings = migrate_stored_settings(stored, true);

    assert!(settings.reminders.is_empty());
}
