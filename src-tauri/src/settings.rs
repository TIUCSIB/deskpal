use std::{fs, path::PathBuf, sync::Mutex, time::{Duration, Instant}};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppSettings {
    #[serde(default)]
    pub main_position: Option<SavedPosition>,
    #[serde(default)]
    pub settings_window_bounds: Option<SavedWindowBounds>,
    #[serde(default = "default_pet_scale")]
    pub pet_scale: f64,
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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            main_position: None,
            settings_window_bounds: None,
            pet_scale: default_pet_scale(),
            info_mode: InfoMode::default(),
            size_locked: false,
            shortcut_enabled: default_shortcut_enabled(),
            launch_at_startup: false,
            main_window_always_on_top: default_always_on_top(),
            main_window_show_in_taskbar: false,
            chat_shortcut: default_chat_shortcut(),
        }
    }
}

fn default_pet_scale() -> f64 {
    DEFAULT_PET_SCALE
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

fn normalize_settings_window_bounds(bounds: SavedWindowBounds) -> SavedWindowBounds {
    SavedWindowBounds {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(MIN_SETTINGS_WINDOW_WIDTH),
        height: bounds.height.max(MIN_SETTINGS_WINDOW_HEIGHT),
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
        let settings = if path.exists() {
            let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            serde_json::from_str(&text).unwrap_or_default()
        } else {
            AppSettings::default()
        };
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

    pub fn reset_all(&self) -> Result<AppSettings, String> {
        self.update(|settings| *settings = AppSettings::default())
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

    pub fn reset_main_position(&self) -> Result<AppSettings, String> {
        self.update(|settings| settings.main_position = None)
    }

    pub fn reset_settings_window_bounds(&self) -> Result<AppSettings, String> {
        self.update(|settings| settings.settings_window_bounds = None)
    }

    pub fn save_settings_window_bounds_throttled(
        &self,
        bounds: SavedWindowBounds,
    ) -> Result<AppSettings, String> {
        let mut data = self.lock()?;
        let now = Instant::now();
        let normalized_bounds = normalize_settings_window_bounds(bounds);
        if data
            .last_settings_window_save
            .is_some_and(|last| now.duration_since(last) < POSITION_SAVE_INTERVAL)
        {
            data.settings.settings_window_bounds = Some(normalized_bounds);
            return Ok(data.settings.clone());
        }
        data.last_settings_window_save = Some(now);
        data.settings.settings_window_bounds = Some(normalized_bounds);
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
        let text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
        fs::write(&self.path, text).map_err(|error| error.to_string())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SettingsData>, String> {
        self.inner
            .lock()
            .map_err(|_| "应用设置暂时不可用".to_string())
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    Ok(dir.join(SETTINGS_FILE))
}
#[cfg(test)] #[path = "settings/tests.rs"] mod tests;
