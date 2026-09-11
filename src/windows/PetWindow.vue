<script setup lang="ts">
/**
 * PetWindow.vue - 桌宠主窗口
 * 负责精灵交互、设置同步和独立浮窗联动。
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import Pet from '@/components/Pet.vue'
import { getPetInteractionReply } from '@/config/petPersonalities'
import { getPetRole } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import { usePetBehavior } from '@/composables/usePetBehavior'
import { usePetInteraction } from '@/composables/usePetInteraction'
import { usePetInteractionState } from '@/composables/usePetInteractionState'
import { usePetState } from '@/composables/usePetState'
import { useSystemFeedback } from '@/composables/useSystemFeedback'
import { useSystemInfo } from '@/composables/useSystemInfo'
import { broadcastPetContext, sendPetContext } from '@/composables/useWindowBridge'
import { DEFAULT_PET_SCALE } from '@/types/settings'
import type { PetContext, PetContextRequest } from '@/types/window'
import { WINDOW_EVENTS } from '@/types/window'
const { info } = useSystemInfo()
const { evaluate: evaluateSystemFeedback } = useSystemFeedback()
const { mood, updateMood } = usePetState()
const { settings, ready, loadSettings } = useAppSettings()
const leftClickPassthrough = computed(() => settings.value.main_window_left_click_passthrough)
const {
  handlePetPress,
  shouldActivate,
  tryTriggerClickFeedback,
  isDragging,
  isDragAnimating,
  dragDirection,
} = usePetInteraction(Date.now, {
  leftClickPassthrough: () => leftClickPassthrough.value,
})
const { interactionText, interactionLevel, record: recordInteraction, dispose: disposeInteraction } = usePetInteractionState()
const {
  animationName,
  animationRevision,
  hovering,
  setMood,
  setHovering,
  setDragging,
  triggerClickFeedback,
  petting,
  setRole,
  start,
  dispose,
} = usePetBehavior()
const petRef = ref<InstanceType<typeof Pet> | null>(null)
const sizeLocked = computed(() => settings.value.size_locked)
const activeRoleId = computed(() => ready.value ? getPetRole(settings.value.pet_role).id : undefined)
let unlistenScale: UnlistenFn | null = null
let unlistenContextRequest: UnlistenFn | null = null
let unlistenOverlaySuppressed: UnlistenFn | null = null
let listenersDisposed = false

/**
 * 最近一次已提交给原生层的信息窗可见性请求。
 *
 * 用于去重：悬停状态每变化一次都会走一遍 `invoke` + 原生侧重定位 + 置顶，
 * 而快速划过宠物边缘时状态可能在极短时间内来回翻转，产生一串互相抵消的
 * 跨进程调用与窗口显隐，表现为"闪一下又没了"。仅当目标状态与上次提交不同
 * 时才真正发起请求。
 */
let lastRequestedInfoVisible: boolean | null = null

function currentPetContext(): PetContext {
  return {
    info: info.value,
    mood: mood.value,
    roleId: settings.value.pet_role,
    scale: petRef.value?.sizeScale ?? settings.value.pet_scale,
    interactionText: interactionText.value,
    interactionLevel: interactionLevel.value,
  }
}

function broadcastCurrentContext() {
  // 拖拽期间信息窗被强制隐藏，且拖拽本身会高频移动窗口导致重渲染，
  // 此时每秒广播上下文只会造成无谓的 IPC 与目标窗口重渲染。
  // 拖拽结束时 `watch(isDragging)` 会重新请求显示并触发一次补发。
  if (isDragging.value) return Promise.resolve()
  return broadcastPetContext(currentPetContext())
}

watch(
  info,
  (value) => {
    if (value) {
      const feedback = evaluateSystemFeedback(value, settings.value.quiet_hours)
      if (feedback) {
        void invoke('show_system_feedback', { payload: feedback }).catch((error: unknown) => {
          console.error('显示系统反馈失败:', error)
        })
      }
    }
    updateMood(value)
    setMood(mood.value)
    if (ready.value) void broadcastCurrentContext()
  },
  { immediate: true },
)

