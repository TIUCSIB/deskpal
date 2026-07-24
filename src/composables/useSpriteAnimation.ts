/**
 * useSpriteAnimation.ts - 精灵表动画 composable
 * 驱动帧循环，随机切换动画，支持缩放
 */
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { PetSpritesheet, PetAnimation } from '@/types/pet'

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

/** 随机选取一个与当前不同的动画 */
function pickRandomAnimation(current: PetAnimation): PetAnimation {
  const others = GUGA_SPRITESHEET.animations.filter(
    (a) => a.name !== current.name,
  )
  return others[Math.floor(Math.random() * others.length)]
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

  let animFrameId = 0
  let lastFrameTime = 0

  /** 切换动画，从第 0 帧开始播放 */
  function switchAnimation(anim: PetAnimation) {
    currentAnimation.value = anim
    frameIndex.value = 0
    lastFrameTime = 0
    backgroundPosition.value = getBackgroundPosition(
      GUGA_SPRITESHEET,
      anim,
      0,
      sizeScale.value,
    )
  }

  /** 帧循环 */
  function tick(timestamp: number) {
    const interval = 1000 / currentAnimation.value.fps

    if (timestamp - lastFrameTime >= interval) {
      lastFrameTime = timestamp
      frameIndex.value =
        (frameIndex.value + 1) % currentAnimation.value.frameCount

      // 动画播放完毕，随机切换下一个
      if (frameIndex.value === 0) {
        switchAnimation(pickRandomAnimation(currentAnimation.value))
      } else {
        backgroundPosition.value = getBackgroundPosition(
          GUGA_SPRITESHEET,
          currentAnimation.value,
          frameIndex.value,
          sizeScale.value,
        )
      }
    }

    animFrameId = requestAnimationFrame(tick)
  }

  /** 设置缩放比例 */
  function setSizeScale(scale: number) {
    sizeScale.value = Math.max(0.3, Math.min(3, scale))
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
    /** 当前帧的 background-position 值 */
    backgroundPosition,
    /** 缩放后的精灵表背景尺寸 */
    backgroundSize,
    /** 缩放后的帧容器尺寸 */
    frameWidth: scaledFrameWidth,
    frameHeight: scaledFrameHeight,
    /** 缩放比例 */
    sizeScale,
    /** 设置缩放比例（0.3 ~ 3.0） */
    setSizeScale,
  }
}
