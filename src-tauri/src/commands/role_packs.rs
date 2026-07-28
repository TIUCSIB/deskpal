use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{
    reminder,
    role_packs::{self, InstalledRole},
    settings::{SettingsState, DEFAULT_PET_ROLE},
};

use super::settings::emit_settings;

fn require_settings_window(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "settings" {
        Ok(())
    } else {
        Err("该操作只能在设置窗口中执行。".to_string())
    }
}

#[tauri::command]
pub fn list_installed_role_packs(app: AppHandle) -> Result<Vec<InstalledRole>, String> {
    role_packs::list(&app)
}

#[tauri::command]
pub async fn install_role_pack(
    app: AppHandle,
    window: WebviewWindow,
) -> Result<Option<InstalledRole>, String> {
    require_settings_window(&window)?;
    let selected = app
        .dialog()
        .file()
        .set_parent(&window)
        .add_filter("DeskPal 角色资源包", &["zip"])
        .blocking_pick_file();
    let path = match selected {
        Some(FilePath::Path(path)) => path,
        Some(FilePath::Url(_)) => return Err("仅支持本机文件路径。".to_string()),
        None => return Ok(None),
    };
    let app_handle = app.clone();
    let installed =
        tauri::async_runtime::spawn_blocking(move || role_packs::install(&app_handle, &path))
            .await
            .map_err(|_| "安装角色资源包时任务中断。".to_string())??;
    Ok(Some(installed))
}

#[tauri::command]
pub fn remove_role_pack(
    app: AppHandle,
    window: WebviewWindow,
    settings: State<'_, SettingsState>,
    role_id: String,
) -> Result<(), String> {
    require_settings_window(&window)?;
    let selected = settings.get()?.pet_role;
    if selected != role_id {
        return role_packs::remove(&app, &role_id);
    }

    let fallback = settings.set_validated_pet_role(DEFAULT_PET_ROLE.to_string())?;
    if let Err(error) = reminder::sync_from_settings(&app) {
        let _ = settings.set_validated_pet_role(role_id.clone());
        return Err(error);
    }
    if let Err(error) = role_packs::remove(&app, &role_id) {
        let restored = settings.set_validated_pet_role(role_id)?;
        if let Err(sync_error) = reminder::sync_from_settings(&app) {
            eprintln!("恢复角色后的提醒同步失败: {sync_error}");
        }
        emit_settings(&app, &restored);
        return Err(error);
    }
    emit_settings(&app, &fallback);
    Ok(())
}
