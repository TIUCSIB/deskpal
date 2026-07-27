<script setup lang="ts">
/**
 * Pet.vue - 桌宠角色（精灵表动画）
 * 使用像素命中确保只有角色非透明区域响应交互。
 */
import { computed, watch } from 'vue'
import { DEFAULT_PET_ROLE, getPetRole } from '@/config/petRoles'
import type { PetRoleId } from '@/types/pet'
import { useSpriteAnimation } from '@/composables/useSpriteAnimation'
import { usePixelHitTest } from '@/composables/usePixelHitTest'

const props = withDefaults(
  defineProps<{
    animationName: string
    animationRevision?: number
    roleId?: PetRoleId
    sizeLocked?: boolean
  }>(),
  {
    roleId: DEFAULT_PET_ROLE,
    sizeLocked: false,
  },
)

const emit = defineEmits<{
  press: [event: MouseEvent]
  activate: [event: MouseEvent]
  hoverChange: [hovering: boolean]
  contextMenu: [event: MouseEvent]
  scaleChange: [scale: number]
  restoreDefaultSize: []
}>()

const role = computed(() => getPetRole(props.roleId))
const spritesheetUrl = computed(() => role.value.spritesheetUrl)
const {
  backgroundPosition,
  backgroundSize,
  frameWidth,
  frameHeight,
  sizeScale,
  setSizeScale,
  playNamedAnimation,
} = useSpriteAnimation(role)

const { hitTest } = usePixelHitTest(
  spritesheetUrl,
  backgroundPosition,
  backgroundSize,
)
let hoveringPetPixel = false

watch(
  () => [props.animationName, props.animationRevision] as const,
  ([animationName]) => {
    playNamedAnimation(animationName)
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
  if (isPetPixel(event)) emit('contextMenu', event)
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
