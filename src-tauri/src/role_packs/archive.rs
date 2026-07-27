use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use zip::ZipArchive;

use super::{manifest, RolePackManifest, RolePackRole};

const MANIFEST_FILE: &str = "manifest.json";
const MAX_ARCHIVE_BYTES: u64 = 12 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

pub(super) struct ValidatedRolePack {
    pub(super) role: RolePackRole,
    pub(super) image_name: String,
    pub(super) image_content: Vec<u8>,
}

pub(super) fn read_role_pack(archive_path: &Path) -> Result<ValidatedRolePack, String> {
    let metadata =
        fs::metadata(archive_path).map_err(|_| "无法读取所选角色资源包。".to_string())?;
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err("角色资源包超过 12 MB 限制。".to_string());
    }
    if !archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".deskpal-role.zip"))
    {
        return Err("仅支持 .deskpal-role.zip 角色资源包。".to_string());
    }

    let mut archive =
        ZipArchive::new(File::open(archive_path).map_err(|_| "无法打开角色资源包。".to_string())?)
            .map_err(|_| "角色资源包不是有效的 ZIP 文件。".to_string())?;
    if archive.len() != 2 {
        return Err("角色资源包只能包含 manifest.json 与一个 PNG 或 WebP 精灵图。".to_string());
    }

    let mut manifest_content = None;
    let mut image = None;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| "无法读取角色资源包内容。".to_string())?;
        let name = entry.name().to_string();
        validate_entry_name(&name)?;
        if entry.is_dir() || entry.is_symlink() {
            return Err("角色资源包不能包含目录或链接。".to_string());
        }
        let max_entry_size = if name == MANIFEST_FILE {
            MAX_MANIFEST_BYTES
        } else {
            MAX_IMAGE_BYTES
        };
        if entry.size() > max_entry_size as u64 || entry.compressed_size() > MAX_ARCHIVE_BYTES {
            return Err("角色资源包包含过大的文件。".to_string());
        }
        let mut content = Vec::with_capacity(entry.size() as usize);
        entry
            .take(max_entry_size as u64 + 1)
            .read_to_end(&mut content)
            .map_err(|_| "读取角色资源包内容失败。".to_string())?;
        if content.len() > max_entry_size {
            return Err("角色资源包包含过大的文件。".to_string());
        }
        match name.as_str() {
            MANIFEST_FILE if manifest_content.is_none() => manifest_content = Some(content),
            "spritesheet.png" | "spritesheet.webp" if image.is_none() => {
                image = Some((name, content))
            }
            _ => return Err("角色资源包包含重复或不受支持的文件。".to_string()),
        }
    }

    let manifest_content =
        manifest_content.ok_or_else(|| "角色资源包缺少 manifest.json。".to_string())?;
    let (image_name, image_content) = image.ok_or_else(|| "角色资源包缺少精灵图。".to_string())?;
    let role = parse_manifest(&manifest_content)?;
    let (width, height) = image_dimensions(&image_name, &image_content)?;
    manifest::validate_image_and_role(&role, width, height)?;
    Ok(ValidatedRolePack {
        role,
        image_name,
        image_content,
    })
}

pub(super) fn image_dimensions(name: &str, content: &[u8]) -> Result<(u32, u32), String> {
    match name {
        "spritesheet.png" => png_dimensions(content),
        "spritesheet.webp" => webp_dimensions(content),
        _ => Err("仅允许 PNG 或 WebP 精灵图。".to_string()),
    }
}

fn parse_manifest(content: &[u8]) -> Result<RolePackRole, String> {
    let manifest: RolePackManifest<RolePackRole> = serde_json::from_slice(content)
        .map_err(|_| "manifest.json 格式无效或包含不受支持的字段。".to_string())?;
    if manifest.schema_version != 1 {
        return Err("不支持该角色资源包版本。".to_string());
    }
    manifest::validate_role(manifest.role)
}

fn validate_entry_name(name: &str) -> Result<(), String> {
    if name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || Path::new(name).is_absolute()
    {
        return Err("角色资源包包含不安全的文件路径。".to_string());
    }
    Ok(())
}

fn png_dimensions(content: &[u8]) -> Result<(u32, u32), String> {
    if content.len() < 24 || &content[..8] != b"\x89PNG\r\n\x1a\n" || &content[12..16] != b"IHDR" {
        return Err("精灵图不是有效 PNG 文件。".to_string());
    }
    Ok((
        u32::from_be_bytes(content[16..20].try_into().unwrap()),
        u32::from_be_bytes(content[20..24].try_into().unwrap()),
    ))
}

fn webp_dimensions(content: &[u8]) -> Result<(u32, u32), String> {
    if content.len() < 30 || &content[..4] != b"RIFF" || &content[8..12] != b"WEBP" {
        return Err("精灵图不是有效 WebP 文件。".to_string());
    }
    match &content[12..16] {
        b"VP8X" if content.len() >= 30 => Ok((
            1 + u32::from_le_bytes([content[24], content[25], content[26], 0]),
            1 + u32::from_le_bytes([content[27], content[28], content[29], 0]),
        )),
        b"VP8 " if content.len() >= 30 && content[23..26] == [0x9d, 0x01, 0x2a] => Ok((
            u16::from_le_bytes(content[26..28].try_into().unwrap()) as u32 & 0x3fff,
            u16::from_le_bytes(content[28..30].try_into().unwrap()) as u32 & 0x3fff,
        )),
        _ => Err("仅支持包含尺寸信息的 WebP 精灵图。".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_archive_paths() {
        for name in [
            "../manifest.json",
            "C:\\manifest.json",
            "assets/spritesheet.png",
        ] {
            assert!(validate_entry_name(name).is_err());
        }
        assert!(validate_entry_name("manifest.json").is_ok());
    }

    #[test]
    fn reads_png_dimensions_only_from_a_valid_header() {
        let mut image = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        image.extend_from_slice(&1536_u32.to_be_bytes());
        image.extend_from_slice(&1872_u32.to_be_bytes());
        assert_eq!(png_dimensions(&image), Ok((1536, 1872)));
        assert!(png_dimensions(b"not-an-image").is_err());
    }
}
