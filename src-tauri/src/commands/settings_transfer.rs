use std::fs;

use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{
    role_packs,
    settings::{PortableSettingsEnvelope, SettingsState},
    windowing,
};

use super::settings::finish_update;

fn require_settings_window(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == windowing::SETTINGS_WINDOW {
        Ok(())
    } else {
        Err("该操作只能在设置窗口中执行。".to_string())
    }
}

fn local_path(path: FilePath) -> Result<std::path::PathBuf, String> {
    match path {
        FilePath::Path(path) => Ok(path),
        FilePath::Url(_) => Err("仅支持本机文件路径。".to_string()),
    }
}

#[tauri::command]
pub async fn export_portable_settings(
    app: AppHandle,
    window: WebviewWindow,
    settings: State<'_, SettingsState>,
) -> Result<bool, String> {
    require_settings_window(&window)?;
    let selected = app
        .dialog()
        .file()
        .set_parent(&window)
        .set_file_name("deskpal-settings.deskpal-settings.json")
        .add_filter("DeskPal 设置", &["deskpal-settings.json"])
        .blocking_save_file();
    let Some(path) = selected else {
        return Ok(false);
    };
    let export = settings.portable_export()?;
    let content =
        serde_json::to_vec_pretty(&export).map_err(|_| "无法生成设置导出文件。".to_string())?;
    let path = local_path(path)?;
    tauri::async_runtime::spawn_blocking(move || fs::write(path, content))
        .await
        .map_err(|_| "导出设置时任务中断。".to_string())?
        .map_err(|_| "写入设置导出文件失败。".to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn import_portable_settings(
    app: AppHandle,
    window: WebviewWindow,
    settings: State<'_, SettingsState>,
) -> Result<bool, String> {
    require_settings_window(&window)?;
    let selected = app
        .dialog()
        .file()
        .set_parent(&window)
        .add_filter("DeskPal 设置", &["deskpal-settings.json"])
        .blocking_pick_file();
    let Some(path) = selected else {
        return Ok(false);
    };
    let path = local_path(path)?;
    let content = tauri::async_runtime::spawn_blocking(move || fs::read_to_string(path))
        .await
        .map_err(|_| "读取设置导入文件时任务中断。".to_string())?
        .map_err(|_| "读取设置导入文件失败。".to_string())?;
    let imported = PortableSettingsEnvelope::parse(&content)?;
    let valid_role = if role_packs::is_valid_role(&app, &imported.settings.pet_role) {
        imported.settings.pet_role.clone()
    } else {
        crate::settings::DEFAULT_PET_ROLE.to_string()
    };
    let previous = settings.get()?;
    let updated = settings.import_portable(imported, valid_role)?;
    let result = (|| {
        windowing::apply_main_window_settings(&app, &updated)?;
        windowing::sync_info_window_visibility(&app)?;
        finish_update(&app, updated.clone())?;
        Ok(())
    })();
    if let Err(error) = result {
        let restored = settings.restore(previous)?;
        let _ = windowing::apply_main_window_settings(&app, &restored);
        let _ = windowing::sync_info_window_visibility(&app);
        let _ = finish_update(&app, restored);
        return Err(error);
    }
    Ok(true)
}

#[tauri::command]
pub fn complete_settings_onboarding(
    app: AppHandle,
    window: WebviewWindow,
    settings: State<'_, SettingsState>,
) -> Result<crate::settings::AppSettings, String> {
    require_settings_window(&window)?;
    finish_update(&app, settings.complete_onboarding()?)
}
