mod actions;
mod history;
mod idle;
mod schedule;

pub use actions::{
    active_payload, complete, dismiss, pause_all_until_tomorrow,
    pause_enabled_reminder_until_tomorrow, pause_until_tomorrow, preview, remove_reminder, snooze,
    start_scheduler, sync_from_settings,
};
pub use history::{ReminderActivity, ReminderEventKind, ReminderHistoryState};

use crate::settings::{AppSettings, Reminder};
use chrono::{DateTime, Local};
use idle::IntervalPause;
use schedule::{is_paused, next_due, quiet_end, schedule_signature};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Mutex,
};

#[derive(Clone, Debug, Serialize)]
pub struct ReminderPayload {
    pub reminder_id: String,
    pub message: String,
    pub snooze_minutes: u32,
}
#[derive(Clone, Debug)]
pub(crate) struct ActiveReminder {
    pub(crate) payload: ReminderPayload,
    pub(crate) scheduled_at: DateTime<Local>,
    pub(crate) preview: bool,
    shown_logged: bool,
}
#[derive(Default)]
struct ReminderData {
    active: Option<ActiveReminder>,
    queued: VecDeque<ActiveReminder>,
    next_due_at: HashMap<String, DateTime<Local>>,
    deferred_scheduled_at: HashMap<String, DateTime<Local>>,
    schedule_signatures: HashMap<String, String>,
    interval_pause: IntervalPause,
}
#[derive(Default)]
pub struct ReminderState {
    inner: Mutex<ReminderData>,
}