watch(
  petting,
  (isPetting) => {
    if (!isPetting) return
    recordInteraction('pet', getPetInteractionReply(settings.value.pet_role, 'pet'))
  },
)

watch(interactionText, () => {
  if (ready.value) void broadcastCurrentContext()
})

watch(
  isDragAnimating,
  (animating) => {
    setDragging(animating ? dragDirection.value : null)
  },
  { flush: 'sync' },
)

// 拖拽动画保持状态与信息窗显隐解耦：
// 动画需延迟复位以播完落地动作，而信息窗应在指针抬起后立即恢复判定。
watch(
  isDragging,
  (dragging) => {
    if (dragging) {
      lastRequestedInfoVisible = false
      void invoke('set_info_window_visible', { visible: false }).catch((error: unknown) => {
        lastRequestedInfoVisible = null
        console.error('拖拽时隐藏系统信息窗口失败:', error)
      })
      return
    }
    // 拖拽结束时若指针仍在宠物上，重新请求显示信息窗。
    if (hovering.value) void handlePetHover(true)
  },
  { flush: 'sync' },
)

// 设置就绪后补发一次上下文：覆盖 loadSettings 未完成期间被跳过的广播
watch(ready, (isReady) => {
  if (!isReady) return
  void broadcastCurrentContext()
})

watch(
  dragDirection,
  (direction) => {
    if (isDragging.value) setDragging(direction)
  },
  { flush: 'sync' },
)

watch(
  () => settings.value.pet_scale,
  (scale) => {
    if (!petRef.value) return
    if (Math.abs(petRef.value.sizeScale - scale) < 0.001) return
    petRef.value.setSizeScale(scale)
  },
)

watch(
  () => activeRoleId.value,
  async (roleId) => {
    if (!roleId) return
    const role = getPetRole(roleId)
    setRole(role.id, role.spritesheet.animations.map((animation) => animation.name))
    await nextTick()
    await broadcastCurrentContext()
  },
  { immediate: true },
)

watch(
  () => [
    petRef.value?.frameWidth ?? 0,
    petRef.value?.frameHeight ?? 0,
    petRef.value?.sizeScale ?? settings.value.pet_scale,
    ready.value,
  ] as const,
  async ([width, height, scale, isReady]) => {
    if (!isReady || !width || !height) return
    try {
      await invoke('resize_main_window', {
        width: Math.ceil(width),
        height: Math.ceil(height),
      })
      await invoke('resize_info_window', { scale })
      await broadcastCurrentContext()
    } catch (error: unknown) {
      console.error('同步桌宠与浮窗尺寸失败:', error)
    }
  },
  { immediate: true },
)

async function handlePetActivate(event: MouseEvent) {
  if (leftClickPassthrough.value && !event.altKey) {
    try {
      await invoke('forward_main_left_click')
    } catch (error) {
      console.error('透传左键点击失败:', error)
    }
    return
  }
  if (!shouldActivate(event)) return
  if (tryTriggerClickFeedback()) {
    triggerClickFeedback()
    recordInteraction('click', getPetInteractionReply(settings.value.pet_role, 'click'))
  }
  try {
    await invoke('show_chat_window')
  } catch (error) {
    console.error('切换聊天窗口失败:', error)
  }
}

function persistPetScale(scale: number) {
  void invoke('save_pet_scale', { scale }).catch((error: unknown) => {
    console.error('保存桌宠缩放失败:', error)
  })
}

