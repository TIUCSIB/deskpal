mod archive;
mod manifest;
mod registry;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

pub use manifest::RolePackRole;
use registry::{InstalledRolePack, RolePackRegistry};

const ROLE_PACKS_DIRECTORY: &str = "role-packs";
const REGISTRY_FILE: &str = "registry.json";
const MANIFEST_FILE: &str = "manifest.json";
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledRole {
    #[serde(flatten)]
    pub role: RolePackRole,
    pub spritesheet_url: String,
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RolePackManifest<T> {
    schema_version: u32,
    role: T,
}

pub fn list(app: &AppHandle) -> Result<Vec<InstalledRole>, String> {
    load_registry(app)?
        .packs
        .into_iter()
        .map(|pack| installed_role(app, pack))
        .collect()
}

pub fn is_builtin_role(role_id: &str) -> bool {
    matches!(role_id, "guga" | "monthly-salary-cat" | "broom-witch")
}

pub fn is_valid_role(app: &AppHandle, role_id: &str) -> bool {
    is_builtin_role(role_id)
        || load_registry(app)
            .map(|registry| {
                registry.packs.into_iter().any(|pack| {
                    pack.role.id == role_id && validate_installed_pack(app, pack).is_ok()
                })
            })
            .unwrap_or(false)
}

pub fn resource_response(
    app: &AppHandle,
    role_id: &str,
) -> Result<(Vec<u8>, &'static str), String> {
    let pack = load_registry(app)?
        .packs
        .into_iter()
        .find(|pack| pack.role.id == role_id)
        .ok_or_else(|| "找不到角色资源。".to_string())?;
    let pack = validate_installed_pack(app, pack)?;
    let content =
        fs::read(pack_image_path(app, &pack)?).map_err(|_| "无法读取角色资源。".to_string())?;
    let (width, height) = archive::image_dimensions(&pack.spritesheet_file, &content)?;
    manifest::validate_image_and_role(&pack.role, width, height)?;
    let content_type = match pack.spritesheet_file.as_str() {
        "spritesheet.png" => "image/png",
        "spritesheet.webp" => "image/webp",
        _ => return Err("角色资源格式无效。".to_string()),
    };
    Ok((content, content_type))
}

pub fn install(app: &AppHandle, archive_path: &Path) -> Result<InstalledRole, String> {
    let pack = archive::read_role_pack(archive_path)?;
    let directory = role_packs_directory(app)?;
    fs::create_dir_all(&directory).map_err(|error| format!("创建角色包目录失败: {error}"))?;
    let staging = directory.join(format!(".staging-{}", pack.role.id));
    remove_directory_if_exists(&staging, "无法清理旧的角色包临时目录。")?;
    fs::create_dir(&staging).map_err(|_| "无法创建角色包临时目录。".to_string())?;

    let result = write_installation(&staging, &pack.role, &pack.image_name, &pack.image_content)
        .and_then(|_| commit_installation(app, &pack.role, &pack.image_name, &staging));
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    installed_role(
        app,
        InstalledRolePack {
            role: pack.role,
            spritesheet_file: pack.image_name,
        },
    )
}

pub fn remove(app: &AppHandle, role_id: &str) -> Result<(), String> {
    if is_builtin_role(role_id) {
        return Err("内置角色不能删除。".to_string());
    }
    let mut registry = load_registry(app)?;
    let original_registry = registry.clone();
    let pack = original_registry
        .packs
        .iter()
        .find(|pack| pack.role.id == role_id)
        .cloned()
        .ok_or_else(|| "找不到该自定义角色。".to_string())?;
    let pack = validate_installed_pack(app, pack)?;
    let directory = role_packs_directory(app)?;
    let installed = directory.join(&pack.role.id);
    let staging = directory.join(format!(".removing-{}", pack.role.id));
    registry
        .packs
        .retain(|installed| installed.role.id != pack.role.id);
    remove_directory_if_exists(&staging, "无法清理旧的角色包删除临时目录。")?;
    fs::rename(&installed, &staging).map_err(|_| "准备删除角色资源失败。".to_string())?;
    if let Err(error) = write_registry(app, &registry) {
        let _ = fs::rename(&staging, &installed);
        return Err(error);
    }
    if let Err(error) = fs::remove_dir_all(&staging) {
        let _ = write_registry(app, &original_registry);
        let _ = fs::rename(&staging, &installed);
        return Err(format!("删除角色资源文件失败: {error}"));
    }
    Ok(())
}

fn installed_role(app: &AppHandle, pack: InstalledRolePack) -> Result<InstalledRole, String> {
    let pack = validate_installed_pack(app, pack)?;
    Ok(InstalledRole {
        spritesheet_url: spritesheet_url(&pack.role.id),
        role: pack.role,
    })
}

fn validate_installed_pack(
    app: &AppHandle,
    pack: InstalledRolePack,
) -> Result<InstalledRolePack, String> {
    let role = manifest::validate_role(pack.role)?;
    if !matches!(
        pack.spritesheet_file.as_str(),
        "spritesheet.png" | "spritesheet.webp"
    ) {
        return Err("角色包注册表包含不安全的资源路径。".to_string());
    }
    let pack = InstalledRolePack {
        role,
        spritesheet_file: pack.spritesheet_file,
    };
    let metadata = fs::symlink_metadata(pack_image_path(app, &pack)?)
        .map_err(|_| "角色资源文件不存在。".to_string())?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_IMAGE_BYTES as u64 {
        return Err("角色资源文件无效。".to_string());
    }
    Ok(pack)
}

fn commit_installation(
    app: &AppHandle,
    role: &RolePackRole,
    spritesheet_file: &str,
    staging: &Path,
) -> Result<(), String> {
    let directory = role_packs_directory(app)?;
    let destination = directory.join(&role.id);
    let backup = directory.join(format!(".backup-{}", role.id));
    remove_directory_if_exists(&backup, "无法清理角色包备份。")?;
    if destination.exists() {
        fs::rename(&destination, &backup).map_err(|_| "无法备份现有角色包。".to_string())?;
    }
    if let Err(error) = fs::rename(staging, &destination) {
        restore_directory(&backup, &destination);
        return Err(format!("安装角色资源包失败: {error}"));
    }

    let mut registry = load_registry(app)?;
    registry.packs.retain(|pack| pack.role.id != role.id);
    registry.packs.push(InstalledRolePack {
        role: role.clone(),
        spritesheet_file: spritesheet_file.to_string(),
    });
    registry
        .packs
        .sort_by(|left, right| left.role.id.cmp(&right.role.id));
    if let Err(error) = write_registry(app, &registry) {
        let _ = fs::remove_dir_all(&destination);
        restore_directory(&backup, &destination);
        return Err(error);
    }
    if backup.exists() {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(())
}

fn write_installation(
    staging: &Path,
    role: &RolePackRole,
    image_name: &str,
    image_content: &[u8],
) -> Result<(), String> {
    let manifest = RolePackManifest {
        schema_version: 1,
        role,
    };
    write_file(
        &staging.join(MANIFEST_FILE),
        &serde_json::to_vec_pretty(&manifest).map_err(|_| "无法生成角色清单。".to_string())?,
    )?;
    write_file(&staging.join(image_name), image_content)
}

fn load_registry(app: &AppHandle) -> Result<RolePackRegistry, String> {
    registry::load(&registry_path(app)?)
}

fn write_registry(app: &AppHandle, registry: &RolePackRegistry) -> Result<(), String> {
    registry::write(&registry_path(app)?, registry)
}

fn pack_image_path(app: &AppHandle, pack: &InstalledRolePack) -> Result<PathBuf, String> {
    Ok(role_packs_directory(app)?
        .join(&pack.role.id)
        .join(&pack.spritesheet_file))
}

fn role_packs_directory(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|_| "无法定位应用数据目录。".to_string())?
        .join(ROLE_PACKS_DIRECTORY))
}

fn registry_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(role_packs_directory(app)?.join(REGISTRY_FILE))
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    fs::write(path, content).map_err(|_| "无法写入应用数据。".to_string())
}

fn remove_directory_if_exists(path: &Path, error_message: &str) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|_| error_message.to_string())?;
    }
    Ok(())
}

fn restore_directory(backup: &Path, destination: &Path) {
    if backup.exists() {
        let _ = fs::rename(backup, destination);
    }
}

fn spritesheet_url(role_id: &str) -> String {
    #[cfg(target_os = "windows")]
    return format!("http://role-pack.localhost/{role_id}");
    #[cfg(not(target_os = "windows"))]
    format!("role-pack://localhost/{role_id}")
}
