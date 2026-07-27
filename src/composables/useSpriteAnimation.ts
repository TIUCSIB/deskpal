/**
 * useSpriteAnimation.ts - 精灵表动画 composable
 * 驱动帧循环、心情动画池和交互动画。
 */
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { PetRole } from '@/types/pet'
import type { PetAnimation, PetSpritesheet } from '@/types/pet'

export const MIN_PET_SCALE = 0.45
export const MAX_PET_SCALE = 1.2

function getFallbackAnimation(sheet: PetSpritesheet) {
  return sheet.animations[0]!
}

/** 根据动画名和帧索引计算 backgroundPosition */
function getBackgroundPosition(
  sheet: PetSpritesheet,
  animation: PetAnimation,
  frameIndex: number,
  scale: number,
): string {
  const cropLeft = sheet.crop?.left ?? 0
  const cropTop = sheet.crop?.top ?? 0
  const x = -(frameIndex * sheet.frameWidth * scale) - (cropLeft * scale)
  const rowStride = sheet.frameHeight + sheet.rowGap
  const y = -(animation.row * rowStride * scale) - (cropTop * scale)
  return `${x}px ${y}px`
}

export function useSpriteAnimation(role: { readonly value: PetRole }) {
  const currentAnimation = ref<PetAnimation>(getFallbackAnimation(role.value.spritesheet))
  const frameIndex = ref(0)
  const backgroundPosition = ref('0px 0px')
  const sizeScale = ref(1)

  let animFrameId = 0
  let lastFrameTime = 0

  function findAnimation(name: string): PetAnimation | undefined {
    return role.value.spritesheet.animations.find((animation) => animation.name === name)
  }

  /** 切换动画，从第 0 帧开始播放 */
  function switchAnimation(animation: PetAnimation) {
    currentAnimation.value = animation
    frameIndex.value = 0
    lastFrameTime = 0
    backgroundPosition.value = getBackgroundPosition(
      role.value.spritesheet,
      animation,
      0,
      sizeScale.value,
    )
  }

  /** 按名称播放一次指定动画 */
  function playNamedAnimation(name: string) {
    const animation = findAnimation(name)
    if (animation) switchAnimation(animation)
  }

  /** 设置缩放比例 */
  function setSizeScale(scale: number) {
    const safeScale = Number.isFinite(scale) ? scale : 1
    sizeScale.value = Math.max(MIN_PET_SCALE, Math.min(MAX_PET_SCALE, safeScale))
  }

  /** 帧循环 */
  function tick(timestamp: number) {
    const interval = 1000 / currentAnimation.value.fps
    if (timestamp - lastFrameTime >= interval) {
      lastFrameTime = timestamp
      frameIndex.value = (frameIndex.value + 1) % currentAnimation.value.frameCount
      backgroundPosition.value = getBackgroundPosition(
        role.value.spritesheet,
        currentAnimation.value,
        frameIndex.value,
        sizeScale.value,
      )
    }
    animFrameId = requestAnimationFrame(tick)
  }

  watch(sizeScale, () => {
    backgroundPosition.value = getBackgroundPosition(
      role.value.spritesheet,
      currentAnimation.value,
      frameIndex.value,
      sizeScale.value,
    )
  })

  watch(role, () => {
    switchAnimation(findAnimation('Idle') ?? getFallbackAnimation(role.value.spritesheet))
  })

  onMounted(() => {
    lastFrameTime = 0
    animFrameId = requestAnimationFrame(tick)
  })

  onUnmounted(() => {
    cancelAnimationFrame(animFrameId)
  })

  const backgroundSize = computed(() => {
    const sheet = role.value.spritesheet
    return `${sheet.imageWidth * sizeScale.value}px ${sheet.imageHeight * sizeScale.value}px`
  })

  const scaledFrameWidth = computed(() => {
    const sheet = role.value.spritesheet
    const crop = sheet.crop
    const width = crop ? sheet.frameWidth - crop.left - crop.right : sheet.frameWidth
    return width * sizeScale.value
  })

  const scaledFrameHeight = computed(() => {
    const sheet = role.value.spritesheet
    const crop = sheet.crop
    const height = crop ? sheet.frameHeight - crop.top - crop.bottom : sheet.frameHeight
    return height * sizeScale.value
  })

  return {
    backgroundPosition,
    backgroundSize,
    frameWidth: scaledFrameWidth,
    frameHeight: scaledFrameHeight,
    sizeScale,
    setSizeScale,
    playNamedAnimation,
  }
}
