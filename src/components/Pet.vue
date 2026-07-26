<script setup lang="ts">
/**
 * Pet.vue - 桌宠角色（精灵表动画）
 * 使用像素命中确保只有角色非透明区域响应交互。
 */
import { watch } from 'vue'
import type { PetMood } from '@/types/system'
import { useSpriteAnimation } from '@/composables/useSpriteAnimation'
import { usePixelHitTest } from '@/composables/usePixelHitTest'
import spritesheetUrl from '@/assets/pet/spritesheet.webp'

const props = withDefaults(
  defineProps<{
    mood: PetMood
    sizeLocked?: boolean
  }>(),
  { sizeLocked: false },
)

const emit = defineEmits<{
  press: [event: MouseEvent]
  activate: [event: MouseEvent]
  hoverChange: [hovering: boolean]
  scaleChange: [scale: number]
  restoreDefaultSize: []
}>()

const {
  backgroundPosition,
  backgroundSize,
  frameWidth,
  frameHeight,
  sizeScale,
  setMoodPool,
  setSizeScale,
  playNamedAnimation,
} = useSpriteAnimation()

const { hitTest } = usePixelHitTest(
  spritesheetUrl,
  backgroundPosition,
  backgroundSize,
)
let hoveringPetPixel = false

watch(
  () => props.mood,
  (mood) => {
    setMoodPool(mood)
  },
  { immediate: true },
)

/** 获取鼠标相对于精灵容器的坐标 */
function getRelativePosition(event: MouseEvent): { x: number; y: number } {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  return { x: event.clientX - rect.left, y: event.clientY - rect.top }
}

/** 判断事件是否命中角色非透明像素 */
function isPetPixel(event: MouseEvent): boolean {
  const { x, y } = getRelativePosition(event)
  return hitTest(x, y)
}

function handleMouseDown(event: MouseEvent) {
  if (event.button === 0 && isPetPixel(event)) emit('press', event)
}

function handleClick(event: MouseEvent) {
  if (event.button !== 0 || !isPetPixel(event)) return
  emit('activate', event)
}

function handleDoubleClick(event: MouseEvent) {
  if (props.sizeLocked || event.button !== 0 || !isPetPixel(event)) return
  emit('restoreDefaultSize')
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault()
}

function handleMouseMove(event: MouseEvent) {
  const hovering = isPetPixel(event)
  if (hovering === hoveringPetPixel) return
  hoveringPetPixel = hovering
  emit('hoverChange', hovering)
}

function handleMouseLeave() {
  if (!hoveringPetPixel) return
  hoveringPetPixel = false
  emit('hoverChange', false)
}

/** 非透明像素上的滚轮缩放 */
function handleWheel(event: WheelEvent) {
  if (props.sizeLocked || !isPetPixel(event)) return
  if (Math.abs(event.deltaY) < 8) return
  event.preventDefault()
  const delta = event.deltaY > 0 ? -0.1 : 0.1
  const nextScale = sizeScale.value + delta
  setSizeScale(nextScale)
  emit('scaleChange', nextScale)
}

/** 暴露给主窗口的缩放控制、尺寸信息与动画控制 */
defineExpose({ sizeScale, setSizeScale, frameWidth, frameHeight, playNamedAnimation })
</script>

<template>
  <div
    class="pet"
    :style="{ width: frameWidth + 'px', height: frameHeight + 'px' }"
    @wheel="handleWheel"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu="handleContextMenu"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
  >
    <div
      class="pet__sprite"
      :style="{
        backgroundImage: `url(${spritesheetUrl})`,
        backgroundSize,
        backgroundPosition,
      }"
    ></div>
  </div>
</template>

<style scoped>
.pet {
  position: relative;
  cursor: pointer;
  overflow: hidden;
  pointer-events: auto;
}

.pet__sprite {
  width: 100%;
  height: 100%;
  background-repeat: no-repeat;
}
</style>