impl ReminderState {
    pub fn configure(&self, settings: &AppSettings) -> Result<(), String> {
        let now = Local::now();
        let mut data = self.lock()?;
        let valid: HashSet<_> = settings
            .reminders
            .iter()
            .filter(|r| r.enabled && !is_paused(r, now))
            .map(|r| r.id.clone())
            .collect();
        data.next_due_at.retain(|id, _| valid.contains(id));
        data.deferred_scheduled_at
            .retain(|id, _| valid.contains(id));
        data.schedule_signatures.retain(|id, _| valid.contains(id));
        data.queued
            .retain(|item| valid.contains(&item.payload.reminder_id));
        if data
            .active
            .as_ref()
            .is_some_and(|item| !item.preview && !valid.contains(&item.payload.reminder_id))
        {
            data.active = None;
        }
        for reminder in &settings.reminders {
            let signature = schedule_signature(reminder);
            if !reminder.enabled {
                continue;
            }
            if is_paused(reminder, now) {
                data.next_due_at.remove(&reminder.id);
                data.schedule_signatures
                    .insert(reminder.id.clone(), signature);
                continue;
            }
            if data.schedule_signatures.get(&reminder.id) != Some(&signature)
                || !data.next_due_at.contains_key(&reminder.id)
            {
                data.next_due_at
                    .insert(reminder.id.clone(), next_due(reminder, now)?);
                data.schedule_signatures
                    .insert(reminder.id.clone(), signature);
            }
        }
        Ok(())
    }
    pub(crate) fn reconcile_interval_pause(
        &self,
        settings: &AppSettings,
        idle_secs: Option<u64>,
    ) -> Result<bool, String> {
        let now = Local::now();
        let mut data = self.lock()?;
        let mut interval_pause = std::mem::take(&mut data.interval_pause);
        let intervals_paused =
            interval_pause.reconcile(now, idle_secs, settings, &mut data.next_due_at);
        data.interval_pause = interval_pause;
        Ok(intervals_paused)
    }
    pub fn active_payload(&self) -> Result<Option<ReminderPayload>, String> {
        Ok(self
            .lock()?
            .active
            .as_ref()
            .map(|item| item.payload.clone()))
    }
    pub(crate) fn active(&self) -> Result<Option<ActiveReminder>, String> {
        Ok(self.lock()?.active.clone())
    }
    pub fn preview(&self, reminder: &Reminder) -> Result<ReminderPayload, String> {
        let mut data = self.lock()?;
        if data.active.is_some() {
            return Err("当前已有提醒正在显示，请先处理后再测试".to_string());
        }
        let payload = payload(reminder);
        data.active = Some(ActiveReminder {
            payload: payload.clone(),
            scheduled_at: Local::now(),
            preview: true,
            shown_logged: true,
        });
        Ok(payload)
    }
    pub(crate) fn collect_due(
        &self,
        settings: &AppSettings,
        intervals_paused: bool,
    ) -> Result<(Option<ReminderPayload>, Vec<ActiveReminder>), String> {
        let now = Local::now();
        let mut data = self.lock()?;
        let active = data
            .active
            .as_ref()
            .map(|item| item.payload.reminder_id.clone());
        let queued: HashSet<_> = data
            .queued
            .iter()
            .map(|item| item.payload.reminder_id.clone())
            .collect();
        let mut due = Vec::new();
        let mut deferred = Vec::new();
        for (index, reminder) in settings.reminders.iter().enumerate() {
            if !reminder.enabled
                || is_paused(reminder, now)
                || (intervals_paused
                    && matches!(
                        &reminder.schedule,
                        crate::settings::ReminderSchedule::Interval { .. }
                    ))
                || active.as_deref() == Some(&reminder.id)
                || queued.contains(&reminder.id)
            {
                continue;
            }
            let scheduled_at = *data
                .next_due_at
                .entry(reminder.id.clone())
                .or_insert(next_due(reminder, now)?);
            if scheduled_at > now {
                continue;
            }
            if let Some(end) = quiet_end(&settings.quiet_hours, now)? {
                data.next_due_at.insert(reminder.id.clone(), end);
                let original_scheduled_at = *data
                    .deferred_scheduled_at
                    .entry(reminder.id.clone())
                    .or_insert(scheduled_at);
                deferred.push(ActiveReminder {
                    payload: payload(reminder),
                    scheduled_at: original_scheduled_at,
                    preview: false,
                    shown_logged: true,
                });
                continue;
            }

            data.next_due_at
                .insert(reminder.id.clone(), next_due(reminder, now)?);
            let original_scheduled_at = data
                .deferred_scheduled_at
                .remove(&reminder.id)
                .unwrap_or(scheduled_at);
            due.push((
                scheduled_at,
                index,
                ActiveReminder {
                    payload: payload(reminder),
                    scheduled_at: original_scheduled_at,
                    preview: false,
                    shown_logged: false,
                },
            ));
        }
        due.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        data.queued.extend(due.into_iter().map(|(_, _, item)| item));
        if data.active.is_none() {
            data.active = data.queued.pop_front();
        }
        Ok((
            data.active.as_ref().map(|item| item.payload.clone()),
            deferred,
        ))
    }
    pub(crate) fn finish(
        &self,
        reminder: &Reminder,
        snooze: bool,
        snooze_until: Option<DateTime<Local>>,
    ) -> Result<Option<ReminderPayload>, String> {
        let mut data = self.lock()?;
        let active = data
            .active
            .take()
            .ok_or_else(|| "提醒已更新，请重新操作".to_string())?;
        if !active.preview && reminder.enabled {
            let due = if snooze {
                data.deferred_scheduled_at
                    .insert(reminder.id.clone(), active.scheduled_at);
                snooze_until.unwrap_or_else(|| {
                    Local::now() + chrono::Duration::minutes(i64::from(reminder.snooze_minutes))
                })
            } else {
                data.deferred_scheduled_at.remove(&reminder.id);
                next_due(reminder, Local::now())?
            };
            data.next_due_at.insert(reminder.id.clone(), due);
        }
        data.active = data.queued.pop_front();
        Ok(data.active.as_ref().map(|item| item.payload.clone()))
    }
    pub fn remove_active(&self, id: &str) -> Result<Option<ReminderPayload>, String> {
        let mut data = self.lock()?;
        if data
            .active
            .as_ref()
            .is_some_and(|item| item.payload.reminder_id == id)
        {
            data.active = None;
        }
        data.queued.retain(|item| item.payload.reminder_id != id);
        data.next_due_at.remove(id);
        data.deferred_scheduled_at.remove(id);
        data.schedule_signatures.remove(id);
        if data.active.is_none() {
            data.active = data.queued.pop_front();
        }
        Ok(data.active.as_ref().map(|item| item.payload.clone()))
    }
    pub(crate) fn clear_active_and_queue(&self) -> Result<Vec<ActiveReminder>, String> {
        let mut data = self.lock()?;
        let mut cleared = data.active.take().into_iter().collect::<Vec<_>>();
        cleared.extend(data.queued.drain(..));
        data.deferred_scheduled_at.clear();
        Ok(cleared)
    }
    pub(crate) fn take_shown_event(&self) -> Result<Option<ActiveReminder>, String> {
        let mut data = self.lock()?;
        let Some(active) = data.active.as_mut() else {
            return Ok(None);
        };
        if active.preview || active.shown_logged {
            return Ok(None);
        }
        active.shown_logged = true;
        Ok(Some(active.clone()))
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ReminderData>, String> {
        self.inner
            .lock()
            .map_err(|_| "提醒状态暂时不可用".to_string())
    }
}
pub(crate) fn payload(reminder: &Reminder) -> ReminderPayload {
    ReminderPayload {
        reminder_id: reminder.id.clone(),
        message: reminder.message.clone(),
        snooze_minutes: reminder.snooze_minutes,
    }
}
