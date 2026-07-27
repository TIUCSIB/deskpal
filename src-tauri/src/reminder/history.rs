use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use chrono::{DateTime, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const HISTORY_FILE: &str = "reminder-history.json";
const HISTORY_DAYS: i64 = 90;
const RECENT_EVENT_LIMIT: usize = 50;
static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReminderEventKind {
    Shown,
    Completed,
    Snoozed,
    Skipped,
    QuietDeferred,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderEvent {
    pub id: String,
    pub reminder_id: String,
    pub message: String,
    pub scheduled_for: String,
    pub occurred_at: String,
    pub kind: ReminderEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ReminderHistory {
    #[serde(default)]
    events: Vec<ReminderEvent>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderRanking {
    pub reminder_id: String,
    pub message: String,
    pub snooze_count: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderActivityStats {
    pub today_completion_rate: Option<f64>,
    pub current_streak_days: u32,
    pub frequently_postponed: Vec<ReminderRanking>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderActivity {
    pub stats: ReminderActivityStats,
    pub events: Vec<ReminderEvent>,
    pub has_more_events: bool,
}

pub struct ReminderHistoryState {
    path: PathBuf,
    inner: Mutex<ReminderHistory>,
}
impl ReminderHistoryState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join(HISTORY_FILE);
        let mut history = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        prune(&mut history, Local::now());
        let state = Self {
            path,
            inner: Mutex::new(history),
        };
        state.persist()?;
        Ok(state)
    }
    pub fn record(
        &self,
        kind: ReminderEventKind,
        reminder_id: String,
        message: String,
        scheduled_for: DateTime<Local>,
    ) -> Result<(), String> {
        let now = Local::now();
        let mut history = self.lock()?;
        history.events.push(ReminderEvent {
            id: event_id(&reminder_id, scheduled_for, kind, now),
            reminder_id,
            message,
            scheduled_for: scheduled_for.to_rfc3339(),
            occurred_at: now.to_rfc3339(),
            kind,
            reason: (kind == ReminderEventKind::QuietDeferred).then(|| "quiet_hours".to_string()),
        });
        prune(&mut history, now);
        let saved = history.clone();
        drop(history);
        self.write(&saved)
    }
    pub fn activity(&self) -> Result<ReminderActivity, String> {
        Ok(activity(&self.lock()?.clone(), Local::now(), false))
    }
    pub fn activity_with_events(
        &self,
        include_all_events: bool,
    ) -> Result<ReminderActivity, String> {
        Ok(activity(
            &self.lock()?.clone(),
            Local::now(),
            include_all_events,
        ))
    }
    pub fn clear(&self) -> Result<(), String> {
        let mut history = self.lock()?;
        history.events.clear();
        let saved = history.clone();
        drop(history);
        self.write(&saved)
    }
    fn persist(&self) -> Result<(), String> {
        self.write(&self.lock()?.clone())
    }
    fn write(&self, history: &ReminderHistory) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(
            &self.path,
            serde_json::to_string_pretty(history).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ReminderHistory>, String> {
        self.inner
            .lock()
            .map_err(|_| "提醒历史暂时不可用".to_string())
    }
}

fn occurrence_id(event: &ReminderEvent) -> String {
    format!("{}:{}", event.reminder_id, event.scheduled_for)
}
fn event_id(
    id: &str,
    scheduled: DateTime<Local>,
    kind: ReminderEventKind,
    now: DateTime<Local>,
) -> String {
    let sequence = EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{id}:{}:{kind:?}:{}:{sequence}",
        scheduled.timestamp_millis(),
        now.timestamp_millis()
    )
}
fn parse_time(value: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|time| time.with_timezone(&Local))
}
fn scheduled_date(event: &ReminderEvent) -> Option<NaiveDate> {
    parse_time(&event.scheduled_for).map(|time| time.date_naive())
}
fn prune(history: &mut ReminderHistory, now: DateTime<Local>) {
    let cutoff = now - Duration::days(HISTORY_DAYS);
    history
        .events
        .retain(|event| parse_time(&event.occurred_at).is_some_and(|time| time >= cutoff));
}
fn activity(
    history: &ReminderHistory,
    now: DateTime<Local>,
    include_all_events: bool,
) -> ReminderActivity {
    let today = now.date_naive();
    let mut shown = HashSet::new();
    let mut completed = HashSet::new();
    for event in &history.events {
        if scheduled_date(event) == Some(today) {
            match event.kind {
                ReminderEventKind::Shown => {
                    shown.insert(occurrence_id(event));
                }
                ReminderEventKind::Completed => {
                    completed.insert(occurrence_id(event));
                }
                _ => {}
            }
        }
    }
    let stats = ReminderActivityStats {
        today_completion_rate: (!shown.is_empty()).then(|| {
            shown.iter().filter(|id| completed.contains(*id)).count() as f64 / shown.len() as f64
        }),
        current_streak_days: streak(history, today),
        frequently_postponed: postponed(history, now),
    };
    let mut events = history.events.clone();
    events.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    let has_more_events = !include_all_events && events.len() > RECENT_EVENT_LIMIT;
    if !include_all_events {
        events.truncate(RECENT_EVENT_LIMIT);
    }
    ReminderActivity {
        stats,
        events,
        has_more_events,
    }
}
fn streak(history: &ReminderHistory, today: NaiveDate) -> u32 {
    let mut actionable: HashMap<NaiveDate, HashSet<String>> = HashMap::new();
    let mut completed = HashSet::new();

    for event in &history.events {
        let Some(date) = scheduled_date(event) else {
            continue;
        };
        match event.kind {
            ReminderEventKind::Shown => {
                actionable
                    .entry(date)
                    .or_default()
                    .insert(occurrence_id(event));
            }
            ReminderEventKind::Completed => {
                completed.insert(occurrence_id(event));
            }
            _ => {}
        }
    }

    let mut dates: Vec<_> = actionable
        .keys()
        .copied()
        .filter(|date| *date <= today)
        .collect();
    dates.sort_unstable_by(|left, right| right.cmp(left));

    let mut total = 0;
    for date in dates {
        let Some(items) = actionable.get(&date) else {
            continue;
        };
        if items.iter().any(|item| !completed.contains(item)) {
            if date == today {
                continue;
            }
            break;
        }
        total += 1;
    }
    total
}
fn postponed(history: &ReminderHistory, now: DateTime<Local>) -> Vec<ReminderRanking> {
    let cutoff = now - Duration::days(30);
    let mut counts: HashMap<String, (String, u32)> = HashMap::new();
    for event in &history.events {
        if event.kind == ReminderEventKind::Snoozed
            && parse_time(&event.occurred_at).is_some_and(|time| time >= cutoff)
        {
            let entry = counts
                .entry(event.reminder_id.clone())
                .or_insert((event.message.clone(), 0));
            entry.0 = event.message.clone();
            entry.1 += 1;
        }
    }
    let mut ranking: Vec<_> = counts
        .into_iter()
        .map(|(reminder_id, (message, snooze_count))| ReminderRanking {
            reminder_id,
            message,
            snooze_count,
        })
        .collect();
    ranking.sort_by(|a, b| {
        b.snooze_count
            .cmp(&a.snooze_count)
            .then(a.reminder_id.cmp(&b.reminder_id))
    });
    ranking
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
