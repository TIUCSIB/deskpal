use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
    thread,
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveTime, TimeZone};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    settings::{AppSettings, Reminder, ReminderSchedule, SettingsState},
    windowing,
};

const REMINDER_EVENT: &str = "pet://reminder-payload";
const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";
const CHECK_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Serialize)]
pub struct ReminderPayload {
    pub reminder_id: String,
    pub message: String,
    pub snooze_minutes: u32,
}

#[derive(Default)]
struct ReminderData {
    active: Option<ReminderPayload>,
    active_is_preview: bool,
    queued: VecDeque<ReminderPayload>,
    next_due_at: HashMap<String, DateTime<Local>>,
    schedule_signatures: HashMap<String, String>,
}

#[derive(Default)]
pub struct ReminderState {
    inner: Mutex<ReminderData>,
}

impl ReminderState {
    pub fn configure(&self, settings: &AppSettings) -> Result<(), String> {
        let now = Local::now();
        let mut data = self.lock()?;
        let valid_ids: std::collections::HashSet<_> = settings
            .reminders
            .iter()
            .filter(|reminder| reminder.enabled && !is_paused(reminder, now))
            .map(|reminder| reminder.id.clone())
            .collect();
        data.next_due_at.retain(|id, _| valid_ids.contains(id));
        data.schedule_signatures
            .retain(|id, _| valid_ids.contains(id));
        data.queued
            .retain(|payload| valid_ids.contains(&payload.reminder_id));

        if data.active.as_ref().is_some_and(|payload| {
            !data.active_is_preview && !valid_ids.contains(&payload.reminder_id)
        }) {
            data.active = None;
            data.active_is_preview = false;
        }

        if !data.active_is_preview {
            if let Some(active) = data.active.as_mut() {
                if let Some(reminder) = settings
                    .reminders
                    .iter()
                    .find(|reminder| reminder.id == active.reminder_id)
                {
                    active.message = reminder.message.clone();
                    active.snooze_minutes = reminder.snooze_minutes;
                }
            }
        }
        data.queued = data
            .queued
            .drain(..)
            .filter_map(|payload| {
                settings
                    .reminders
                    .iter()
                    .find(|reminder| reminder.id == payload.reminder_id)
                    .map(payload_from_reminder)
            })
            .collect();

        for reminder in &settings.reminders {
            let signature = schedule_signature(reminder);
            if !reminder.enabled {
                data.next_due_at.remove(&reminder.id);
                data.schedule_signatures.remove(&reminder.id);
                continue;
            }
            if is_paused(reminder, now) {
                data.next_due_at.remove(&reminder.id);
                data.schedule_signatures
                    .insert(reminder.id.clone(), signature);
                continue;
            }
            let changed = data.schedule_signatures.get(&reminder.id) != Some(&signature);
            if changed || !data.next_due_at.contains_key(&reminder.id) {
                data.next_due_at
                    .insert(reminder.id.clone(), next_due(reminder, now)?);
                data.schedule_signatures
                    .insert(reminder.id.clone(), signature);
            }
        }
        Ok(())
    }

    pub fn active_payload(&self) -> Result<Option<ReminderPayload>, String> {
        Ok(self.lock()?.active.clone())
    }

    fn active_is_preview(&self) -> Result<bool, String> {
        Ok(self.lock()?.active_is_preview)
    }

    pub fn preview(&self, reminder: &Reminder) -> Result<ReminderPayload, String> {
        let mut data = self.lock()?;
        if data.active.is_some() {
            return Err("当前已有提醒正在显示，请先处理后再测试".to_string());
        }
        let payload = payload_from_reminder(reminder);
        data.active = Some(payload.clone());
        data.active_is_preview = true;
        Ok(payload)
    }

