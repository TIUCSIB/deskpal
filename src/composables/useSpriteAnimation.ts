/**
 * useSpriteAnimation.ts - 精灵表动画 composable
 * 驱动帧循环、心情动画池和交互动画。
 */
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { PetMood } from '@/types/system'
import type { PetAnimation, PetSpritesheet } from '@/types/pet'

export const MIN_PET_SCALE = 0.45
export const MAX_PET_SCALE = 1.2

/** 咕嘎精灵表配置 */
const GUGA_SPRITESHEET: PetSpritesheet = {
  id: 'guga',
  displayName: '咕嘎',
  imageWidth: 1536,
  imageHeight: 1872,
  frameWidth: 192,
  frameHeight: 209,
  rowGap: 11,
  /** 裁剪帧四周留白，使点击区域紧贴角色 */
  crop: { left: 25, right: 25, top: 5, bottom: 5 },
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

const MOOD_POOLS: Record<PetMood, string[]> = {
  happy: ['Idle', 'Review', 'RunRight', 'RunLeft', 'Waiting'],
  normal: ['Idle', 'Review', 'RunRight', 'RunLeft', 'Waiting', 'Running'],
  sleepy: ['Waiting', 'Idle', 'Review'],
  warning: ['Failed', 'Running', 'Review', 'Idle'],
}

function findAnimation(name: string): PetAnimation | undefined {
  return GUGA_SPRITESHEET.animations.find((animation) => animation.name === name)
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
  const y = -(animation.row * sheet.frameHeight * scale) - (cropTop * scale)
  return `${x}px ${y}px`
}

export function useSpriteAnimation() {
  const currentAnimation = ref<PetAnimation>(GUGA_SPRITESHEET.animations[0])
  const frameIndex = ref(0)
  const backgroundPosition = ref('0px 0px')
  const sizeScale = ref(1)
  const animationPool = ref<string[]>([...MOOD_POOLS.normal])

  let animFrameId = 0
  let lastFrameTime = 0

  /** 切换动画，从第 0 帧开始播放 */
  function switchAnimation(animation: PetAnimation) {
    currentAnimation.value = animation
    frameIndex.value = 0
    lastFrameTime = 0
    backgroundPosition.value = getBackgroundPosition(
      GUGA_SPRITESHEET,
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

  /** 根据心情切换常驻动画池 */
  function setMoodPool(mood: PetMood) {
    animationPool.value = [...MOOD_POOLS[mood]]
    if (!animationPool.value.includes(currentAnimation.value.name)) {
      const next = findAnimation(animationPool.value[0])
      if (next) switchAnimation(next)
    }
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
        GUGA_SPRITESHEET,
        currentAnimation.value,
        frameIndex.value,
        sizeScale.value,
      )
    }

    animFrameId = requestAnimationFrame(tick)
  }

  /** 缩放变化时同步背景位置 */
  watch(sizeScale, () => {
    backgroundPosition.value = getBackgroundPosition(
      GUGA_SPRITESHEET,
      currentAnimation.value,
      frameIndex.value,
      sizeScale.value,
    )
  })

  onMounted(() => {
    lastFrameTime = 0
    animFrameId = requestAnimationFrame(tick)
  })

  onUnmounted(() => {
    cancelAnimationFrame(animFrameId)
  })

  /** 缩放后的背景尺寸 */
  const backgroundSize = computed(() => {
    const w = GUGA_SPRITESHEET.imageWidth * sizeScale.value
    const h = GUGA_SPRITESHEET.imageHeight * sizeScale.value
    return `${w}px ${h}px`
  })

  /** 缩放后的帧尺寸（已裁剪留白） */
  const scaledFrameWidth = computed(() => {
    const crop = GUGA_SPRITESHEET.crop
    const w = crop
      ? GUGA_SPRITESHEET.frameWidth - crop.left - crop.right
      : GUGA_SPRITESHEET.frameWidth
    return w * sizeScale.value
  })

  const scaledFrameHeight = computed(() => {
    const crop = GUGA_SPRITESHEET.crop
    const h = crop
      ? GUGA_SPRITESHEET.frameHeight - crop.top - crop.bottom
      : GUGA_SPRITESHEET.frameHeight
    return h * sizeScale.value
  })

  return {
    backgroundPosition,
    backgroundSize,
    frameWidth: scaledFrameWidth,
    frameHeight: scaledFrameHeight,
    sizeScale,
    setMoodPool,
    setSizeScale,
    playNamedAnimation,
  }
}
