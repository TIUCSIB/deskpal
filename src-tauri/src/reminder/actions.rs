use std::{thread, time::Duration};

use super::{
    schedule::{quiet_end, tomorrow_start},
    ActiveReminder, ReminderEventKind, ReminderHistoryState, ReminderPayload, ReminderState,
};
use crate::{
    commands::system_info::SystemMonitor,
    settings::{AppSettings, Reminder, SettingsState},
    windowing,
};
use tauri::{AppHandle, Emitter, Manager, State};

const REMINDER_EVENT: &str = "pet://reminder-payload";
const ACTIVITY_UPDATED_EVENT: &str = "pet://reminder-activity-updated";
const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";
const CHECK_INTERVAL: Duration = Duration::from_secs(10);

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(CHECK_INTERVAL);
        if let Err(error) = check_and_fire(&app) {
            eprintln!("提醒调度失败: {error}");
        }
    });
}
pub fn sync_from_settings(app: &AppHandle) -> Result<(), String> {
    let settings = get_settings(app)?;
    state(app)?.configure(&settings)?;
    sync_next(app, state(app)?.active_payload()?)
}
pub fn active_payload(app: &AppHandle) -> Result<Option<ReminderPayload>, String> {
    state(app)?.active_payload()
}
pub fn preview(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let settings = get_settings(app)?;
    let reminder = find_reminder(&settings, &reminder_id)?;
    show_payload(app, state(app)?.preview(reminder)?)
}
pub fn dismiss(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    complete(app, reminder_id)
}
pub fn complete(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    finish(app, reminder_id, false, ReminderEventKind::Completed)
}
pub fn snooze(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    finish(app, reminder_id, true, ReminderEventKind::Snoozed)
}
pub fn pause_all_until_tomorrow(app: &AppHandle) -> Result<(), String> {
    let updated =
        settings_state(app)?.pause_enabled_reminders_until(tomorrow_start().to_rfc3339())?;
    for active in state(app)?.clear_active_and_queue()? {
        if !active.preview {
            log(app, ReminderEventKind::Skipped, &active)?;
        }
    }
    state(app)?.configure(&updated)?;
    windowing::hide_reminder_window(app)?;
    app.emit(SETTINGS_UPDATED_EVENT, &updated)
        .map_err(|e| e.to_string())
}
pub fn pause_until_tomorrow(app: &AppHandle, reminder_id: String) -> Result<(), String> {
    let active = require_active(app, &reminder_id)?;
    if active.preview {
        return complete(app, reminder_id);
    }
    log(app, ReminderEventKind::Skipped, &active)?;
    let updated = settings_state(app)?
        .set_reminder_pause(reminder_id.clone(), Some(tomorrow_start().to_rfc3339()))?;
    state(app)?.configure(&updated)?;
    sync_next(app, state(app)?.remove_active(&reminder_id)?)?;
    app.emit(SETTINGS_UPDATED_EVENT, &updated)
        .map_err(|e| e.to_string())
}
pub fn remove_reminder(app: &AppHandle, id: &str) -> Result<(), String> {
    if let Some(active) = state(app)?.active()? {
        if active.payload.reminder_id == id && !active.preview {
            log(app, ReminderEventKind::Skipped, &active)?;
        }
    }
    sync_next(app, state(app)?.remove_active(id)?)
}

fn finish(
    app: &AppHandle,
    id: String,
    snooze: bool,
    kind: ReminderEventKind,
) -> Result<(), String> {
    let settings = get_settings(app)?;
    let reminder = find_reminder(&settings, &id)?;
    let active = require_active(app, &id)?;
    let quiet_end = if snooze {
        let snooze_target =
            chrono::Local::now() + chrono::Duration::minutes(i64::from(reminder.snooze_minutes));
        quiet_end(&settings.quiet_hours, snooze_target)?
    } else {
        None
    };
    if !active.preview {
        log(app, kind, &active)?;
        if quiet_end.is_some() {
            log(app, ReminderEventKind::QuietDeferred, &active)?;
        }
    }
    sync_next(app, state(app)?.finish(reminder, snooze, quiet_end)?)
}
fn check_and_fire(app: &AppHandle) -> Result<(), String> {
    let settings = get_settings(app)?;
    let idle_secs = app
        .try_state::<SystemMonitor>()
        .and_then(|monitor| monitor.idle_seconds());
    let intervals_paused = state(app)?.reconcile_interval_pause(&settings, idle_secs)?;
    let (payload, deferred) = state(app)?.collect_due(&settings, intervals_paused)?;
    for item in deferred {
        log(app, ReminderEventKind::QuietDeferred, &item)?;
    }
    if let Some(payload) = payload {
        show_payload(app, payload)?;
    }
    Ok(())
}
fn sync_next(app: &AppHandle, next: Option<ReminderPayload>) -> Result<(), String> {
    if let Some(payload) = next {
        show_payload(app, payload)
    } else {
        windowing::hide_reminder_window(app)
    }
}
fn show_payload(app: &AppHandle, payload: ReminderPayload) -> Result<(), String> {
    if let Some(active) = state(app)?.take_shown_event()? {
        log(app, ReminderEventKind::Shown, &active)?;
    }
    app.emit_to(windowing::REMINDER_WINDOW, REMINDER_EVENT, payload)
        .map_err(|e| e.to_string())?;
    windowing::sync_reminder_window_visibility(app)
}
fn log(app: &AppHandle, kind: ReminderEventKind, active: &ActiveReminder) -> Result<(), String> {
    let history = app
        .try_state::<ReminderHistoryState>()
        .ok_or_else(|| "找不到提醒历史状态".to_string())?;
    history.record(
        kind,
        active.payload.reminder_id.clone(),
        active.payload.message.clone(),
        active.scheduled_at,
    )?;
    app.emit(ACTIVITY_UPDATED_EVENT, history.activity()?)
        .map_err(|e| e.to_string())
}
fn state(app: &AppHandle) -> Result<State<'_, ReminderState>, String> {
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())
}
fn settings_state(app: &AppHandle) -> Result<State<'_, SettingsState>, String> {
    app.try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())
}
fn get_settings(app: &AppHandle) -> Result<AppSettings, String> {
    settings_state(app)?.get()
}
fn find_reminder<'a>(settings: &'a AppSettings, id: &str) -> Result<&'a Reminder, String> {
    settings
        .reminders
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| "找不到该提醒".to_string())
}
fn require_active(app: &AppHandle, id: &str) -> Result<ActiveReminder, String> {
    let active = state(app)?
        .active()?
        .ok_or_else(|| "提醒已更新，请重新操作".to_string())?;
    if active.payload.reminder_id != id {
        return Err("提醒已更新，请重新操作".to_string());
    }
    Ok(active)
}