    pub fn collect_due(&self, settings: &AppSettings) -> Result<Option<ReminderPayload>, String> {
        let now = Local::now();
        let mut data = self.lock()?;
        let active_id = data
            .active
            .as_ref()
            .map(|payload| payload.reminder_id.clone());
        let queued_ids: std::collections::HashSet<_> = data
            .queued
            .iter()
            .map(|payload| payload.reminder_id.clone())
            .collect();
        let mut due = Vec::new();

        for (index, reminder) in settings.reminders.iter().enumerate() {
            if !reminder.enabled || is_paused(reminder, now) {
                continue;
            }
            if !data.next_due_at.contains_key(&reminder.id) {
                data.next_due_at
                    .insert(reminder.id.clone(), next_due(reminder, now)?);
            }
            if active_id.as_deref() == Some(reminder.id.as_str())
                || queued_ids.contains(&reminder.id)
            {
                continue;
            }
            let Some(next_due_at) = data.next_due_at.get(&reminder.id).cloned() else {
                continue;
            };
            if next_due_at <= now {
                due.push((next_due_at, index, payload_from_reminder(reminder)));
                data.next_due_at
                    .insert(reminder.id.clone(), next_due(reminder, now)?);
            }
        }
        due.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        data.queued
            .extend(due.into_iter().map(|(_, _, payload)| payload));
        if data.active.is_none() {
            data.active = data.queued.pop_front();
            data.active_is_preview = false;
            return Ok(data.active.clone());
        }
        Ok(None)
    }

    pub fn dismiss(&self, reminder: &Reminder) -> Result<Option<ReminderPayload>, String> {
        let mut data = self.lock()?;
        if !data.active_is_preview && reminder.enabled {
            data.next_due_at
                .insert(reminder.id.clone(), next_due(reminder, Local::now())?);
        }
        data.active = None;
        data.active_is_preview = false;
        data.active = data.queued.pop_front();
        Ok(data.active.clone())
    }

    pub fn snooze(&self, reminder: &Reminder) -> Result<Option<ReminderPayload>, String> {
        let mut data = self.lock()?;
        if !data.active_is_preview && reminder.enabled {
            data.next_due_at.insert(
                reminder.id.clone(),
                Local::now() + ChronoDuration::minutes(i64::from(reminder.snooze_minutes)),
            );
        }
        data.active = None;
        data.active_is_preview = false;
        data.active = data.queued.pop_front();
        Ok(data.active.clone())
    }

    pub fn remove_active(&self, id: &str) -> Result<Option<ReminderPayload>, String> {
        let mut data = self.lock()?;
        if data
            .active
            .as_ref()
            .is_some_and(|payload| payload.reminder_id == id)
        {
            data.active = None;
            data.active_is_preview = false;
        }
        data.queued.retain(|payload| payload.reminder_id != id);
        data.next_due_at.remove(id);
        data.schedule_signatures.remove(id);
        if data.active.is_none() {
            data.active = data.queued.pop_front();
        }
        Ok(data.active.clone())
    }

    pub fn clear_active_and_queue(&self) -> Result<(), String> {
        let mut data = self.lock()?;
        data.active = None;
        data.active_is_preview = false;
        data.queued.clear();
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ReminderData>, String> {
        self.inner
            .lock()
            .map_err(|_| "提醒状态暂时不可用".to_string())
    }
}

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(CHECK_INTERVAL);
        if let Err(error) = check_and_fire(&app) {
            eprintln!("提醒调度失败: {error}");
        }
    });
}

pub fn sync_from_settings(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    let state = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?;
    state.configure(&settings)?;
    if let Some(payload) = state.active_payload()? {
        show_payload(app, payload)?;
    } else {
        windowing::hide_reminder_window(app)?;
    }
    Ok(())
}

pub fn active_payload(app: &AppHandle) -> Result<Option<ReminderPayload>, String> {
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .active_payload()
}

fn active_is_preview(app: &AppHandle) -> Result<bool, String> {
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .active_is_preview()
}

pub fn preview(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let settings = get_settings(app)?;
    let reminder = find_reminder(&settings, &reminder_id)?;
    let payload = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .preview(reminder)?;
    show_payload(app, payload)
}

pub fn dismiss(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let settings = get_settings(app)?;
    let reminder = find_reminder(&settings, &reminder_id)?;
    let active = active_payload(app)?;
    if active.as_ref().map(|payload| &payload.reminder_id) != Some(&reminder_id) {
        return Err("提醒已更新，请重新操作".to_string());
    }
    let next = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .dismiss(reminder)?;
    sync_next_payload(app, next)
}

pub fn snooze(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let settings = get_settings(app)?;
    let reminder = find_reminder(&settings, &reminder_id)?;
    let active = active_payload(app)?;
    if active.as_ref().map(|payload| &payload.reminder_id) != Some(&reminder_id) {
        return Err("提醒已更新，请重新操作".to_string());
    }
    let next = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .snooze(reminder)?;
    sync_next_payload(app, next)
}

