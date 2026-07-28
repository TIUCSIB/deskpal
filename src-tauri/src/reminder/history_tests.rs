use chrono::{DateTime, Local};

use super::*;

fn event(kind: ReminderEventKind, id: &str, time: DateTime<Local>) -> ReminderEvent {
    ReminderEvent {
        id: "test".to_string(),
        reminder_id: id.to_string(),
        message: "喝水".to_string(),
        scheduled_for: time.to_rfc3339(),
        occurred_at: time.to_rfc3339(),
        kind,
        reason: None,
    }
}

#[test]
fn completion_rate_is_null_without_shown_occurrence() {
    assert_eq!(
        activity(&ReminderHistory::default(), Local::now(), false)
            .stats
            .today_completion_rate,
        None
    );
}

#[test]
fn completion_rate_counts_distinct_shown_occurrences() {
    let now = Local::now();
    let history = ReminderHistory {
        events: vec![
            event(ReminderEventKind::Shown, "a", now),
            event(ReminderEventKind::Completed, "a", now),
            event(ReminderEventKind::Completed, "a", now),
        ],
    };
    assert_eq!(
        activity(&history, now, false).stats.today_completion_rate,
        Some(1.0)
    );
}

#[test]
fn streak_skips_empty_days_but_stops_at_a_past_incomplete_day() {
    let today = Local::now();
    let completed_day = today - chrono::Duration::days(2);
    let incomplete_day = today - chrono::Duration::days(4);
    let earlier_day = today - chrono::Duration::days(5);
    let history = ReminderHistory {
        events: vec![
            event(ReminderEventKind::Shown, "a", completed_day),
            event(ReminderEventKind::Completed, "a", completed_day),
            event(ReminderEventKind::Shown, "a", incomplete_day),
            event(ReminderEventKind::Shown, "a", earlier_day),
            event(ReminderEventKind::Completed, "a", earlier_day),
        ],
    };

    assert_eq!(
        activity(&history, today, false).stats.current_streak_days,
        1
    );
}

#[test]
fn pending_today_does_not_erase_a_completed_history_streak() {
    let today = Local::now();
    let previous_day = today - chrono::Duration::days(1);
    let history = ReminderHistory {
        events: vec![
            event(ReminderEventKind::Shown, "today", today),
            event(ReminderEventKind::Shown, "yesterday", previous_day),
            event(ReminderEventKind::Completed, "yesterday", previous_day),
        ],
    };

    assert_eq!(
        activity(&history, today, false).stats.current_streak_days,
        1
    );
}

#[test]
fn postponed_ranking_only_counts_recent_manual_snoozes() {
    let now = Local::now();
    let old = now - chrono::Duration::days(31);
    let history = ReminderHistory {
        events: vec![
            event(ReminderEventKind::Snoozed, "water", now),
            event(ReminderEventKind::Snoozed, "water", now),
            event(ReminderEventKind::Snoozed, "rest", now),
            event(ReminderEventKind::QuietDeferred, "rest", now),
            event(ReminderEventKind::Snoozed, "old", old),
        ],
    };

    assert_eq!(
        activity(&history, now, false).stats.frequently_postponed,
        vec![
            ReminderRanking {
                reminder_id: "water".to_string(),
                message: "喝水".to_string(),
                snooze_count: 2,
            },
            ReminderRanking {
                reminder_id: "rest".to_string(),
                message: "喝水".to_string(),
                snooze_count: 1,
            },
        ]
    );
}

#[test]
fn activity_limits_recent_events_until_all_events_are_requested() {
    let now = Local::now();
    let history = ReminderHistory {
        events: (0..51)
            .map(|index| event(ReminderEventKind::Shown, &format!("{index}"), now))
            .collect(),
    };

    let recent = activity(&history, now, false);
    assert_eq!(recent.events.len(), 50);
    assert!(recent.has_more_events);

    let all = activity(&history, now, true);
    assert_eq!(all.events.len(), 51);
    assert!(!all.has_more_events);
}

#[test]
fn history_pruning_removes_events_older_than_ninety_days() {
    let now = Local::now();
    let mut history = ReminderHistory {
        events: vec![
            event(
                ReminderEventKind::Shown,
                "recent",
                now - chrono::Duration::days(89),
            ),
            event(
                ReminderEventKind::Shown,
                "expired",
                now - chrono::Duration::days(91),
            ),
        ],
    };

    prune(&mut history, now);

    assert_eq!(history.events.len(), 1);
    assert_eq!(history.events[0].reminder_id, "recent");
}
