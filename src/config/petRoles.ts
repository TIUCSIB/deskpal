import type { PetRole, PetRoleId, PetSpritesheet } from '@/types/pet'
import gugaSpritesheetUrl from '@/assets/pets/guga/spritesheet.webp'
import monthlySalaryCatSpritesheetUrl from '@/assets/pets/monthly-salary-cat/spritesheet.webp'
import broomWitchSpritesheetUrl from '@/assets/pets/broom-witch/spritesheet.webp'

export const DEFAULT_PET_ROLE: PetRoleId = 'guga'

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

export function isPetRoleId(value: string): value is PetRoleId {
  return PET_ROLES.some((role) => role.id === value)
}

export function getPetRole(id: string): PetRole {
  return PET_ROLES.find((role) => role.id === id) ?? PET_ROLES[0]
}