pub fn pause_all_until_tomorrow(app: &AppHandle) -> Result<(), String> {
    let paused_until = tomorrow_start().to_rfc3339();
    let settings_state = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?;
    let settings = settings_state.pause_enabled_reminders_until(paused_until)?;
    let state = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?;
    state.clear_active_and_queue()?;
    state.configure(&settings)?;
    windowing::hide_reminder_window(app)?;
    app.emit(SETTINGS_UPDATED_EVENT, &settings)
        .map_err(|error| error.to_string())
}

pub fn pause_until_tomorrow(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let active = active_payload(app)?;
    if active.as_ref().map(|payload| &payload.reminder_id) != Some(&reminder_id) {
        return Err("提醒已更新，请重新操作".to_string());
    }
    if active_is_preview(app)? {
        let settings = get_settings(app)?;
        let reminder = find_reminder(&settings, &reminder_id)?;
        let next = app
            .try_state::<ReminderState>()
            .ok_or_else(|| "找不到提醒状态".to_string())?
            .dismiss(reminder)?;
        return sync_next_payload(app, next);
    }
    let paused_until = tomorrow_start().to_rfc3339();
    let settings_state = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?;
    let settings = settings_state.set_reminder_pause(reminder_id.clone(), Some(paused_until))?;
    let state = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?;
    state.configure(&settings)?;
    let next = state.remove_active(&reminder_id)?;
    sync_next_payload(app, next)?;
    app.emit(SETTINGS_UPDATED_EVENT, &settings)
        .map_err(|error| error.to_string())
}

pub fn remove_reminder(app: &AppHandle, id: &str) -> Result<(), String> {
    let next = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .remove_active(id)?;
    sync_next_payload(app, next)
}

fn check_and_fire(app: &AppHandle) -> Result<(), String> {
    let settings = get_settings(app)?;
    let payload = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .collect_due(&settings)?;
    if let Some(payload) = payload {
        show_payload(app, payload)?;
    }
    Ok(())
}

fn sync_next_payload(app: &AppHandle, payload: Option<ReminderPayload>) -> Result<(), String> {
    if let Some(payload) = payload {
        show_payload(app, payload)
    } else {
        windowing::hide_reminder_window(app)
    }
}

fn show_payload(app: &AppHandle, payload: ReminderPayload) -> Result<(), String> {
    app.emit_to(windowing::REMINDER_WINDOW, REMINDER_EVENT, payload)
        .map_err(|error| error.to_string())?;
    windowing::sync_reminder_window_visibility(app)
}

fn get_settings(app: &AppHandle) -> Result<AppSettings, String> {
    app.try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()
}

fn find_reminder<'a>(settings: &'a AppSettings, id: &str) -> Result<&'a Reminder, String> {
    settings
        .reminders
        .iter()
        .find(|reminder| reminder.id == id)
        .ok_or_else(|| "找不到该提醒".to_string())
}

fn payload_from_reminder(reminder: &Reminder) -> ReminderPayload {
    ReminderPayload {
        reminder_id: reminder.id.clone(),
        message: reminder.message.clone(),
        snooze_minutes: reminder.snooze_minutes,
    }
}

fn schedule_signature(reminder: &Reminder) -> String {
    let pause = reminder.paused_until.as_deref().unwrap_or_default();
    match &reminder.schedule {
        ReminderSchedule::Interval { interval_minutes } => {
            format!("interval:{interval_minutes}:{}:{pause}", reminder.enabled)
        }
        ReminderSchedule::FixedTime { time } => {
            format!("fixed:{time}:{}:{pause}", reminder.enabled)
        }
    }
}

fn is_paused(reminder: &Reminder, now: DateTime<Local>) -> bool {
    reminder
        .paused_until
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|until| until.with_timezone(&Local) > now)
        .unwrap_or(false)
}

fn next_due(reminder: &Reminder, now: DateTime<Local>) -> Result<DateTime<Local>, String> {
    match &reminder.schedule {
        ReminderSchedule::Interval { interval_minutes } => {
            Ok(now + ChronoDuration::minutes(i64::from((*interval_minutes).max(1))))
        }
        ReminderSchedule::FixedTime { time } => next_fixed_time(time, now),
    }
}

fn next_fixed_time(time: &str, now: DateTime<Local>) -> Result<DateTime<Local>, String> {
    let parsed =
        NaiveTime::parse_from_str(time, "%H:%M").map_err(|_| "固定提醒时间格式无效".to_string())?;
    let today = now.date_naive();
    let candidate = local_datetime(today, parsed)?;
    if candidate > now {
        Ok(candidate)
    } else {
        local_datetime(today + ChronoDuration::days(1), parsed)
    }
}

