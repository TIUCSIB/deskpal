use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

const SETTINGS_FILE: &str = "settings.json";
const POSITION_SAVE_INTERVAL: Duration = Duration::from_millis(500);
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
pub struct ReminderSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_reminder_message")]
    pub message: String,
    #[serde(default = "default_reminder_interval_minutes")]
    pub interval_minutes: u32,
    #[serde(default = "default_reminder_snooze_minutes")]
    pub snooze_minutes: u32,
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
    pub reminder: ReminderSettings,
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
impl Default for ReminderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            message: default_reminder_message(),
            interval_minutes: default_reminder_interval_minutes(),
            snooze_minutes: default_reminder_snooze_minutes(),
        }
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
            reminder: ReminderSettings::default(),
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
fn normalize_pet_role(role: String) -> String {
    match role.as_str() {
        "guga" | "monthly-salary-cat" | "broom-witch" => role,
        _ => default_pet_role(),
    }
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
            serde_json::from_str(&fs::read_to_string(&path).map_err(|e| e.to_string())?)
                .unwrap_or_default()
        } else {
            AppSettings::default()
        };
        settings.pet_role = normalize_pet_role(settings.pet_role);
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
    pub fn set_reminder_enabled(&self, enabled: bool) -> Result<AppSettings, String> {
        self.update(|s| s.reminder.enabled = enabled)
    }
    pub fn set_reminder_message(&self, message: String) -> Result<AppSettings, String> {
        self.update(|s| s.reminder.message = normalize_reminder_message(message))
    }
    pub fn set_reminder_interval(&self, minutes: u32) -> Result<AppSettings, String> {
        self.update(|s| s.reminder.interval_minutes = normalize_minutes(minutes))
    }
    pub fn set_reminder_snooze_minutes(&self, minutes: u32) -> Result<AppSettings, String> {
        self.update(|s| s.reminder.snooze_minutes = normalize_minutes(minutes))
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
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(
            &self.path,
            serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
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
