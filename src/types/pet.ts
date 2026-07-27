/**
 * pet.ts - 精灵表宠物类型定义
 * 定义内置桌宠角色和精灵表动画配置。
 */

export type PetRoleId = 'guga' | 'monthly-salary-cat' | 'broom-witch'

/** 单个动画定义 */
export interface PetAnimation {
  /** 动画名称 */
  name: string
  /** 精灵表中的行索引 */
  row: number
  /** 帧数量 */
  frameCount: number
  /** 播放帧率 */
  fps: number
}

/** 精灵表元数据 */
export interface PetSpritesheet {
  /** 宠物 ID */
  id: string
  /** 显示名称 */
  displayName: string
  /** 精灵表图片宽度（px） */
  imageWidth: number
  /** 精灵表图片高度（px） */
  imageHeight: number
  /** 每帧宽度（px） */
  frameWidth: number
  /** 每帧高度（px） */
  frameHeight: number
  /** 行间距（px） */
  rowGap: number
  /** 可用动画列表 */
  animations: PetAnimation[]
  /** 帧内裁剪配置（px），去除角色周围透明留白 */
  crop?: {
    /** 左侧裁剪像素 */
    left: number
    /** 右侧裁剪像素 */
    right: number
    /** 顶部裁剪像素 */
    top: number
    /** 底部裁剪像素 */
    bottom: number
  }
}

export interface PetRole {
  id: PetRoleId
  displayName: string
  description: string
  kind?: string
  spritesheetUrl: string
  spritesheet: PetSpritesheet
}

/** 心情 → 动画名称映射 */
export type MoodAnimationMap = Record<string, string>
