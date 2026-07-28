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
