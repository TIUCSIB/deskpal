use std::collections::HashSet;

use super::state::{LegacyReminderSettings, StoredSettings};
use super::{
    default_reminder_message, normalize_pause, normalize_time, Reminder, ReminderRepeat,
    ReminderSchedule, SavedWindowBounds, MIN_SETTINGS_WINDOW_HEIGHT, MIN_SETTINGS_WINDOW_WIDTH,
};

const LEGACY_REMINDER_ID: &str = "legacy-default";

pub(crate) fn parse_settings(content: &str) -> Option<super::AppSettings> {
    let value = serde_json::from_str::<serde_json::Value>(content).ok()?;
    let has_reminders = value.as_object()?.contains_key("reminders");
    let stored = serde_json::from_value::<StoredSettings>(value).ok()?;
    let mut settings = stored.settings;
    match settings.schema_version {
        0 => {
            if !has_reminders {
                settings.reminders = stored.reminder.map(legacy_reminder).into_iter().collect();
            }
            settings.schema_version = super::SETTINGS_SCHEMA_VERSION;
        }
        super::SETTINGS_SCHEMA_VERSION => {}
        _ => return None,
    }
    Some(settings)
}
pub(crate) fn normalize_pet_role(role: String) -> String {
    match role.as_str() {
        "guga" | "monthly-salary-cat" | "broom-witch" => role,
        _ => super::default_pet_role(),
    }
}
pub(crate) fn normalize_bounds(bounds: SavedWindowBounds) -> SavedWindowBounds {
    SavedWindowBounds {
        width: bounds.width.max(MIN_SETTINGS_WINDOW_WIDTH),
        height: bounds.height.max(MIN_SETTINGS_WINDOW_HEIGHT),
        ..bounds
    }
}
pub(crate) fn normalize_reminder(mut reminder: Reminder, index: usize) -> Reminder {
    reminder.id = if reminder.id.trim().is_empty() {
        format!("reminder-{}", index + 1)
    } else {
        reminder.id.trim().to_string()
    };
    reminder.message = if reminder.message.trim().is_empty() {
        default_reminder_message()
    } else {
        reminder.message.trim().to_string()
    };
    reminder.schedule = normalize_schedule(reminder.schedule);
    reminder.snooze_minutes = reminder.snooze_minutes.max(1);
    reminder.paused_until = normalize_pause(reminder.paused_until);
    reminder
}
pub(crate) fn normalize_reminders(reminders: Vec<Reminder>) -> Vec<Reminder> {
    let mut ids = HashSet::new();
    reminders
        .into_iter()
        .enumerate()
        .map(|(index, reminder)| {
            let mut reminder = normalize_reminder(reminder, index);
            reminder.id = unique_id_with_seen(&ids, &reminder.id);
            ids.insert(reminder.id.clone());
            reminder
        })
        .collect()
}
pub(crate) fn unique_id(reminders: &[Reminder], base: &str) -> String {
    let ids: HashSet<_> = reminders.iter().map(|r| r.id.as_str()).collect();
    unique_id_with_seen(&ids, base)
}
fn unique_id_with_seen<T: AsRef<str> + std::hash::Hash + Eq>(
    ids: &HashSet<T>,
    base: &str,
) -> String {
    let mut candidate = base.to_string();
    let mut suffix = 2;
    while ids.iter().any(|id| id.as_ref() == candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    candidate
}
fn normalize_schedule(schedule: ReminderSchedule) -> ReminderSchedule {
    match schedule {
        ReminderSchedule::Interval { interval_minutes } => ReminderSchedule::Interval {
            interval_minutes: interval_minutes.max(1),
        },
        ReminderSchedule::FixedTime { time, repeat } => ReminderSchedule::FixedTime {
            time: normalize_time(time),
            repeat: normalize_repeat(repeat),
        },
    }
}
fn normalize_repeat(repeat: ReminderRepeat) -> ReminderRepeat {
    match repeat {
        ReminderRepeat::CustomWeekdays { weekdays } => {
            let mut days: Vec<u8> = weekdays
                .into_iter()
                .filter(|day| (1..=7).contains(day))
                .collect();
            days.sort_unstable();
            days.dedup();
            if days.is_empty() {
                ReminderRepeat::Daily
            } else {
                ReminderRepeat::CustomWeekdays { weekdays: days }
            }
        }
        value => value,
    }
}
pub(crate) fn legacy_reminder(legacy: LegacyReminderSettings) -> Reminder {
    Reminder {
        id: LEGACY_REMINDER_ID.to_string(),
        enabled: legacy.enabled,
        message: legacy.message,
        schedule: ReminderSchedule::Interval {
            interval_minutes: legacy.interval_minutes,
        },
        snooze_minutes: legacy.snooze_minutes,
        paused_until: None,
    }
}
