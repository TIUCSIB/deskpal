use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

const SETTINGS_FILE: &str = "settings.json";
const POSITION_SAVE_INTERVAL: Duration = Duration::from_millis(500);
const LEGACY_REMINDER_ID: &str = "legacy-default";
pub const DEFAULT_PET_SCALE: f64 = 0.85;
pub const DEFAULT_CHAT_SHORTCUT: &str = "Ctrl+Alt+D";
pub const DEFAULT_SETTINGS_WINDOW_WIDTH: u32 = 500;
pub const DEFAULT_SETTINGS_WINDOW_HEIGHT: u32 = 560;
pub const MIN_SETTINGS_WINDOW_WIDTH: u32 = 460;
pub const MIN_SETTINGS_WINDOW_HEIGHT: u32 = 520;
pub const DEFAULT_REMINDER_MESSAGE: &str = "记得喝水，起来活动一下吧";
pub const DEFAULT_REMINDER_INTERVAL_MINUTES: u32 = 30;
pub const DEFAULT_REMINDER_SNOOZE_MINUTES: u32 = 5;
pub const DEFAULT_PET_ROLE: &str = "guga";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InfoMode {
    Auto,
    Always,
    Hidden,
}

impl Default for InfoMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SavedWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReminderSchedule {
    Interval { interval_minutes: u32 },
    FixedTime { time: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Reminder {
    pub id: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_reminder_message")]
    pub message: String,
    pub schedule: ReminderSchedule,
    #[serde(default = "default_reminder_snooze_minutes")]
    pub snooze_minutes: u32,
    #[serde(default)]
    pub paused_until: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReminderInput {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_reminder_message")]
    pub message: String,
    pub schedule: ReminderSchedule,
    #[serde(default = "default_reminder_snooze_minutes")]
    pub snooze_minutes: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct LegacyReminderSettings {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_reminder_message")]
    message: String,
    #[serde(default = "default_reminder_interval_minutes")]
    interval_minutes: u32,
    #[serde(default = "default_reminder_snooze_minutes")]
    snooze_minutes: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppSettings {
    #[serde(default)]
    pub main_position: Option<SavedPosition>,
    #[serde(default)]
    pub settings_window_bounds: Option<SavedWindowBounds>,
    #[serde(default = "default_pet_scale")]
    pub pet_scale: f64,
    #[serde(default = "default_pet_role")]
    pub pet_role: String,
    #[serde(default)]
    pub info_mode: InfoMode,
    #[serde(default)]
    pub size_locked: bool,
    #[serde(default = "default_shortcut_enabled")]
    pub shortcut_enabled: bool,
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default = "default_always_on_top")]
    pub main_window_always_on_top: bool,
    #[serde(default)]
    pub main_window_show_in_taskbar: bool,
    #[serde(default = "default_chat_shortcut")]
    pub chat_shortcut: String,
    #[serde(default)]
    pub reminders: Vec<Reminder>,
}

#[derive(Deserialize)]
struct StoredSettings {
    #[serde(flatten)]
    settings: AppSettings,
    #[serde(default)]
    reminder: Option<LegacyReminderSettings>,
}

impl From<(PhysicalPosition<i32>, PhysicalSize<u32>)> for SavedWindowBounds {
    fn from(value: (PhysicalPosition<i32>, PhysicalSize<u32>)) -> Self {
        Self {
            x: value.0.x,
            y: value.0.y,
            width: value.1.width,
            height: value.1.height,
        }
    }
}

impl From<SavedWindowBounds> for (PhysicalPosition<i32>, PhysicalSize<u32>) {
    fn from(bounds: SavedWindowBounds) -> Self {
        (
            PhysicalPosition::new(bounds.x, bounds.y),
            PhysicalSize::new(bounds.width, bounds.height),
        )
    }
}

impl From<PhysicalPosition<i32>> for SavedPosition {
    fn from(position: PhysicalPosition<i32>) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
}

impl From<SavedPosition> for PhysicalPosition<i32> {
    fn from(position: SavedPosition) -> Self {
        Self::new(position.x, position.y)
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            main_position: None,
            settings_window_bounds: None,
            pet_scale: default_pet_scale(),
            pet_role: default_pet_role(),
            info_mode: InfoMode::default(),
            size_locked: false,
            shortcut_enabled: default_shortcut_enabled(),
            launch_at_startup: false,
            main_window_always_on_top: default_always_on_top(),
            main_window_show_in_taskbar: false,
            chat_shortcut: default_chat_shortcut(),
            reminders: Vec::new(),
        }
    }
}

fn default_pet_scale() -> f64 {
    DEFAULT_PET_SCALE
}

fn default_pet_role() -> String {
    DEFAULT_PET_ROLE.to_string()
}

fn default_shortcut_enabled() -> bool {
    true
}

fn default_always_on_top() -> bool {
    true
}

fn default_chat_shortcut() -> String {
    DEFAULT_CHAT_SHORTCUT.to_string()
}

fn default_reminder_message() -> String {
    DEFAULT_REMINDER_MESSAGE.to_string()
}

fn default_reminder_interval_minutes() -> u32 {
    DEFAULT_REMINDER_INTERVAL_MINUTES
}

fn default_reminder_snooze_minutes() -> u32 {
    DEFAULT_REMINDER_SNOOZE_MINUTES
}

fn normalize_settings_window_bounds(bounds: SavedWindowBounds) -> SavedWindowBounds {
    SavedWindowBounds {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(MIN_SETTINGS_WINDOW_WIDTH),
        height: bounds.height.max(MIN_SETTINGS_WINDOW_HEIGHT),
    }
}

fn normalize_reminder_message(message: String) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        default_reminder_message()
    } else {
        trimmed.to_string()
    }
}

