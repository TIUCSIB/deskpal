use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use tauri::AppHandle;

use super::normalize::{normalize_bounds, normalize_reminder, parse_settings, unique_id};
use super::state_support::{
    normalize_loaded_settings, normalize_settings, recover_interrupted_write, settings_path,
};
use super::{
    normalize_pause, normalize_quiet_hours, AppSettings, InfoMode, PortableSettingsEnvelope,
    QuietHours, Reminder, ReminderInput, SavedPosition, SavedWindowBounds,
};

const POSITION_SAVE_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct SettingsData {
    pub(crate) settings: AppSettings,
    pub(crate) last_position_save: Option<Instant>,
    pub(crate) last_settings_window_save: Option<Instant>,
}
pub struct SettingsState {
    pub(crate) path: PathBuf,
    pub(crate) inner: Mutex<SettingsData>,
}

impl SettingsState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let path = settings_path(app)?;
        recover_interrupted_write(&path)?;
        let mut settings = match fs::read_to_string(&path) {
            Ok(content) => parse_settings(&content).ok_or_else(|| {
                format!(
                    "设置文件格式无效，已保留原文件以避免覆盖：{}",
                    path.display()
                )
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
            Err(error) => return Err(format!("读取设置文件失败: {error}")),
        };
        normalize_loaded_settings(app, &mut settings);
        let state = Self {
            path,
            inner: Mutex::new(SettingsData {
                settings,
                last_position_save: None,
                last_settings_window_save: None,
            }),
        };
        state.persist()?;
        Ok(state)
    }
    pub fn get(&self) -> Result<AppSettings, String> {
        Ok(self.lock()?.settings.clone())
    }
    pub fn set_pet_scale(&self, scale: f64) -> Result<AppSettings, String> {
        self.update(|s| s.pet_scale = scale)
    }
    pub fn set_validated_pet_role(&self, role: String) -> Result<AppSettings, String> {
        self.update(|settings| settings.pet_role = role)
    }
    pub fn complete_onboarding(&self) -> Result<AppSettings, String> {
        self.update(|settings| settings.onboarding_completed = true)
    }
    pub fn portable_export(&self) -> Result<PortableSettingsEnvelope, String> {
        Ok(PortableSettingsEnvelope::from_settings(&self.get()?))
    }
    pub fn import_portable(
        &self,
        envelope: PortableSettingsEnvelope,
        valid_pet_role: String,
    ) -> Result<AppSettings, String> {
        let current = self.get()?;
        let mut imported = envelope.apply_to(&current, valid_pet_role);
        normalize_settings(&mut imported);
        self.replace(imported)
    }
    pub fn restore(&self, settings: AppSettings) -> Result<AppSettings, String> {
        self.replace(settings)
    }
    pub fn set_info_mode(&self, mode: InfoMode) -> Result<AppSettings, String> {
        self.update(|s| s.info_mode = mode)
    }
    pub fn set_size_locked(&self, locked: bool) -> Result<AppSettings, String> {
        self.update(|s| s.size_locked = locked)
    }
    pub fn set_shortcut_enabled(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| s.shortcut_enabled = enabled)
    }
    pub fn set_launch_at_startup(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| s.launch_at_startup = enabled)
    }
    pub fn set_main_window_always_on_top(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| s.main_window_always_on_top = enabled)
    }
    pub fn set_main_window_show_in_taskbar(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| s.main_window_show_in_taskbar = enabled)
    }
    pub fn set_main_window_left_click_passthrough(
        &self,
        enabled: bool,
    ) -> Result<AppSettings, String> {
        self.update(|s| s.main_window_left_click_passthrough = enabled)
    }
    pub fn set_chat_shortcut(&self, shortcut: String) -> Result<AppSettings, String> {
        self.update(|s| s.chat_shortcut = shortcut)
    }
    pub fn set_quiet_hours(&self, quiet_hours: QuietHours) -> Result<AppSettings, String> {
        self.update(|s| s.quiet_hours = normalize_quiet_hours(quiet_hours))
    }
    pub fn create_reminder(&self, input: ReminderInput) -> Result<AppSettings, String> {
        self.update(|settings| {
            let index = settings.reminders.len();
            let reminder = normalize_reminder(
                Reminder {
                    id: input
                        .id
                        .unwrap_or_else(|| format!("reminder-{}", index + 1)),
                    enabled: true,
                    message: input.message,
                    schedule: input.schedule,
                    snooze_minutes: input.snooze_minutes,
                    paused_until: None,
                },
                index,
            );
            let id = unique_id(&settings.reminders, &reminder.id);
            settings.reminders.push(Reminder { id, ..reminder });
        })
    }
    pub fn update_reminder(&self, reminder: Reminder) -> Result<AppSettings, String> {
        self.update(|s| {
            if let Some(index) = s.reminders.iter().position(|item| item.id == reminder.id) {
                s.reminders[index] = normalize_reminder(reminder, index);
            }
        })
    }
    pub fn delete_reminder(&self, id: String) -> Result<AppSettings, String> {
        self.update(|s| s.reminders.retain(|r| r.id != id))
    }
    pub fn set_reminder_enabled(&self, id: String, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| {
            if let Some(r) = s.reminders.iter_mut().find(|r| r.id == id) {
                r.enabled = enabled;
                if !enabled {
                    r.paused_until = None;
                }
            }
        })
    }
    pub fn set_reminder_pause(
        &self,
        id: String,
        paused_until: Option<String>,
    ) -> Result<AppSettings, String> {
        self.update(|s| {
            if let Some(r) = s.reminders.iter_mut().find(|r| r.id == id) {
                r.paused_until = normalize_pause(paused_until);
            }
        })
    }
    pub fn pause_enabled_reminder_until(
        &self,
        id: String,
        paused_until: String,
    ) -> Result<(AppSettings, String), String> {
        let mut data = self.lock()?;
        let previous = data.settings.clone();
        let reminder = data
            .settings
            .reminders
            .iter_mut()
            .find(|reminder| reminder.id == id)
            .ok_or_else(|| "找不到该提醒".to_string())?;
        if !reminder.enabled {
            return Err("该提醒尚未启用".to_string());
        }
        let pause_active = reminder
            .paused_until
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .is_some_and(|until| until.with_timezone(&chrono::Utc) > chrono::Utc::now());
        if pause_active {
            return Err("该提醒已暂停".to_string());
        }
        let message = reminder.message.clone();
        reminder.paused_until = normalize_pause(Some(paused_until));
        let settings = data.settings.clone();
        drop(data);
        if let Err(error) = self.write(&settings) {
            self.lock()?.settings = previous;
            return Err(error);
        }
        Ok((settings, message))
    }
    pub fn pause_enabled_reminders_until(
        &self,
        paused_until: String,
    ) -> Result<AppSettings, String> {
        self.update(|s| {
            let until = normalize_pause(Some(paused_until));
            for r in &mut s.reminders {
                if r.enabled {
                    r.paused_until = until.clone();
                }
            }
        })
    }
    pub fn reset_all(&self) -> Result<AppSettings, String> {
        self.update(|s| *s = AppSettings::default())
    }
    pub fn reset_main_position(&self) -> Result<AppSettings, String> {
        self.update(|s| s.main_position = None)
    }
    pub fn reset_settings_window_bounds(&self) -> Result<AppSettings, String> {
        self.update(|s| s.settings_window_bounds = None)
    }
    pub fn save_main_position_throttled(
        &self,
        position: SavedPosition,
    ) -> Result<AppSettings, String> {
        self.save_position(position)
    }
    pub fn save_settings_window_bounds_throttled(
        &self,
        bounds: SavedWindowBounds,
    ) -> Result<AppSettings, String> {
        self.save_bounds(normalize_bounds(bounds))
    }
    pub fn flush_pending_geometry(&self) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        let persist = data.last_position_save.is_some() || data.last_settings_window_save.is_some();
        data.last_position_save = None;
        data.last_settings_window_save = None;
        self.persist_data(data, persist)
    }
    fn save_position(&self, position: SavedPosition) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        data.settings.main_position = Some(position);
        let persist = data
            .last_position_save
            .is_none_or(|time| Instant::now().duration_since(time) >= POSITION_SAVE_INTERVAL);
        data.last_position_save = Some(Instant::now());
        self.persist_data(data, persist)
    }
    fn save_bounds(&self, bounds: SavedWindowBounds) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        data.settings.settings_window_bounds = Some(bounds);
        let persist = data
            .last_settings_window_save
            .is_none_or(|time| Instant::now().duration_since(time) >= POSITION_SAVE_INTERVAL);
        data.last_settings_window_save = Some(Instant::now());
        self.persist_data(data, persist)
    }
    fn persist_data(
        &self,
        data: std::sync::MutexGuard<'_, SettingsData>,
        persist: bool,
    ) -> Result<AppSettings, String> {
        let settings = data.settings.clone();
        drop(data);
        if persist {
            self.write(&settings)?;
        }
        Ok(settings)
    }
    fn update(&self, mutate: impl FnOnce(&mut AppSettings)) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        let previous = data.settings.clone();
        mutate(&mut data.settings);
        let settings = data.settings.clone();
        drop(data);
        if let Err(error) = self.write(&settings) {
            self.lock()?.settings = previous;
            return Err(error);
        }
        Ok(settings)
    }
    fn replace(&self, settings: AppSettings) -> Result<AppSettings, String> {
        self.write(&settings)?;
        self.lock()?.settings = settings.clone();
        Ok(settings)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SettingsData>, String> {
        self.inner
            .lock()
            .map_err(|_| "应用设置暂时不可用".to_string())
    }
}
