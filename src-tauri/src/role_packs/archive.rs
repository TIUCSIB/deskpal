use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use zip::ZipArchive;

use super::{manifest, RolePackManifest, RolePackRole};
use manifest::{RolePackAnimation, RolePackSpritesheet};

const MANIFEST_FILE: &str = "manifest.json";
const LEGACY_PET_FILE: &str = "pet.json";
const LEGACY_IMAGE_WIDTH: u32 = 1536;
const LEGACY_IMAGE_HEIGHT: u32 = 1872;
const LEGACY_FRAME_WIDTH: u32 = 192;
const LEGACY_FRAME_HEIGHT: u32 = 208;
const MAX_ARCHIVE_BYTES: u64 = 12 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyPetManifest {
    id: String,
    display_name: String,
    description: String,
    #[serde(default)]
    kind: String,
    spritesheet_path: String,
}

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
        .is_some_and(|name| name.ends_with(".deskpal-role.zip") || name.ends_with(".zip"))
    {
        return Err("仅支持 ZIP 格式的角色资源包。".to_string());
    }

    let mut archive =
        ZipArchive::new(File::open(archive_path).map_err(|_| "无法打开角色资源包。".to_string())?)
            .map_err(|_| "角色资源包不是有效的 ZIP 文件。".to_string())?;
    if archive.len() != 2 {
        return Err(
            "角色资源包只能包含 pet.json 或 manifest.json 与一个 PNG 或 WebP 精灵图。".to_string(),
        );
    }

    let mut metadata_content = None;
    let mut metadata_name = None;
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
        let max_entry_size = if matches!(name.as_str(), MANIFEST_FILE | LEGACY_PET_FILE) {
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
            MANIFEST_FILE | LEGACY_PET_FILE if metadata_content.is_none() => {
                metadata_name = Some(name);
                metadata_content = Some(content);
            }
            "spritesheet.png" | "spritesheet.webp" if image.is_none() => {
                image = Some((name, content));
            }
            _ => return Err("角色资源包包含重复或不受支持的文件。".to_string()),
        }
    }

    let metadata_content =
        metadata_content.ok_or_else(|| "角色资源包缺少 pet.json 或 manifest.json。".to_string())?;
    let metadata_name = metadata_name.expect("角色包元数据名称应与内容同时存在");
    let (image_name, image_content) = image.ok_or_else(|| "角色资源包缺少精灵图。".to_string())?;
    let (width, height) = image_dimensions(&image_name, &image_content)?;
    let role = parse_role_metadata(
        &metadata_name,
        &metadata_content,
        &image_name,
        width,
        height,
    )?;
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

fn parse_role_metadata(
    name: &str,
    content: &[u8],
    image_name: &str,
    image_width: u32,
    image_height: u32,
) -> Result<RolePackRole, String> {
    match name {
        MANIFEST_FILE => parse_manifest(content),
        LEGACY_PET_FILE => parse_legacy_pet(content, image_name, image_width, image_height),
        _ => Err("角色资源包元数据格式无效。".to_string()),
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

fn parse_legacy_pet(
    content: &[u8],
    image_name: &str,
    image_width: u32,
    image_height: u32,
) -> Result<RolePackRole, String> {
    let pet: LegacyPetManifest = serde_json::from_slice(content)
        .map_err(|_| "pet.json 格式无效或包含不受支持的字段。".to_string())?;
    if pet.spritesheet_path != image_name {
        return Err("pet.json 中的 spritesheetPath 必须匹配资源包内的精灵图文件。".to_string());
    }
    if image_width != LEGACY_IMAGE_WIDTH || image_height != LEGACY_IMAGE_HEIGHT {
        return Err("pet.json 兼容格式仅支持 1536 × 1872 的固定精灵图布局。".to_string());
    }
    manifest::validate_role(RolePackRole {
        id: pet.id,
        display_name: pet.display_name,
        description: pet.description,
        kind: pet.kind,
        spritesheet: RolePackSpritesheet {
            width: LEGACY_IMAGE_WIDTH,
            height: LEGACY_IMAGE_HEIGHT,
            frame_width: LEGACY_FRAME_WIDTH,
            frame_height: LEGACY_FRAME_HEIGHT,
            row_gap: 0,
            crop: None,
            animations: legacy_animations(),
        },
    })
}

fn legacy_animations() -> Vec<RolePackAnimation> {
    [
        ("Idle", 0, 6, 4),
        ("RunRight", 1, 8, 6),
        ("RunLeft", 2, 8, 6),
        ("Waving", 3, 4, 5),
        ("Jumping", 4, 5, 5),
        ("Failed", 5, 8, 5),
        ("Waiting", 6, 6, 3),
        ("Running", 7, 6, 6),
        ("Review", 8, 6, 4),
    ]
    .into_iter()
    .map(|(name, row, frames, fps)| RolePackAnimation {
        name: name.to_string(),
        row,
        frames,
        fps,
    })
    .collect()
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
    if content.len() < 12 || &content[..4] != b"RIFF" || &content[8..12] != b"WEBP" {
        return Err("精灵图不是有效 WebP 文件。".to_string());
    }
    match &content[12..16] {
        b"VP8X" if content.len() >= 30 => Ok((
            1 + u32::from_le_bytes([content[24], content[25], content[26], 0]),
            1 + u32::from_le_bytes([content[27], content[28], content[29], 0]),
        )),
        b"VP8L" if content.len() >= 25 && content[20] == 0x2f => {
            let width = 1 + u32::from(content[21] | ((content[22] & 0x3f) << 8));
            let height = 1 + u32::from(
                (content[22] >> 6) | (content[23] << 2) | ((content[24] & 0x0f) << 10),
            );
            Ok((width, height))
        }
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
    fn reads_lossless_webp_dimensions() {
        let content = b"RIFF\xc8\x7b\x1a\x00WEBPVP8L\xbb\x7b\x1a\x00\x2f\xff\xc5\xd3\x11";
        assert_eq!(webp_dimensions(content), Ok((1536, 1872)));
        assert!(webp_dimensions(b"RIFF\0\0\0\0WEBPVP8L\0\0\0\0\0").is_err());
    }

    #[test]
    fn normalizes_legacy_pet_metadata_to_the_fixed_layout() {
        let role = parse_legacy_pet(
            br#"{
                "id":"tiny-crt",
                "displayName":"Tiny CRT",
                "description":"A tiny terminal monitor.",
                "spritesheetPath":"spritesheet.webp",
                "kind":"object"
            }"#,
            "spritesheet.webp",
            LEGACY_IMAGE_WIDTH,
            LEGACY_IMAGE_HEIGHT,
        )
        .expect("legacy metadata is valid");

        assert_eq!(role.spritesheet.frame_width, LEGACY_FRAME_WIDTH);
        assert_eq!(role.spritesheet.animations.len(), 9);
        assert_eq!(role.spritesheet.animations[0].name, "Idle");
        assert_eq!(role.spritesheet.animations[8].name, "Review");
    }

    #[test]
    fn rejects_legacy_metadata_with_a_wrong_image_path_or_layout() {
        let content = br#"{
            "id":"tiny-crt",
            "displayName":"Tiny CRT",
            "description":"A tiny terminal monitor.",
            "spritesheetPath":"other.webp"
        }"#;
        assert!(parse_legacy_pet(
            content,
            "spritesheet.webp",
            LEGACY_IMAGE_WIDTH,
            LEGACY_IMAGE_HEIGHT,
        )
        .is_err());
        assert!(parse_legacy_pet(content, "other.webp", 512, 512,).is_err());
    }
}
