use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    settings::{ReminderSettings, SettingsState},
    windowing,
};

const REMINDER_EVENT: &str = "pet://reminder-payload";
const CHECK_INTERVAL: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, Serialize)]
pub struct ReminderPayload {
    pub message: String,
    pub snooze_minutes: u32,
}

#[derive(Default)]
struct ReminderData {
    active: Option<ReminderPayload>,
    next_due_at: Option<Instant>,
}

#[derive(Default)]
pub struct ReminderState {
    inner: Mutex<ReminderData>,
}

impl ReminderState {
    pub fn configure(&self, settings: &ReminderSettings) -> Result<(), String> {
        let mut data = self.lock()?;
        if settings.enabled {
            if data.active.is_some() {
                data.active = Some(payload_from_settings(settings));
            } else {
                data.next_due_at = Some(Self::next_in(settings.interval_minutes));
            }
        } else {
            data.active = None;
            data.next_due_at = None;
        }
        Ok(())
    }

    pub fn active_payload(&self) -> Result<Option<ReminderPayload>, String> {
        Ok(self.lock()?.active.clone())
    }

    pub fn should_fire(&self) -> Result<bool, String> {
        let data = self.lock()?;
        Ok(data.active.is_none() && data.next_due_at.is_some_and(|due| Instant::now() >= due))
    }

    pub fn activate(&self, settings: &ReminderSettings) -> Result<ReminderPayload, String> {
        let payload = payload_from_settings(settings);
        let mut data = self.lock()?;
        data.active = Some(payload.clone());
        data.next_due_at = None;
        Ok(payload)
    }

    pub fn dismiss(&self, settings: &ReminderSettings) -> Result<(), String> {
        let mut data = self.lock()?;
        data.active = None;
        data.next_due_at = if settings.enabled {
            Some(Self::next_in(settings.interval_minutes))
        } else {
            None
        };
        Ok(())
    }

    pub fn snooze(&self, settings: &ReminderSettings) -> Result<(), String> {
        let mut data = self.lock()?;
        data.active = None;
        data.next_due_at = if settings.enabled {
            Some(Self::next_in(settings.snooze_minutes))
        } else {
            None
        };
        Ok(())
    }

    fn next_in(minutes: u32) -> Instant {
        Instant::now() + Duration::from_secs(u64::from(minutes.max(1)) * 60)
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
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .configure(&settings.reminder)?;
    if !settings.reminder.enabled {
        windowing::hide_reminder_window(app)?;
        return Ok(());
    }
    if let Some(payload) = active_payload(app)? {
        app.emit_to(windowing::REMINDER_WINDOW, REMINDER_EVENT, payload)
            .map_err(|error| error.to_string())?;
        windowing::sync_reminder_window_visibility(app)?;
    }
    Ok(())
}

pub fn active_payload(app: &AppHandle) -> Result<Option<ReminderPayload>, String> {
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .active_payload()
}

pub fn preview(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    let payload = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .activate(&settings.reminder)?;
    show_payload(app, payload)
}

pub fn dismiss(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .dismiss(&settings.reminder)?;
    windowing::hide_reminder_window(app)
}

pub fn snooze(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    app.try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?
        .snooze(&settings.reminder)?;
    windowing::hide_reminder_window(app)
}

fn check_and_fire(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .try_state::<SettingsState>()
        .ok_or_else(|| "找不到应用设置状态".to_string())?
        .get()?;
    let state = app
        .try_state::<ReminderState>()
        .ok_or_else(|| "找不到提醒状态".to_string())?;
    if !state.should_fire()? {
        return Ok(());
    }
    show_payload(app, state.activate(&settings.reminder)?)
}

fn show_payload(app: &AppHandle, payload: ReminderPayload) -> Result<(), String> {
    app.emit_to(windowing::REMINDER_WINDOW, REMINDER_EVENT, payload)
        .map_err(|error| error.to_string())?;
    windowing::sync_reminder_window_visibility(app)
}

fn payload_from_settings(settings: &ReminderSettings) -> ReminderPayload {
    ReminderPayload {
        message: settings.message.trim().to_string(),
        snooze_minutes: settings.snooze_minutes,
    }
}
