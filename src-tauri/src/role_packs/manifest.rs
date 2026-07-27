use std::collections::HashSet;

use serde::{Deserialize, Serialize};

const MAX_ID_LENGTH: usize = 48;
const MAX_DISPLAY_NAME_LENGTH: usize = 32;
const MAX_DESCRIPTION_LENGTH: usize = 160;
const MAX_KIND_LENGTH: usize = 24;
const MAX_ANIMATIONS: usize = 16;
const MAX_FRAMES: u32 = 24;
const MAX_FPS: u32 = 30;
const MAX_IMAGE_DIMENSION: u32 = 4096;
const MAX_IMAGE_PIXELS: u64 = 8_388_608;
const BUILT_IN_ROLE_IDS: [&str; 3] = ["guga", "monthly-salary-cat", "broom-witch"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RolePackRole {
    pub id: String,
    pub display_name: String,
    pub description: String,
    #[serde(default)]
    pub kind: String,
    pub spritesheet: RolePackSpritesheet,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RolePackSpritesheet {
    pub width: u32,
    pub height: u32,
    pub frame_width: u32,
    pub frame_height: u32,
    #[serde(default)]
    pub row_gap: u32,
    #[serde(default)]
    pub crop: Option<RolePackCrop>,
    pub animations: Vec<RolePackAnimation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RolePackCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RolePackAnimation {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

pub fn validate_role(role: RolePackRole) -> Result<RolePackRole, String> {
    validate_id(&role.id)?;
    validate_text("角色名称", &role.display_name, MAX_DISPLAY_NAME_LENGTH)?;
    validate_text("角色说明", &role.description, MAX_DESCRIPTION_LENGTH)?;
    if !role.kind.is_empty() {
        validate_text("角色种类", &role.kind, MAX_KIND_LENGTH)?;
    }
    validate_spritesheet(&role.spritesheet)?;
    Ok(role)
}

pub fn validate_image_and_role(
    role: &RolePackRole,
    image_width: u32,
    image_height: u32,
) -> Result<(), String> {
    if role.spritesheet.width != image_width || role.spritesheet.height != image_height {
        return Err("精灵图实际尺寸与 manifest.json 不一致。".to_string());
    }
    validate_image_dimensions(image_width, image_height)?;
    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > MAX_ID_LENGTH
        || BUILT_IN_ROLE_IDS.contains(&id)
        || !id.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || (byte == b'-' && index > 0 && index + 1 < id.len())
        })
    {
        return Err("角色 ID 必须是非内置的小写 ASCII slug。".to_string());
    }
    Ok(())
}

fn validate_text(field_name: &str, value: &str, max_length: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().count() > max_length || contains_unsafe_text(value)
    {
        return Err(format!(
            "{field_name}不能为空、不能过长且不能包含 URL 或控制字符。"
        ));
    }
    Ok(())
}

fn contains_unsafe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.chars().any(char::is_control)
        || ["http:", "https:", "data:", "file:", "javascript:", "<", ">"]
            .iter()
            .any(|forbidden| lower.contains(forbidden))
}

fn validate_spritesheet(spritesheet: &RolePackSpritesheet) -> Result<(), String> {
    validate_image_dimensions(spritesheet.width, spritesheet.height)?;
    if spritesheet.frame_width == 0
        || spritesheet.frame_height == 0
        || spritesheet.frame_width > spritesheet.width
        || spritesheet.frame_height > spritesheet.height
    {
        return Err("精灵图帧尺寸无效。".to_string());
    }
    let crop = spritesheet.crop.as_ref();
    if let Some(crop) = crop {
        if crop.width == 0
            || crop.height == 0
            || crop
                .x
                .checked_add(crop.width)
                .is_none_or(|right| right > spritesheet.width)
            || crop
                .y
                .checked_add(crop.height)
                .is_none_or(|bottom| bottom > spritesheet.height)
        {
            return Err("精灵图裁剪区域超出图片边界。".to_string());
        }
    }
    if spritesheet.animations.is_empty() || spritesheet.animations.len() > MAX_ANIMATIONS {
        return Err("角色包必须包含 1 至 16 个动画。".to_string());
    }
    let mut names = HashSet::new();
    let mut has_idle = false;
    for animation in &spritesheet.animations {
        validate_animation(animation, spritesheet, crop)?;
        if !names.insert(animation.name.to_ascii_lowercase()) {
            return Err("角色包不能包含重名动画。".to_string());
        }
        has_idle |= animation.name == "Idle";
    }
    if !has_idle {
        return Err("角色包必须包含 Idle 动画。".to_string());
    }
    Ok(())
}

fn validate_image_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0
        || height == 0
        || width > MAX_IMAGE_DIMENSION
        || height > MAX_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err("精灵图尺寸超出安全限制。".to_string());
    }
    Ok(())
}

