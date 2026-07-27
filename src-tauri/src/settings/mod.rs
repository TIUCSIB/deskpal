mod normalize;
mod state;

pub use state::SettingsState;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalSize};

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
pub const DEFAULT_QUIET_HOURS_START: &str = "23:00";
pub const DEFAULT_QUIET_HOURS_END: &str = "08:00";

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
pub enum ReminderRepeat {
    Daily,
    Weekdays,
    CustomWeekdays { weekdays: Vec<u8> },
}
impl Default for ReminderRepeat {
    fn default() -> Self {
        Self::Daily
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReminderSchedule {
    Interval {
        interval_minutes: u32,
    },
    FixedTime {
        time: String,
        #[serde(default)]
        repeat: ReminderRepeat,
    },
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
pub struct QuietHours {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_quiet_hours_start")]
    pub start: String,
    #[serde(default = "default_quiet_hours_end")]
    pub end: String,
}
impl Default for QuietHours {
    fn default() -> Self {
        Self {
            enabled: false,
            start: default_quiet_hours_start(),
            end: default_quiet_hours_end(),
        }
    }
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
    #[serde(default)]
    pub quiet_hours: QuietHours,
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
            shortcut_enabled: true,
            launch_at_startup: false,
            main_window_always_on_top: true,
            main_window_show_in_taskbar: false,
            chat_shortcut: default_chat_shortcut(),
            reminders: Vec::new(),
            quiet_hours: QuietHours::default(),
        }
    }
}

pub(crate) fn default_pet_scale() -> f64 {
    DEFAULT_PET_SCALE
}
pub(crate) fn default_pet_role() -> String {
    DEFAULT_PET_ROLE.to_string()
}
pub(crate) fn default_shortcut_enabled() -> bool {
    true
}
pub(crate) fn default_always_on_top() -> bool {
    true
}
pub(crate) fn default_chat_shortcut() -> String {
    DEFAULT_CHAT_SHORTCUT.to_string()
}
pub(crate) fn default_reminder_message() -> String {
    DEFAULT_REMINDER_MESSAGE.to_string()
}
pub(crate) fn default_reminder_interval_minutes() -> u32 {
    DEFAULT_REMINDER_INTERVAL_MINUTES
}
pub(crate) fn default_reminder_snooze_minutes() -> u32 {
    DEFAULT_REMINDER_SNOOZE_MINUTES
}
pub(crate) fn default_quiet_hours_start() -> String {
    DEFAULT_QUIET_HOURS_START.to_string()
}
pub(crate) fn default_quiet_hours_end() -> String {
    DEFAULT_QUIET_HOURS_END.to_string()
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
    fn from(value: SavedWindowBounds) -> Self {
        (
            PhysicalPosition::new(value.x, value.y),
            PhysicalSize::new(value.width, value.height),
        )
    }
}
impl From<PhysicalPosition<i32>> for SavedPosition {
    fn from(value: PhysicalPosition<i32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl From<SavedPosition> for PhysicalPosition<i32> {
    fn from(value: SavedPosition) -> Self {
        Self::new(value.x, value.y)
    }
}

pub(crate) fn normalize_time(time: String) -> String {
    let parts: Vec<_> = time.trim().split(':').collect();
    let parsed = if parts.len() == 2 {
        parts[0]
            .parse::<u32>()
            .ok()
            .zip(parts[1].parse::<u32>().ok())
    } else {
        None
    };
    match parsed {
        Some((hour, minute)) if hour < 24 && minute < 60 => format!("{hour:02}:{minute:02}"),
        _ => "09:00".to_string(),
    }
}
pub(crate) fn normalize_pause(paused_until: Option<String>) -> Option<String> {
    paused_until.and_then(|value| {
        DateTime::parse_from_rfc3339(value.trim())
            .ok()
            .map(|time| time.with_timezone(&Local).to_rfc3339())
    })
}
pub(crate) fn normalize_quiet_hours(mut hours: QuietHours) -> QuietHours {
    hours.start = normalize_time(hours.start);
    hours.end = normalize_time(hours.end);
    hours
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
#[cfg(test)]
mod tests_crud;
