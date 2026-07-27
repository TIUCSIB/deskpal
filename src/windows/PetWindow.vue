<script setup lang="ts">
/**
 * PetWindow.vue - 桌宠主窗口
 * 负责精灵交互、设置同步和独立浮窗联动。
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import Pet from '@/components/Pet.vue'
import { getPetRole } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import { usePetBehavior } from '@/composables/usePetBehavior'
import { usePetInteraction } from '@/composables/usePetInteraction'
import { usePetState } from '@/composables/usePetState'
import { useSystemInfo } from '@/composables/useSystemInfo'
import { broadcastPetContext, sendPetContext } from '@/composables/useWindowBridge'
import { DEFAULT_PET_SCALE } from '@/types/settings'
import type { PetContext, PetContextRequest } from '@/types/window'
import { WINDOW_EVENTS } from '@/types/window'

const { info } = useSystemInfo()
const { mood, updateMood } = usePetState()
const { settings, ready, loadSettings } = useAppSettings()
const { handlePetPress, shouldActivate, isDragging, dragDirection } = usePetInteraction()
const {
  animationName,
  animationRevision,
  hovering,
  setMood,
  setHovering,
  setDragging,
  activate,
  setRole,
  start,
  dispose,
} = usePetBehavior()
const petRef = ref<InstanceType<typeof Pet> | null>(null)
const sizeLocked = computed(() => settings.value.size_locked)
let unlistenScale: UnlistenFn | null = null
let unlistenContextRequest: UnlistenFn | null = null
let listenersDisposed = false
let persistScaleTimer: ReturnType<typeof setTimeout> | null = null
let pendingScale: number | null = null

/** 获取当前宠物状态供浮窗渲染 */
function currentPetContext(): PetContext {
  return {
    info: info.value,
    mood: mood.value,
    roleId: settings.value.pet_role,
    scale: petRef.value?.sizeScale ?? 1,
  }
}

/** 广播当前宠物状态供浮窗渲染 */
function broadcastCurrentContext() {
  return broadcastPetContext(currentPetContext())
}

watch(
  info,
  (value) => {
    updateMood(value)
    setMood(mood.value)
    void broadcastCurrentContext()
  },
  { immediate: true },
)

watch(
  isDragging,
  (dragging) => {
    setDragging(dragging ? dragDirection.value : null)
    if (dragging) {
      void invoke('set_info_window_visible', { visible: false }).catch((error: unknown) => {
        console.error('拖拽时隐藏系统信息窗口失败:', error)
      })
      return
    }
    if (hovering.value) void handlePetHover(true)
  },
  { flush: 'sync' },
)

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
  () => settings.value.pet_role,
  async (roleId) => {
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
    petRef.value?.sizeScale ?? 1,
    settings.value.pet_role,
    ready.value,
  ] as const,
  async ([width, height, scale, _role, isReady]) => {
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
  if (!shouldActivate(event)) return
  activate()
  try {
    await invoke('toggle_chat_window')
  } catch (error) {
    console.error('切换聊天窗口失败:', error)
  }
}

function persistPetScale(scale: number) {
  pendingScale = scale
  if (persistScaleTimer) clearTimeout(persistScaleTimer)
  persistScaleTimer = setTimeout(() => {
    const value = pendingScale
    pendingScale = null
    persistScaleTimer = null
    if (value === null) return
    void invoke('save_pet_scale', { scale: value }).catch((error: unknown) => {
      console.error('保存桌宠缩放失败:', error)
    })
  }, 120)
}

async function handlePetHover(hovering: boolean) {
  if (isDragging.value && hovering) return
  setHovering(hovering)
  try {
    await invoke('set_info_window_visible', { visible: hovering && !isDragging.value })
    if (hovering) await broadcastCurrentContext()
  } catch (error) {
    console.error('切换系统信息窗口失败:', error)
  }
}

function handleScaleChange(scale: number) {
  persistPetScale(scale)
}

function handleRestoreDefaultSize() {
  if (!petRef.value || sizeLocked.value) return
  activate()
  petRef.value.setSizeScale(DEFAULT_PET_SCALE)
  persistPetScale(DEFAULT_PET_SCALE)
}

onMounted(async () => {
  listenersDisposed = false
  const [nextUnlistenContextRequest, nextUnlistenScale] = await Promise.all([
    listen<PetContextRequest>(WINDOW_EVENTS.petContextRequest, (event) => {
      void sendPetContext(event.payload.recipient, currentPetContext()).catch((error: unknown) => {
        console.error('回复浮窗状态请求失败:', error)
      })
    }),
    listen<number>(WINDOW_EVENTS.setScale, (event) => {
      petRef.value?.setSizeScale(event.payload)
    }),
  ])
  if (listenersDisposed) {
    nextUnlistenContextRequest()
    nextUnlistenScale()
    return
  }
  unlistenContextRequest = nextUnlistenContextRequest
  unlistenScale = nextUnlistenScale

  const loaded = await loadSettings()
  await nextTick()
  petRef.value?.setSizeScale(loaded.pet_scale)
  setMood(mood.value)
  start()
})

onUnmounted(() => {
  listenersDisposed = true
  unlistenContextRequest?.()
  unlistenScale?.()
  if (persistScaleTimer) clearTimeout(persistScaleTimer)
  dispose()
})
</script>

<template>
  <main class="pet-window">
    <Pet
      ref="petRef"
      :animation-name="animationName"
      :animation-revision="animationRevision"
      :role-id="settings.pet_role"
      :size-locked="sizeLocked"
      @press="handlePetPress"
      @activate="handlePetActivate"
      @hover-change="handlePetHover"
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
