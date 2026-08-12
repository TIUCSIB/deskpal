use super::*;

#[test]
fn old_fixed_time_settings_default_to_daily_repeat() {
    let reminder: Reminder = serde_json::from_str(
        r#"{
        "id":"water", "enabled":true, "message":"喝水",
        "schedule":{"type":"fixed_time","time":"09:30"}, "snooze_minutes":5
    }"#,
    )
    .expect("old reminder parses");
    assert_eq!(
        reminder.schedule,
        ReminderSchedule::FixedTime {
            time: "09:30".to_string(),
            repeat: ReminderRepeat::Daily
        }
    );
}

#[test]
fn quiet_hours_defaults_are_disabled_with_suggested_times() {
    let settings = AppSettings::default();
    assert_eq!(
        settings.quiet_hours,
        QuietHours {
            enabled: false,
            start: "23:00".to_string(),
            end: "08:00".to_string()
        }
    );
}

#[test]
fn left_click_passthrough_defaults_to_disabled() {
    let settings = AppSettings::default();
    assert!(!settings.main_window_left_click_passthrough);
}

#[test]
fn custom_weekdays_serialize_as_iso_days() {
    let schedule = ReminderSchedule::FixedTime {
        time: "09:00".to_string(),
        repeat: ReminderRepeat::CustomWeekdays {
            weekdays: vec![1, 5, 7],
        },
    };
    let value = serde_json::to_value(schedule).expect("serialize schedule");
    assert_eq!(value["repeat"]["weekdays"], serde_json::json!([1, 5, 7]));
}

#[test]
fn invalid_quiet_times_are_normalized() {
    assert_eq!(
        normalize_quiet_hours(QuietHours {
            enabled: true,
            start: "25:00".to_string(),
            end: "08:99".to_string()
        })
        .start,
        "09:00"
    );
}

#[test]
fn custom_weekdays_are_canonicalized_before_persisting() {
    let reminder = normalize::normalize_reminder(
        Reminder {
            id: "weekday".to_string(),
            enabled: true,
            message: "提醒".to_string(),
            schedule: ReminderSchedule::FixedTime {
                time: "9:05".to_string(),
                repeat: ReminderRepeat::CustomWeekdays {
                    weekdays: vec![7, 3, 3, 0, 8, 1],
                },
            },
            snooze_minutes: 5,
            paused_until: None,
            snoozed_until: None,
            fired_at: None,
            completed_at: None,
        },
        0,
    );

    assert_eq!(
        reminder.schedule,
        ReminderSchedule::FixedTime {
            time: "09:05".to_string(),
            repeat: ReminderRepeat::CustomWeekdays {
                weekdays: vec![1, 3, 7],
            },
        }
    );
}

#[test]
fn once_schedule_round_trips_rfc3339_time() {
    let at = "2026-07-27T09:30:00+08:00";
    let reminder: Reminder = serde_json::from_str(
        &format!(
            r#"{{
                "id":"once", "enabled":true, "message":"单次提醒",
                "schedule":{{"type":"once","at":"{at}"}}, "snooze_minutes":5
            }}"#
        ),
    )
    .expect("once reminder parses");

    assert_eq!(
        reminder.schedule,
        ReminderSchedule::Once {
            at: at.to_string(),
        }
    );
}