fn normalize_minutes(value: u32) -> u32 {
    value.max(1)
}

fn normalize_time(time: String) -> String {
    let parts: Vec<_> = time.trim().split(':').collect();
    if parts.len() != 2 {
        return "09:00".to_string();
    }
    let Ok(hour) = parts[0].parse::<u32>() else {
        return "09:00".to_string();
    };
    let Ok(minute) = parts[1].parse::<u32>() else {
        return "09:00".to_string();
    };
    if hour > 23 || minute > 59 {
        return "09:00".to_string();
    }
    format!("{hour:02}:{minute:02}")
}

fn normalize_schedule(schedule: ReminderSchedule) -> ReminderSchedule {
    match schedule {
        ReminderSchedule::Interval { interval_minutes } => ReminderSchedule::Interval {
            interval_minutes: normalize_minutes(interval_minutes),
        },
        ReminderSchedule::FixedTime { time } => ReminderSchedule::FixedTime {
            time: normalize_time(time),
        },
    }
}

fn normalize_pause(paused_until: Option<String>) -> Option<String> {
    paused_until.and_then(|value| {
        DateTime::parse_from_rfc3339(value.trim())
            .ok()
            .map(|time| time.with_timezone(&Local).to_rfc3339())
    })
}

fn normalize_reminder(mut reminder: Reminder, index: usize) -> Reminder {
    let fallback_id = format!("reminder-{}", index + 1);
    reminder.id = if reminder.id.trim().is_empty() {
        fallback_id
    } else {
        reminder.id.trim().to_string()
    };
    reminder.message = normalize_reminder_message(reminder.message);
    reminder.schedule = normalize_schedule(reminder.schedule);
    reminder.snooze_minutes = normalize_minutes(reminder.snooze_minutes);
    reminder.paused_until = normalize_pause(reminder.paused_until);
    reminder
}

fn normalize_reminders(reminders: Vec<Reminder>) -> Vec<Reminder> {
    let mut ids = std::collections::HashSet::new();
    reminders
        .into_iter()
        .enumerate()
        .map(|(index, mut reminder)| {
            reminder = normalize_reminder(reminder, index);
            let base = reminder.id.clone();
            let mut suffix = 2;
            while !ids.insert(reminder.id.clone()) {
                reminder.id = format!("{base}-{suffix}");
                suffix += 1;
            }
            reminder
        })
        .collect()
}

fn normalize_pet_role(role: String) -> String {
    match role.as_str() {
        "guga" | "monthly-salary-cat" | "broom-witch" => role,
        _ => default_pet_role(),
    }
}

fn legacy_reminder(legacy: LegacyReminderSettings) -> Reminder {
    Reminder {
        id: LEGACY_REMINDER_ID.to_string(),
        enabled: legacy.enabled,
        message: legacy.message,
        schedule: ReminderSchedule::Interval {
            interval_minutes: legacy.interval_minutes,
        },
        snooze_minutes: legacy.snooze_minutes,
        paused_until: None,
    }
}

fn migrate_stored_settings(stored: StoredSettings, has_reminders: bool) -> AppSettings {
    let mut settings = stored.settings;
    if !has_reminders {
        settings.reminders = stored.reminder.map(legacy_reminder).into_iter().collect();
    }
    settings
}

struct SettingsData {
    settings: AppSettings,
    last_position_save: Option<Instant>,
    last_settings_window_save: Option<Instant>,
}

pub struct SettingsState {
    path: PathBuf,
    inner: Mutex<SettingsData>,
}

