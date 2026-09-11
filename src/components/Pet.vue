<script setup lang="ts">
/**
 * Pet.vue - 桌宠角色（精灵表动画）
 * 使用像素命中确保只有角色非透明区域响应交互。
 */
import { computed, onUnmounted, ref, watch } from 'vue'
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
/** 最近一次指针事件。用于抢占后无需等待新事件即可重评 hover。 */
let lastPointerEvent: PointerEvent | null = null

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

/** 精灵容器元素引用。`currentTarget` 仅在事件派发期间有效，
 *  抢占后的延迟重评需要独立持有容器引用才能换算相对坐标。 */
const containerRef = ref<HTMLElement | null>(null)

/** 获取鼠标相对于精灵容器的坐标 */
function getRelativePosition(event: PointerLikeEvent): { x: number; y: number } | null {
  const container = containerRef.value ?? (event.currentTarget as HTMLElement | null)
  if (!container) return null
  const rect = container.getBoundingClientRect()
  return { x: event.clientX - rect.left, y: event.clientY - rect.top }
}

/** 判断事件是否命中角色非透明像素 */
function isPetPixel(event: PointerLikeEvent): boolean {
  const position = getRelativePosition(event)
  if (!position) return false
  return hitTest(position.x, position.y)
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
  lastPointerEvent = event
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
 *
 * 首帧例外：若当前尚未处于悬停态，说明这是"移入"的第一步，延迟一帧会让
 * 用户感到明显的反应迟滞。此时同步评测一次，后续移动再走节流路径。
 */
function handlePointerMove(event: PointerEvent) {
  lastPointerEvent = event
  if (!hoveringPetPixel) {
    // 移入路径：立即评测，消除入场延迟
    if (hoverRafId) {
      cancelAnimationFrame(hoverRafId)
      hoverRafId = 0
      pendingHoverEvent = null
    }
    syncHoverState(event)
    return
  }
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
  lastPointerEvent = null
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
  lastPointerEvent = null
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
/**
 * 依据当前指针位置重新评测 hover。
 *
 * 用途：原生层可能因浮窗互斥仲裁主动隐藏了信息窗，但本组件的去重状态
 * `hoveringPetPixel` 仍为 true，导致后续 pointermove 不再上报。该函数在
 * 拖拽结束、浮窗被抢占等时机被调用，强制收敛状态。
 *
 * 关键点：抢占发生时指针通常**静止不动**（例如刚右键唤出菜单），若按
 * "无参即判为未命中"处理，就必须等下一次真实 pointermove 才能恢复显示，
 * 而 pointermove 还受 RAF 节流 —— 用户会感到"移回去也不出来"。
 * 因此优先复用最近一次指针事件重评真实命中结果。
 *
 * @param event 可选的指针事件；缺省时回退到最近一次记录的指针位置
 */
function reevaluateHover(event?: PointerLikeEvent) {
  const target = event ?? lastPointerEvent
  const hovering = target ? isPetPixel(target) : false
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
    ref="containerRef"
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
