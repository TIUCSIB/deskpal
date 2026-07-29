use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use tauri::{AppHandle, Manager};

use super::{
    default_reminder_message, default_reminder_snooze_minutes, normalize_quiet_hours, AppSettings,
    SETTINGS_SCHEMA_VERSION,
};

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

impl super::state::SettingsState {
    pub(crate) fn persist(&self) -> Result<(), String> {
        self.write(&self.get()?)
    }

    pub(crate) fn write(&self, settings: &AppSettings) -> Result<(), String> {
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
}

pub(crate) fn recover_interrupted_write(path: &std::path::Path) -> Result<(), String> {
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

pub(crate) fn normalize_loaded_settings(app: &AppHandle, settings: &mut AppSettings) {
    settings.schema_version = SETTINGS_SCHEMA_VERSION;
    let stored_role = std::mem::take(&mut settings.pet_role);
    settings.pet_role = if crate::role_packs::is_valid_role(app, &stored_role) {
        stored_role
    } else {
        super::normalize::normalize_pet_role(stored_role)
    };
    normalize_settings(settings);
}

pub(crate) fn normalize_settings(settings: &mut AppSettings) {
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

pub(crate) fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json"))
}