impl SettingsState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let path = settings_path(app)?;
        let mut settings = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            match serde_json::from_str::<StoredSettings>(&content) {
                Ok(stored) => {
                    let has_reminders = serde_json::from_str::<serde_json::Value>(&content)
                        .ok()
                        .and_then(|value| {
                            value
                                .as_object()
                                .map(|object| object.contains_key("reminders"))
                        })
                        .unwrap_or(false);
                    migrate_stored_settings(stored, has_reminders)
                }
                Err(error) => {
                    eprintln!("无法读取应用设置，已使用默认值: {error}");
                    AppSettings::default()
                }
            }
        } else {
            AppSettings::default()
        };
        settings.pet_role = normalize_pet_role(settings.pet_role);
        settings.reminders = normalize_reminders(settings.reminders);
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
        self.update(|settings| settings.pet_scale = scale)
    }

    pub fn set_pet_role(&self, role: String) -> Result<AppSettings, String> {
        self.update(|settings| settings.pet_role = normalize_pet_role(role))
    }

    pub fn set_info_mode(&self, mode: InfoMode) -> Result<AppSettings, String> {
        self.update(|settings| settings.info_mode = mode)
    }

    pub fn set_size_locked(&self, locked: bool) -> Result<AppSettings, String> {
        self.update(|settings| settings.size_locked = locked)
    }

    pub fn set_shortcut_enabled(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|settings| settings.shortcut_enabled = enabled)
    }

    pub fn set_launch_at_startup(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|settings| settings.launch_at_startup = enabled)
    }

    pub fn set_main_window_always_on_top(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|settings| settings.main_window_always_on_top = enabled)
    }

    pub fn set_main_window_show_in_taskbar(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|settings| settings.main_window_show_in_taskbar = enabled)
    }

    pub fn set_chat_shortcut(&self, shortcut: String) -> Result<AppSettings, String> {
        self.update(|settings| settings.chat_shortcut = shortcut)
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
            let base = reminder.id.clone();
            let mut candidate = base.clone();
            let mut suffix = 2;
            while settings.reminders.iter().any(|item| item.id == candidate) {
                candidate = format!("{base}-{suffix}");
                suffix += 1;
            }
            settings.reminders.push(Reminder {
                id: candidate,
                ..reminder
            });
        })
    }

    pub fn update_reminder(&self, reminder: Reminder) -> Result<AppSettings, String> {
        self.update(|settings| {
            let Some(index) = settings
                .reminders
                .iter()
                .position(|item| item.id == reminder.id)
            else {
                return;
            };
            settings.reminders[index] = normalize_reminder(reminder, index);
        })
    }

    pub fn delete_reminder(&self, id: String) -> Result<AppSettings, String> {
        self.update(|settings| settings.reminders.retain(|reminder| reminder.id != id))
    }

    pub fn set_reminder_enabled(&self, id: String, enabled: bool) -> Result<AppSettings, String> {
        self.update(|settings| {
            if let Some(reminder) = settings.reminders.iter_mut().find(|item| item.id == id) {
                reminder.enabled = enabled;
                if !enabled {
                    reminder.paused_until = None;
                }
            }
        })
    }

    pub fn set_reminder_pause(
        &self,
        id: String,
        paused_until: Option<String>,
    ) -> Result<AppSettings, String> {
        self.update(|settings| {
            if let Some(reminder) = settings.reminders.iter_mut().find(|item| item.id == id) {
                reminder.paused_until = normalize_pause(paused_until);
            }
        })
    }

    pub fn pause_enabled_reminders_until(
        &self,
        paused_until: String,
    ) -> Result<AppSettings, String> {
        self.update(|settings| {
            let paused_until = normalize_pause(Some(paused_until));
            for reminder in &mut settings.reminders {
                if reminder.enabled {
                    reminder.paused_until = paused_until.clone();
                }
            }
        })
    }

    pub fn reset_all(&self) -> Result<AppSettings, String> {
        self.update(|settings| *settings = AppSettings::default())
    }

    pub fn reset_main_position(&self) -> Result<AppSettings, String> {
        self.update(|settings| settings.main_position = None)
    }

    pub fn reset_settings_window_bounds(&self) -> Result<AppSettings, String> {
        self.update(|settings| settings.settings_window_bounds = None)
    }

    pub fn save_main_position_throttled(
        &self,
        position: SavedPosition,
    ) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        let now = Instant::now();
        if data
            .last_position_save
            .is_some_and(|last| now.duration_since(last) < POSITION_SAVE_INTERVAL)
        {
            data.settings.main_position = Some(position);
            return Ok(data.settings.clone());
        }
        data.last_position_save = Some(now);
        data.settings.main_position = Some(position);
        let settings = data.settings.clone();
        drop(data);
        self.write(&settings)?;
        Ok(settings)
    }

    pub fn save_settings_window_bounds_throttled(
        &self,
        bounds: SavedWindowBounds,
    ) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        let now = Instant::now();
        let bounds = normalize_settings_window_bounds(bounds);
        if data
            .last_settings_window_save
            .is_some_and(|last| now.duration_since(last) < POSITION_SAVE_INTERVAL)
        {
            data.settings.settings_window_bounds = Some(bounds);
            return Ok(data.settings.clone());
        }
        data.last_settings_window_save = Some(now);
        data.settings.settings_window_bounds = Some(bounds);
        let settings = data.settings.clone();
        drop(data);
        self.write(&settings)?;
        Ok(settings)
    }

    fn update(&self, mutate: impl FnOnce(&mut AppSettings)) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        mutate(&mut data.settings);
        let settings = data.settings.clone();
        drop(data);
        self.write(&settings)?;
        Ok(settings)
    }

    fn persist(&self) -> Result<(), String> {
        self.write(&self.get()?)
    }

    fn write(&self, settings: &AppSettings) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(
            &self.path,
            serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SettingsData>, String> {
        self.inner
            .lock()
            .map_err(|_| "应用设置暂时不可用".to_string())
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join(SETTINGS_FILE))
}

#[cfg(test)]
#[path = "settings/tests.rs"]
mod tests;
