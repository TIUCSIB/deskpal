use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager};

use super::normalize::{
    normalize_bounds, normalize_pet_role, normalize_reminder, parse_settings, unique_id,
};
use super::{
    default_reminder_message, default_reminder_snooze_minutes, normalize_pause,
    normalize_quiet_hours, AppSettings, InfoMode, PortableSettingsEnvelope, QuietHours, Reminder,
    ReminderInput, SavedPosition, SavedWindowBounds, SETTINGS_SCHEMA_VERSION,
};

const SETTINGS_FILE: &str = "settings.json";
const POSITION_SAVE_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct LegacyReminderSettings {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default = "default_reminder_message")]
    pub(crate) message: String,
    #[serde(default = "super::default_reminder_interval_minutes")]
    pub(crate) interval_minutes: u32,
    #[serde(default = "default_reminder_snooze_minutes")]
    pub(crate) snooze_minutes: u32,
}
#[derive(serde::Deserialize)]
pub(crate) struct StoredSettings {
    #[serde(flatten)]
    pub(crate) settings: AppSettings,
    #[serde(default)]
    pub(crate) reminder: Option<LegacyReminderSettings>,
}

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
    pub fn set_pet_role(&self, role: String) -> Result<AppSettings, String> {
        self.update(|s| s.pet_role = normalize_pet_role(role))
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
    fn persist(&self) -> Result<(), String> {
        self.write(&self.get()?)
    }
    fn write(&self, settings: &AppSettings) -> Result<(), String> {
        let content = serde_json::to_vec_pretty(settings)
            .map_err(|error| format!("序列化设置失败: {error}"))?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "设置文件路径无效。".to_string())?;
        fs::create_dir_all(parent).map_err(|error| format!("创建设置目录失败: {error}"))?;

        let temporary = self.path.with_extension("json.tmp");
        let backup = self.path.with_extension("json.bak");
        let mut file =
            File::create(&temporary).map_err(|error| format!("创建设置临时文件失败: {error}"))?;
        if let Err(error) = file.write_all(&content).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("写入设置临时文件失败: {error}"));
        }
        drop(file);

        if backup.exists() {
            fs::remove_file(&backup).map_err(|error| format!("清理旧设置备份失败: {error}"))?;
        }
        let had_current_settings = self.path.exists();
        if had_current_settings {
            fs::rename(&self.path, &backup)
                .map_err(|error| format!("备份当前设置失败: {error}"))?;
        }
        if let Err(error) = fs::rename(&temporary, &self.path) {
            if had_current_settings {
                let _ = fs::rename(&backup, &self.path);
            }
            let _ = fs::remove_file(&temporary);
            return Err(format!("替换设置文件失败: {error}"));
        }
        if backup.exists() {
            fs::remove_file(backup).map_err(|error| format!("清理设置备份失败: {error}"))?;
        }
        Ok(())
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SettingsData>, String> {
        self.inner
            .lock()
            .map_err(|_| "应用设置暂时不可用".to_string())
    }
}

fn recover_interrupted_write(path: &std::path::Path) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    if !path.exists() && backup.exists() {
        fs::rename(&backup, path).map_err(|error| format!("恢复设置备份失败: {error}"))?;
    }
    if temporary.exists() {
        fs::remove_file(temporary).map_err(|error| format!("清理未完成设置写入失败: {error}"))?;
    }
    if path.exists() && backup.exists() {
        fs::remove_file(backup).map_err(|error| format!("清理过期设置备份失败: {error}"))?;
    }
    Ok(())
}

fn normalize_loaded_settings(app: &AppHandle, settings: &mut AppSettings) {
    settings.schema_version = SETTINGS_SCHEMA_VERSION;
    let stored_role = std::mem::take(&mut settings.pet_role);
    settings.pet_role = if crate::role_packs::is_valid_role(app, &stored_role) {
        stored_role
    } else {
        normalize_pet_role(stored_role)
    };
    normalize_settings(settings);
}

fn normalize_settings(settings: &mut AppSettings) {
    settings.schema_version = SETTINGS_SCHEMA_VERSION;
    settings.pet_scale = if settings.pet_scale.is_finite() {
        settings.pet_scale.clamp(0.45, 1.2)
    } else {
        super::DEFAULT_PET_SCALE
    };
    settings.quiet_hours = normalize_quiet_hours(std::mem::take(&mut settings.quiet_hours));
    settings.reminders =
        super::normalize::normalize_reminders(std::mem::take(&mut settings.reminders));
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(SETTINGS_FILE))
}
