use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{AppSettings, InfoMode, QuietHours, Reminder};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const PORTABLE_SETTINGS_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PortableSettings {
    pub pet_scale: f64,
    pub pet_role: String,
    pub info_mode: InfoMode,
    pub size_locked: bool,
    pub main_window_always_on_top: bool,
    pub main_window_left_click_passthrough: bool,
    pub reminders: Vec<Reminder>,
    pub quiet_hours: QuietHours,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PortableSettingsEnvelope {
    pub format_version: u32,
    pub exported_at: String,
    pub settings: PortableSettings,
}

impl PortableSettingsEnvelope {
    pub fn from_settings(settings: &AppSettings) -> Self {
        Self {
            format_version: PORTABLE_SETTINGS_FORMAT_VERSION,
            exported_at: Utc::now().to_rfc3339(),
            settings: PortableSettings {
                pet_scale: settings.pet_scale,
                pet_role: settings.pet_role.clone(),
                info_mode: settings.info_mode,
                size_locked: settings.size_locked,
                main_window_always_on_top: settings.main_window_always_on_top,
                main_window_left_click_passthrough: settings.main_window_left_click_passthrough,
                reminders: settings.reminders.clone(),
                quiet_hours: settings.quiet_hours.clone(),
            },
        }
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let envelope: Self = serde_json::from_str(content)
            .map_err(|_| "设置导入文件不是有效的 JSON。".to_string())?;
        if envelope.format_version != PORTABLE_SETTINGS_FORMAT_VERSION {
            return Err("不支持该设置导入文件版本。".to_string());
        }
        DateTime::parse_from_rfc3339(&envelope.exported_at)
            .map_err(|_| "设置导入文件缺少有效的导出时间。".to_string())?;
        Ok(envelope)
    }

    pub fn apply_to(self, current: &AppSettings, pet_role: String) -> AppSettings {
        AppSettings {
            schema_version: SETTINGS_SCHEMA_VERSION,
            onboarding_completed: current.onboarding_completed,
            main_position: current.main_position,
            settings_window_bounds: current.settings_window_bounds,
            pet_scale: self.settings.pet_scale,
            pet_role,
            info_mode: self.settings.info_mode,
            size_locked: self.settings.size_locked,
            shortcut_enabled: current.shortcut_enabled,
            launch_at_startup: current.launch_at_startup,
            main_window_always_on_top: self.settings.main_window_always_on_top,
            main_window_show_in_taskbar: current.main_window_show_in_taskbar,
            main_window_left_click_passthrough: self.settings.main_window_left_click_passthrough,
            chat_shortcut: current.chat_shortcut.clone(),
            reminders: self.settings.reminders,
            quiet_hours: self.settings.quiet_hours,
        }
    }
}
