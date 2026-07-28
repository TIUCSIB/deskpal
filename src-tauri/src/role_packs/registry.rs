use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

use super::RolePackRole;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledRolePack {
    #[serde(flatten)]
    pub role: RolePackRole,
    pub spritesheet_file: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct RolePackRegistry {
    pub(super) packs: Vec<InstalledRolePack>,
}

pub(super) fn load(path: &Path) -> Result<RolePackRegistry, String> {
    recover_interrupted_write(path)?;
    if !path.exists() {
        return Ok(RolePackRegistry::default());
    }
    serde_json::from_slice(&fs::read(path).map_err(|_| "读取角色包注册表失败。".to_string())?)
        .map_err(|_| "角色包注册表格式无效。".to_string())
}

pub(super) fn write(path: &Path, registry: &RolePackRegistry) -> Result<(), String> {
    let contents =
        serde_json::to_vec_pretty(registry).map_err(|_| "无法保存角色包注册表。".to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| "角色包注册表路径无效。".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "无法创建角色包目录。".to_string())?;
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    let mut file =
        File::create(&temporary).map_err(|_| "无法创建角色包注册表临时文件。".to_string())?;
    if let Err(error) = file.write_all(&contents).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("写入角色包注册表失败: {error}"));
    }
    drop(file);

    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| "无法清理角色包注册表备份。".to_string())?;
    }
    let had_current = path.exists();
    if had_current {
        fs::rename(path, &backup).map_err(|_| "无法备份角色包注册表。".to_string())?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if had_current {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(format!("替换角色包注册表失败: {error}"));
    }
    if backup.exists() {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

fn recover_interrupted_write(path: &Path) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    if !path.exists() && backup.exists() {
        fs::rename(&backup, path).map_err(|_| "恢复角色包注册表备份失败。".to_string())?;
    }
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|_| "清理角色包注册表临时文件失败。".to_string())?;
    }
    if path.exists() && backup.exists() {
        fs::remove_file(&backup).map_err(|_| "清理角色包注册表备份失败。".to_string())?;
    }
    Ok(())
}