fn validate_animation(
    animation: &RolePackAnimation,
    spritesheet: &RolePackSpritesheet,
    crop: Option<&RolePackCrop>,
) -> Result<(), String> {
    if animation.name.is_empty()
        || animation.name.len() > 24
        || !animation
            .name
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
        || animation.frames == 0
        || animation.frames > MAX_FRAMES
        || animation.fps == 0
        || animation.fps > MAX_FPS
    {
        return Err("动画名称、帧数或帧率无效。".to_string());
    }
    let origin_x = crop.map_or(0, |value| value.x);
    let origin_y = crop.map_or(0, |value| value.y);
    let available_width = crop.map_or(spritesheet.width, |value| value.width);
    let available_height = crop.map_or(spritesheet.height, |value| value.height);
    let frames_width = animation
        .frames
        .checked_mul(spritesheet.frame_width)
        .ok_or_else(|| "动画帧宽度溢出。".to_string())?;
    let row_offset = animation
        .row
        .checked_mul(
            spritesheet
                .frame_height
                .checked_add(spritesheet.row_gap)
                .ok_or_else(|| "动画行距溢出。".to_string())?,
        )
        .ok_or_else(|| "动画行位置溢出。".to_string())?;
    if frames_width > available_width
        || row_offset
            .checked_add(spritesheet.frame_height)
            .is_none_or(|bottom| bottom > available_height)
        || origin_x
            .checked_add(frames_width)
            .is_none_or(|right| right > spritesheet.width)
        || origin_y
            .checked_add(row_offset)
            .and_then(|top| top.checked_add(spritesheet.frame_height))
            .is_none_or(|bottom| bottom > spritesheet.height)
    {
        return Err("动画几何超出精灵图可用区域。".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role() -> RolePackRole {
        RolePackRole {
            id: "custom-cat".to_string(),
            display_name: "自定义猫".to_string(),
            description: "一只安静的自定义猫。".to_string(),
            kind: "猫咪".to_string(),
            spritesheet: RolePackSpritesheet {
                width: 512,
                height: 512,
                frame_width: 128,
                frame_height: 128,
                row_gap: 0,
                crop: None,
                animations: vec![RolePackAnimation {
                    name: "Idle".to_string(),
                    row: 0,
                    frames: 4,
                    fps: 8,
                }],
            },
        }
    }

    #[test]
    fn accepts_passive_role_metadata() {
        let role = role();
        assert!(validate_role(role.clone()).is_ok());
        assert!(validate_image_and_role(&role, 512, 512).is_ok());
    }

    #[test]
    fn rejects_builtin_ids_urls_and_missing_idle_animation() {
        let mut built_in = role();
        built_in.id = "guga".to_string();
        assert!(validate_role(built_in).is_err());

        let mut url = role();
        url.description = "https://example.com".to_string();
        assert!(validate_role(url).is_err());

        let mut no_idle = role();
        no_idle.spritesheet.animations[0].name = "Walk".to_string();
        assert!(validate_role(no_idle).is_err());
    }

    #[test]
    fn rejects_out_of_bounds_animation_geometry() {
        let mut invalid = role();
        invalid.spritesheet.animations[0].frames = 5;
        assert!(validate_role(invalid).is_err());
    }
}