fn local_datetime(date: chrono::NaiveDate, time: NaiveTime) -> Result<DateTime<Local>, String> {
    Local
        .from_local_datetime(&date.and_time(time))
        .earliest()
        .ok_or_else(|| "无法计算本地提醒时间".to_string())
}

fn tomorrow_start() -> DateTime<Local> {
    let now = Local::now();
    let tomorrow = now.date_naive() + ChronoDuration::days(1);
    Local
        .from_local_datetime(&tomorrow.and_hms_opt(0, 0, 0).expect("午夜时间有效"))
        .earliest()
        .unwrap_or(now + ChronoDuration::days(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_reminder(time: &str) -> Reminder {
        Reminder {
            id: "daily".to_string(),
            enabled: true,
            message: "测试".to_string(),
            schedule: ReminderSchedule::FixedTime {
                time: time.to_string(),
            },
            snooze_minutes: 5,
            paused_until: None,
        }
    }

    #[test]
    fn fixed_time_uses_today_when_time_is_still_ahead() {
        let now = Local
            .with_ymd_and_hms(2026, 7, 27, 8, 0, 0)
            .earliest()
            .expect("local datetime");
        let due = next_due(&fixed_reminder("09:30"), now).expect("next due");

        assert_eq!(due.date_naive(), now.date_naive());
        assert_eq!(due.format("%H:%M").to_string(), "09:30");
    }

    #[test]
    fn fixed_time_moves_to_tomorrow_after_today_has_passed() {
        let now = Local
            .with_ymd_and_hms(2026, 7, 27, 10, 0, 0)
            .earliest()
            .expect("local datetime");
        let due = next_due(&fixed_reminder("09:30"), now).expect("next due");

        assert_eq!(due.date_naive(), now.date_naive() + ChronoDuration::days(1));
        assert_eq!(due.format("%H:%M").to_string(), "09:30");
    }

    #[test]
    fn pause_state_only_applies_before_its_deadline() {
        let now = Local::now();
        let mut reminder = fixed_reminder("09:30");
        reminder.paused_until = Some((now + ChronoDuration::minutes(1)).to_rfc3339());
        assert!(is_paused(&reminder, now));

        reminder.paused_until = Some((now - ChronoDuration::minutes(1)).to_rfc3339());
        assert!(!is_paused(&reminder, now));
    }

    fn interval_reminder(id: &str, message: &str) -> Reminder {
        Reminder {
            id: id.to_string(),
            enabled: true,
            message: message.to_string(),
            schedule: ReminderSchedule::Interval {
                interval_minutes: 30,
            },
            snooze_minutes: 5,
            paused_until: None,
        }
    }

    #[test]
    fn due_reminders_are_shown_in_deadline_order() {
        let state = ReminderState::default();
        let first = interval_reminder("first", "第一个");
        let second = interval_reminder("second", "第二个");
        let settings = AppSettings {
            reminders: vec![first.clone(), second.clone()],
            ..AppSettings::default()
        };
        let now = Local::now();
        {
            let mut data = state.lock().expect("reminder state");
            data.next_due_at
                .insert(first.id.clone(), now - ChronoDuration::minutes(2));
            data.next_due_at
                .insert(second.id.clone(), now - ChronoDuration::minutes(1));
        }

        let active = state
            .collect_due(&settings)
            .expect("collect reminders")
            .expect("first active reminder");
        assert_eq!(active.reminder_id, first.id);

        let next = state
            .dismiss(&first)
            .expect("dismiss first")
            .expect("next reminder");
        assert_eq!(next.reminder_id, second.id);
    }

    #[test]
    fn unrelated_settings_update_keeps_existing_deadline() {
        let state = ReminderState::default();
        let reminder = interval_reminder("water", "喝水");
        let settings = AppSettings {
            reminders: vec![reminder],
            ..AppSettings::default()
        };
        state.configure(&settings).expect("configure reminder");
        let scheduled = state
            .lock()
            .expect("reminder state")
            .next_due_at
            .get("water")
            .cloned()
            .expect("scheduled deadline");

        let updated_settings = AppSettings {
            pet_scale: 1.1,
            ..settings
        };
        state
            .configure(&updated_settings)
            .expect("reconfigure reminder");
        let after_update = state
            .lock()
            .expect("reminder state")
            .next_due_at
            .get("water")
            .cloned()
            .expect("retained deadline");

        assert_eq!(after_update, scheduled);
    }
}
