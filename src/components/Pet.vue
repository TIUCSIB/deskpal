<script setup lang="ts">
/**
 * Pet.vue - 桌宠角色（精灵表动画）
 * 使用像素命中确保只有角色非透明区域响应交互。
 */
import { computed, onUnmounted, watch } from 'vue'
import { DEFAULT_PET_ROLE, getPetRole } from '@/config/petRoles'
import type { PetRoleId } from '@/types/pet'
import { useSpriteAnimation } from '@/composables/useSpriteAnimation'
import { usePixelHitTest } from '@/composables/usePixelHitTest'

const props = withDefaults(
  defineProps<{
    animationName: string
    animationRevision?: number
    roleId?: PetRoleId
    scale?: number
    sizeLocked?: boolean
    leftClickPassthrough?: boolean
  }>(),
  {
    roleId: DEFAULT_PET_ROLE,
    scale: 1,
    sizeLocked: false,
    leftClickPassthrough: false,
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

const { hitTest, setFrameKey } = usePixelHitTest(
  spritesheetUrl,
  backgroundPosition,
  backgroundSize,
)

/**
 * 帧位置变化时失效逐像素缓存。
 *
 * `backgroundPosition` 唯一标识了当前帧在精灵表上的位置（由帧索引、
 * 动画行、缩放共同决定），因此可直接作为缓存键。同一容器坐标在不同帧
 * 对应不同源图像素，不失效会得到错误的命中结果。
 */
watch(backgroundPosition, (position) => setFrameKey(position), { immediate: true })

let hoveringPetPixel = false

type PointerLikeEvent = MouseEvent | PointerEvent | WheelEvent

watch(
  () => props.animationName,
  (animationName) => {
    playNamedAnimation(animationName)
  },
  { immediate: true },
)

watch(
  () => props.scale,
  (scale) => {
    setSizeScale(scale)
  },
  { immediate: true },
)

/** 获取鼠标相对于精灵容器的坐标 */
function getRelativePosition(event: PointerLikeEvent): { x: number; y: number } {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  return { x: event.clientX - rect.left, y: event.clientY - rect.top }
}

/** 判断事件是否命中角色非透明像素 */
function isPetPixel(event: PointerLikeEvent): boolean {
  const { x, y } = getRelativePosition(event)
  return hitTest(x, y)
}

/** 同步当前 hover 状态，避免只依赖鼠标移动事件 */
function syncHoverState(event: PointerLikeEvent) {
  const hovering = isPetPixel(event)
  if (hovering === hoveringPetPixel) return
  hoveringPetPixel = hovering
  emit('hoverChange', hovering)
}

function handleMouseDown(event: MouseEvent) {
  syncHoverState(event)
  if (event.button === 0 && isPetPixel(event)) emit('press', event)
}

function handleClick(event: MouseEvent) {
  if (event.button !== 0 || !isPetPixel(event)) return
  emit('activate', event)
}

function handleDoubleClick(event: MouseEvent) {
  if (props.sizeLocked || event.button !== 0 || !isPetPixel(event)) return
  if (props.leftClickPassthrough && !event.altKey) return
  emit('restoreDefaultSize')
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault()
  if (isPetPixel(event)) emit('contextMenu', event)
}

function handlePointerEnter(event: PointerEvent) {
  pendingHoverEvent = event
  syncHoverState(event)
}

let pendingHoverEvent: PointerEvent | null = null
let hoverRafId = 0

/**
 * 按动画帧节流的悬停评测。
 *
 * `pointermove` 在高刷新率设备上可达 120 Hz，而每次评测都要做像素级
 * 命中检测（同步 GPU→CPU 回读）。同一帧内指针的多次移动只需最后一次
 * 结果，故合并到 `requestAnimationFrame` 中执行一次。
 */
function handlePointerMove(event: PointerEvent) {
  pendingHoverEvent = event
  if (hoverRafId) return
  hoverRafId = requestAnimationFrame(() => {
    hoverRafId = 0
    const pending = pendingHoverEvent
    pendingHoverEvent = null
    if (pending) syncHoverState(pending)
  })
}

function handleMouseLeave() {
  if (hoverRafId) {
    cancelAnimationFrame(hoverRafId)
    hoverRafId = 0
    pendingHoverEvent = null
  }
  if (!hoveringPetPixel) return
  hoveringPetPixel = false
  emit('hoverChange', false)
}

onUnmounted(() => {
  if (hoverRafId) cancelAnimationFrame(hoverRafId)
  hoverRafId = 0
  pendingHoverEvent = null
})

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

/**
 * 依据当前指针位置重新评测 hover。
 *
 * 用途：原生层可能因浮窗互斥仲裁主动隐藏了信息窗，但本组件的去重状态
 * `hoveringPetPixel` 仍为 true，导致后续 pointermove 不再上报。该函数在
 * 拖拽结束、窗口重新激活等时机被调用，强制收敛状态。
 *
 * @param event 可选的指针事件；缺省时按"未命中"处理，避免残留 hover
 */
function reevaluateHover(event?: PointerLikeEvent) {
  const hovering = event ? isPetPixel(event) : false
  if (hovering === hoveringPetPixel) {
    // 状态未变，但原生层可能已隐藏浮窗，需强制重发一次以触发重新显示
    if (hovering) emit('hoverChange', true)
    return
  }
  hoveringPetPixel = hovering
  emit('hoverChange', hovering)
}

/** 暴露给主窗口的缩放控制、尺寸信息与动画控制 */
defineExpose({
  sizeScale,
  setSizeScale,
  frameWidth,
  frameHeight,
  playNamedAnimation,
  reevaluateHover,
})
</script>

<template>
  <div
    class="pet"
    :style="{ width: frameWidth + 'px', height: frameHeight + 'px' }"
    @wheel="handleWheel"
    @pointerenter="handlePointerEnter"
    @pointermove="handlePointerMove"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu="handleContextMenu"
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
