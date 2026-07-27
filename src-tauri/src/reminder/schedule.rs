use crate::settings::{QuietHours, Reminder, ReminderRepeat, ReminderSchedule};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};

pub(crate) fn schedule_signature(reminder: &Reminder) -> String {
    format!(
        "{:?}:{}:{:?}",
        reminder.schedule, reminder.enabled, reminder.paused_until
    )
}
pub(crate) fn is_paused(reminder: &Reminder, now: DateTime<Local>) -> bool {
    reminder
        .paused_until
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|until| until.with_timezone(&Local) > now)
}
pub(crate) fn next_due(
    reminder: &Reminder,
    now: DateTime<Local>,
) -> Result<DateTime<Local>, String> {
    match &reminder.schedule {
        ReminderSchedule::Interval { interval_minutes } => {
            Ok(now + Duration::minutes(i64::from((*interval_minutes).max(1))))
        }
        ReminderSchedule::FixedTime { time, repeat } => next_fixed_time(time, repeat, now),
    }
}
pub(crate) fn quiet_end(
    hours: &QuietHours,
    now: DateTime<Local>,
) -> Result<Option<DateTime<Local>>, String> {
    if !hours.enabled {
        return Ok(None);
    }
    let start = parse_time(&hours.start)?;
    let end = parse_time(&hours.end)?;
    if start == end {
        return Ok(None);
    }
    let current = now.time();
    let within = if start < end {
        current >= start && current < end
    } else {
        current >= start || current < end
    };
    if !within {
        return Ok(None);
    }
    let date = if start < end || current < end {
        now.date_naive()
    } else {
        now.date_naive() + Duration::days(1)
    };
    Ok(Some(local_datetime(date, end)?))
}
pub(crate) fn tomorrow_start() -> DateTime<Local> {
    let now = Local::now();
    local_datetime(
        now.date_naive() + Duration::days(1),
        NaiveTime::from_hms_opt(0, 0, 0).expect("valid midnight"),
    )
    .unwrap_or(now + Duration::days(1))
}
fn next_fixed_time(
    time: &str,
    repeat: &ReminderRepeat,
    now: DateTime<Local>,
) -> Result<DateTime<Local>, String> {
    let time = parse_time(time).map_err(|_| "固定提醒时间格式无效".to_string())?;
    for offset in 0..8 {
        let date = now.date_naive() + Duration::days(offset);
        if matches_repeat(repeat, date.weekday()) {
            let candidate = local_datetime(date, time)?;
            if candidate >= now {
                return Ok(candidate);
            }
        }
    }
    Err("无法计算下一次固定提醒".to_string())
}
fn matches_repeat(repeat: &ReminderRepeat, weekday: Weekday) -> bool {
    let day = weekday.number_from_monday() as u8;
    match repeat {
        ReminderRepeat::Daily => true,
        ReminderRepeat::Weekdays => day <= 5,
        ReminderRepeat::CustomWeekdays { weekdays } => weekdays.contains(&day),
    }
}
fn parse_time(value: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| "安静时间格式无效".to_string())
}
fn local_datetime(date: NaiveDate, time: NaiveTime) -> Result<DateTime<Local>, String> {
    Local
        .from_local_datetime(&date.and_time(time))
        .earliest()
        .ok_or_else(|| "无法计算本地提醒时间".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weekdays_skip_weekend() {
        assert!(matches_repeat(&ReminderRepeat::Weekdays, Weekday::Mon));
        assert!(!matches_repeat(&ReminderRepeat::Weekdays, Weekday::Sun));
    }

    #[test]
    fn custom_weekdays_find_the_next_matching_day() {
        let now = Local
            .with_ymd_and_hms(2026, 7, 27, 10, 0, 0)
            .earliest()
            .expect("local date");
        let next = next_fixed_time(
            "09:00",
            &ReminderRepeat::CustomWeekdays { weekdays: vec![3] },
            now,
        )
        .expect("next fixed reminder");

        assert_eq!(next.weekday(), Weekday::Wed);
        assert_eq!(
            next.time(),
            NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")
        );
    }

    #[test]
    fn fixed_time_at_the_current_minute_is_due_immediately() {
        let now = Local
            .with_ymd_and_hms(2026, 7, 27, 9, 0, 0)
            .earliest()
            .expect("local date");
        assert_eq!(
            next_fixed_time("09:00", &ReminderRepeat::Daily, now).expect("next reminder"),
            now
        );
    }
    #[test]
    fn overnight_quiet_hours_are_start_inclusive_end_exclusive() {
        let hours = QuietHours {
            enabled: true,
            start: "23:00".into(),
            end: "08:00".into(),
        };
        let start = Local
            .with_ymd_and_hms(2026, 7, 27, 23, 0, 0)
            .earliest()
            .unwrap();
        let end = Local
            .with_ymd_and_hms(2026, 7, 28, 8, 0, 0)
            .earliest()
            .unwrap();
        assert!(quiet_end(&hours, start).unwrap().is_some());
        assert!(quiet_end(&hours, end).unwrap().is_none());
    }

    #[test]
    fn daytime_quiet_hours_defer_to_the_same_day_end() {
        let hours = QuietHours {
            enabled: true,
            start: "12:00".into(),
            end: "13:00".into(),
        };
        let now = Local
            .with_ymd_and_hms(2026, 7, 27, 12, 30, 0)
            .earliest()
            .expect("local date");
        let end = quiet_end(&hours, now)
            .expect("quiet end")
            .expect("in quiet hours");

        assert_eq!(end.date_naive(), now.date_naive());
        assert_eq!(
            end.time(),
            NaiveTime::from_hms_opt(13, 0, 0).expect("valid time")
        );
    }
}
