import { computed, ref } from 'vue'
import type { PetRole, PetRoleId, PetSpritesheet } from '@/types/pet'
import gugaSpritesheetUrl from '@/assets/pets/guga/spritesheet.webp'
import monthlySalaryCatSpritesheetUrl from '@/assets/pets/monthly-salary-cat/spritesheet.webp'
import broomWitchSpritesheetUrl from '@/assets/pets/broom-witch/spritesheet.webp'

export const DEFAULT_PET_ROLE: PetRoleId = 'guga'

interface InstalledRoleAnimation {
  name: string
  row: number
  frames: number
  fps: number
}

interface InstalledRoleSpritesheet {
  width: number
  height: number
  frameWidth: number
  frameHeight: number
  rowGap: number
  crop?: { x: number, y: number, width: number, height: number }
  animations: InstalledRoleAnimation[]
}

export interface InstalledPetRole {
  id: PetRoleId
  displayName: string
  description: string
  kind: string
  spritesheetUrl: string
  spritesheet: InstalledRoleSpritesheet
}

const BASE_SPRITESHEET: Omit<PetSpritesheet, 'id' | 'displayName'> = {
  imageWidth: 1536,
  imageHeight: 1872,
  frameWidth: 192,
  frameHeight: 208,
  rowGap: 0,
  animations: [
    { name: 'Idle', row: 0, frameCount: 6, fps: 4 },
    { name: 'RunRight', row: 1, frameCount: 8, fps: 6 },
    { name: 'RunLeft', row: 2, frameCount: 8, fps: 6 },
    { name: 'Waving', row: 3, frameCount: 4, fps: 5 },
    { name: 'Jumping', row: 4, frameCount: 5, fps: 5 },
    { name: 'Failed', row: 5, frameCount: 8, fps: 5 },
    { name: 'Waiting', row: 6, frameCount: 6, fps: 3 },
    { name: 'Running', row: 7, frameCount: 6, fps: 6 },
    { name: 'Review', row: 8, frameCount: 6, fps: 4 },
  ],
}

/** 不可被外部角色包覆盖的内置角色。 */
export const PET_ROLES: PetRole[] = [
  {
    id: 'guga',
    displayName: '咕嘎',
    description: '默认桌宠角色，适合日常陪伴。',
    kind: 'animal',
    spritesheetUrl: gugaSpritesheetUrl,
    spritesheet: { ...BASE_SPRITESHEET, id: 'guga', displayName: '咕嘎' },
  },
  {
    id: 'monthly-salary-cat',
    displayName: '月薪猫',
    description: '一只认真打工、等月薪到账的小猫。',
    kind: 'animal',
    spritesheetUrl: monthlySalaryCatSpritesheetUrl,
    spritesheet: {
      ...BASE_SPRITESHEET,
      id: 'monthly-salary-cat',
      displayName: 'Monthly salary cat（月薪猫）',
    },
  },
  {
    id: 'broom-witch',
    displayName: '琪琪',
    description: '骑着扫帚的小魔女，适合带一点魔法感的桌面。',
    kind: 'person',
    spritesheetUrl: broomWitchSpritesheetUrl,
    spritesheet: { ...BASE_SPRITESHEET, id: 'broom-witch', displayName: 'Broom Witch' },
  },
]

const installedRoles = ref<PetRole[]>([])

/** 当前可选角色；外部角色只会由原生端已验证的元数据转换而来。 */
export const petRoles = computed(() => [...PET_ROLES, ...installedRoles.value])

/** 用原生端返回的已安装角色替换运行时注册表。 */
export function replaceInstalledPetRoles(roles: InstalledPetRole[]) {
  const builtInIds = new Set(PET_ROLES.map(role => role.id))
  installedRoles.value = roles
    .filter(role => !builtInIds.has(role.id))
    .map(toPetRole)
}

/** 判断角色是否属于内置资源。 */
export function isBuiltInPetRole(roleId: string): boolean {
  return PET_ROLES.some(role => role.id === roleId)
}

export function isPetRoleId(value: string): value is PetRoleId {
  return petRoles.value.some(role => role.id === value)
}

export function getPetRole(id: string | undefined): PetRole {
  return petRoles.value.find(role => role.id === id) ?? PET_ROLES[0]!
}

function toPetRole(role: InstalledPetRole): PetRole {
  const sheet = role.spritesheet
  return {
    id: role.id,
    displayName: role.displayName,
    description: role.description,
    kind: role.kind || undefined,
    spritesheetUrl: role.spritesheetUrl,
    spritesheet: {
      id: role.id,
      displayName: role.displayName,
      imageWidth: sheet.width,
      imageHeight: sheet.height,
      frameWidth: sheet.frameWidth,
      frameHeight: sheet.frameHeight,
      rowGap: sheet.rowGap,
      animations: sheet.animations.map(animation => ({
        name: animation.name,
        row: animation.row,
        frameCount: animation.frames,
        fps: animation.fps,
      })),
      crop: sheet.crop
        ? {
            left: sheet.crop.x,
            right: sheet.width - sheet.crop.x - sheet.crop.width,
            top: sheet.crop.y,
            bottom: sheet.height - sheet.crop.y - sheet.crop.height,
          }
        : undefined,
    },
  }
}
