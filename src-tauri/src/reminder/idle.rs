use std::collections::HashMap;

use chrono::{DateTime, Duration, Local};

use crate::settings::{AppSettings, ReminderSchedule};

pub(crate) const IDLE_PAUSE_THRESHOLD_SECONDS: u64 = 5 * 60;

#[derive(Default)]
pub(crate) struct IntervalPause {
    frozen_at: Option<DateTime<Local>>,
}

impl IntervalPause {
    /// 根据系统空闲状态冻结或恢复间隔提醒的倒计时。
    pub(crate) fn reconcile(
        &mut self,
        now: DateTime<Local>,
        idle_secs: Option<u64>,
        settings: &AppSettings,
        next_due_at: &mut HashMap<String, DateTime<Local>>,
    ) -> bool {
        if let Some(idle_secs) =
            idle_secs.filter(|seconds| *seconds >= IDLE_PAUSE_THRESHOLD_SECONDS)
        {
            let idle_duration = Duration::seconds(idle_secs.min(i64::MAX as u64) as i64);
            self.frozen_at.get_or_insert(now - idle_duration);
            return true;
        }

        let Some(frozen_at) = self.frozen_at.take() else {
            return false;
        };
        let paused_for = now.signed_duration_since(frozen_at);
        if paused_for <= Duration::zero() {
            return false;
        }

        for reminder in settings.reminders.iter().filter(|reminder| {
            reminder.enabled && matches!(&reminder.schedule, ReminderSchedule::Interval { .. })
        }) {
            if let Some(due_at) = next_due_at.get_mut(&reminder.id) {
                *due_at += paused_for;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::settings::{AppSettings, Reminder, ReminderRepeat};

    fn at(hour: u32, minute: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 7, 27, hour, minute, 0)
            .earliest()
            .expect("valid local time")
    }

    fn reminders() -> AppSettings {
        AppSettings {
            reminders: vec![
                Reminder {
                    id: "interval".into(),
                    enabled: true,
                    message: "间隔提醒".into(),
                    schedule: ReminderSchedule::Interval {
                        interval_minutes: 30,
                    },
                    snooze_minutes: 5,
                    paused_until: None,
                },
                Reminder {
                    id: "fixed".into(),
                    enabled: true,
                    message: "固定提醒".into(),
                    schedule: ReminderSchedule::FixedTime {
                        time: "10:30".into(),
                        repeat: ReminderRepeat::Daily,
                    },
                    snooze_minutes: 5,
                    paused_until: None,
                },
            ],
            ..AppSettings::default()
        }
    }

    #[test]
    fn does_not_freeze_before_five_minutes() {
        let mut pause = IntervalPause::default();
        let settings = reminders();
        let mut next_due = HashMap::new();

        assert!(!pause.reconcile(
            at(10, 0),
            Some(IDLE_PAUSE_THRESHOLD_SECONDS - 1),
            &settings,
            &mut next_due,
        ));
    }

    #[test]
    fn resume_moves_only_interval_due_times_by_the_paused_duration() {
        let mut pause = IntervalPause::default();
        let settings = reminders();
        let mut next_due = HashMap::from([
            ("interval".into(), at(10, 20)),
            ("fixed".into(), at(10, 30)),
        ]);

        assert!(pause.reconcile(
            at(10, 5),
            Some(IDLE_PAUSE_THRESHOLD_SECONDS),
            &settings,
            &mut next_due,
        ));
        assert!(pause.reconcile(
            at(10, 15),
            Some(IDLE_PAUSE_THRESHOLD_SECONDS + 10),
            &settings,
            &mut next_due,
        ));
        assert!(!pause.reconcile(at(10, 30), Some(0), &settings, &mut next_due));

        assert_eq!(next_due["interval"].time(), at(10, 50).time());
        assert_eq!(next_due["fixed"].time(), at(10, 30).time());
    }

    #[test]
    fn frozen_state_survives_scheduler_checks() {
        let mut pause = IntervalPause::default();
        let settings = reminders();
        let mut next_due = HashMap::new();

        assert!(pause.reconcile(
            at(10, 0),
            Some(IDLE_PAUSE_THRESHOLD_SECONDS),
            &settings,
            &mut next_due,
        ));
        assert!(pause.reconcile(
            at(10, 1),
            Some(IDLE_PAUSE_THRESHOLD_SECONDS),
            &settings,
            &mut next_due,
        ));
    }
}
