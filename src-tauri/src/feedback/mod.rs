use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::windowing;

const FEEDBACK_DURATION_SECONDS: u64 = 10;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SystemFeedbackPayload {
    pub id: String,
    pub kind: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub occurred_at: i64,
}

#[derive(Default)]
pub struct SystemFeedbackState {
    active: Mutex<Option<SystemFeedbackPayload>>,
}

impl SystemFeedbackState {
    pub fn active_payload(&self) -> Result<Option<SystemFeedbackPayload>, String> {
        self.active
            .lock()
            .map(|payload| payload.clone())
            .map_err(|_| "系统反馈状态暂时不可用".to_string())
    }

    fn set_active(&self, payload: Option<SystemFeedbackPayload>) -> Result<(), String> {
        *self
            .active
            .lock()
            .map_err(|_| "系统反馈状态暂时不可用".to_string())? = payload;
        Ok(())
    }
}

pub fn show(app: &AppHandle, payload: SystemFeedbackPayload) -> Result<(), String> {
    let state = app
        .try_state::<SystemFeedbackState>()
        .ok_or_else(|| "系统反馈状态尚未初始化".to_string())?;
    state.set_active(Some(payload.clone()))?;
    app.emit("pet://system-feedback-payload", payload.clone())
        .map_err(|error| error.to_string())?;
    windowing::sync_system_feedback_window_visibility(app)?;

    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(FEEDBACK_DURATION_SECONDS));
        if active_payload(&app)
            .ok()
            .flatten()
            .is_some_and(|active| active.id == payload.id)
        {
            let _ = dismiss(&app, payload.id);
        }
    });
    Ok(())
}

pub fn dismiss(app: &AppHandle, id: String) -> Result<(), String> {
    let state = app
        .try_state::<SystemFeedbackState>()
        .ok_or_else(|| "系统反馈状态尚未初始化".to_string())?;
    let active = state.active_payload()?;
    if active.is_some_and(|payload| payload.id != id) {
        return Ok(());
    }
    state.set_active(None)?;
    windowing::sync_system_feedback_window_visibility(app)
}

pub fn active_payload(app: &AppHandle) -> Result<Option<SystemFeedbackPayload>, String> {
    app.try_state::<SystemFeedbackState>()
        .ok_or_else(|| "系统反馈状态尚未初始化".to_string())?
        .active_payload()
}