async function handlePetHover(hovering: boolean) {
  if (isDragging.value && hovering) return
  setHovering(hovering)
  // 动画保持期间（拖拽刚结束、落地动作仍在播放）不弹出信息窗，
  // 避免窗口刚从拖拽位移中稳定下来就被浮窗遮挡。
  const visible = hovering && !isDragAnimating.value
  if (visible === lastRequestedInfoVisible) return
  lastRequestedInfoVisible = visible
  try {
    await invoke('set_info_window_visible', { visible })
    if (!visible) return
    // 上下文广播不依赖设置是否就绪：设置未就绪时使用当前已知值发送一次，
    // 避免信息窗显示后停留在初始空上下文。设置加载完成后还有后续广播兜底。
    await broadcastCurrentContext()
  } catch (error) {
    // 提交失败时回退记录，使下一次相同请求能够重试
    lastRequestedInfoVisible = null
    console.error('切换系统信息窗口失败:', error)
  }
}

async function handleContextMenu(event: MouseEvent) {
  if (isDragging.value) return
  try {
    await invoke('show_main_context_menu', { x: event.clientX, y: event.clientY })
  } catch (error: unknown) {
    console.error('打开右键快捷菜单失败:', error)
  }
}

function handleScaleChange(scale: number) {
  persistPetScale(scale)
}

function handleRestoreDefaultSize() {
  if (!petRef.value || sizeLocked.value) return
  triggerClickFeedback()
  petRef.value.setSizeScale(DEFAULT_PET_SCALE)
  persistPetScale(DEFAULT_PET_SCALE)
}

onMounted(async () => {
  listenersDisposed = false
  const [nextUnlistenContextRequest, nextUnlistenScale, nextUnlistenSuppressed] =
    await Promise.all([
      listen<PetContextRequest>(WINDOW_EVENTS.petContextRequest, (event) => {
        if (!ready.value) return
        void sendPetContext(event.payload.recipient, currentPetContext()).catch((error: unknown) => {
          console.error('回复浮窗状态请求失败:', error)
        })
      }),
      listen<number>(WINDOW_EVENTS.setScale, (event) => {
        petRef.value?.setSizeScale(event.payload)
      }),
      listen<string>(WINDOW_EVENTS.overlaySuppressed, () => {
        // 原生层因互斥仲裁隐藏了浮窗，重置 hover 去重状态与请求记录，
        // 使指针保持悬停时也能重新触发显示。
        // `reevaluateHover` 内部会复用最近一次指针位置，因此无需新的
        // pointermove 事件 —— 抢占往往发生在指针静止时（如刚右键唤出菜单）。
        lastRequestedInfoVisible = null
        petRef.value?.reevaluateHover()
      }),
    ])
  if (listenersDisposed) {
    nextUnlistenContextRequest()
    nextUnlistenScale()
    nextUnlistenSuppressed()
    return
  }
  unlistenContextRequest = nextUnlistenContextRequest
  unlistenScale = nextUnlistenScale
  unlistenOverlaySuppressed = nextUnlistenSuppressed

  const loaded = await loadSettings()
  await nextTick()
  petRef.value?.setSizeScale(loaded.pet_scale)
  await nextTick()
  setMood(mood.value)
  start()
  try {
    await invoke('show_startup_main_window')
    await invoke('refresh_main_window_presentation')
  } catch (error: unknown) {
    console.error('刷新桌宠窗口呈现状态失败:', error)
  }
})

onUnmounted(() => {
  listenersDisposed = true
  unlistenContextRequest?.()
  unlistenScale?.()
  unlistenOverlaySuppressed?.()
  disposeInteraction()
  dispose()
})
</script>

<template>
  <main class="pet-window">
    <Pet
      v-if="ready"
      ref="petRef"
      :animation-name="animationName"
      :animation-revision="animationRevision"
      :role-id="activeRoleId"
      :scale="settings.pet_scale"
      :size-locked="sizeLocked"
      :left-click-passthrough="leftClickPassthrough"
      @press="handlePetPress"
      @activate="handlePetActivate"
      @hover-change="handlePetHover"
      @context-menu="handleContextMenu"
      @scale-change="handleScaleChange"
      @restore-default-size="handleRestoreDefaultSize"
    />
  </main>
</template>

<style scoped>
.pet-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  background: transparent;
}
</style>
