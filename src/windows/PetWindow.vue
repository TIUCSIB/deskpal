<script setup lang="ts">
/**
 * PetWindow.vue - 桌宠主窗口
 * 负责精灵交互、设置同步和独立浮窗联动。
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import Pet from '@/components/Pet.vue'
import { useAppSettings } from '@/composables/useAppSettings'
import { usePetInteraction } from '@/composables/usePetInteraction'
import { usePetState } from '@/composables/usePetState'
import { useSystemInfo } from '@/composables/useSystemInfo'
import { broadcastPetContext } from '@/composables/useWindowBridge'
import type { PetMood } from '@/types/system'
import { DEFAULT_PET_SCALE } from '@/types/settings'
import { WINDOW_EVENTS } from '@/types/window'

const { info } = useSystemInfo()
const { mood, updateMood } = usePetState()
const { settings, ready, loadSettings } = useAppSettings()
const { handlePetPress, shouldActivate, isDragging, dragDirection } = usePetInteraction()
const petRef = ref<InstanceType<typeof Pet> | null>(null)
const sizeLocked = computed(() => settings.value.size_locked)
const INTERACTION_RETURN_DELAY = 900
const IDLE_ANIMATION_DELAY = 15000
const AMBIENT_ANIMATION_DELAY = 60000
const MOOD_ANIMATIONS: Record<PetMood, string> = {
  happy: 'Review',
  normal: 'Idle',
  sleepy: 'Waiting',
  warning: 'Failed',
}
const AMBIENT_ANIMATIONS: Record<PetMood, string[]> = {
  happy: ['Idle', 'Review', 'Waiting'],
  normal: ['Idle', 'Review', 'Waiting'],
  sleepy: ['Waiting', 'Idle', 'Review'],
  warning: ['Failed'],
}
let unlistenScale: UnlistenFn | null = null
let persistScaleTimer: ReturnType<typeof setTimeout> | null = null
let idleTimer: ReturnType<typeof setTimeout> | null = null
let ambientTimer: ReturnType<typeof setTimeout> | null = null
let interactionReturnTimer: ReturnType<typeof setTimeout> | null = null
let ambientCursor = 0
let pendingScale: number | null = null

/** 广播当前宠物状态供浮窗渲染 */
function broadcastCurrentContext() {
  return broadcastPetContext({
    info: info.value,
    mood: mood.value,
    scale: petRef.value?.sizeScale ?? 1,
  })
}

function clearIdleTimer() {
  if (!idleTimer) return
  clearTimeout(idleTimer)
  idleTimer = null
}

function clearAmbientTimer() {
  if (!ambientTimer) return
  clearTimeout(ambientTimer)
  ambientTimer = null
}

function clearInteractionReturnTimer() {
  if (!interactionReturnTimer) return
  clearTimeout(interactionReturnTimer)
  interactionReturnTimer = null
}

function scheduleInteractionReturn() {
  clearInteractionReturnTimer()
  interactionReturnTimer = setTimeout(() => {
    petRef.value?.playNamedAnimation('Idle')
    scheduleIdleAnimation()
  }, INTERACTION_RETURN_DELAY)
}

function scheduleIdleAnimation() {
  clearIdleTimer()
  clearAmbientTimer()
  if (mood.value === 'warning' || isDragging.value) return
  idleTimer = setTimeout(() => {
    petRef.value?.playNamedAnimation('Waiting')
    scheduleAmbientAnimation()
  }, IDLE_ANIMATION_DELAY)
}

function scheduleAmbientAnimation() {
  clearAmbientTimer()
  if (mood.value === 'warning' || isDragging.value) return
  ambientTimer = setTimeout(() => {
    const pool = AMBIENT_ANIMATIONS[mood.value] ?? AMBIENT_ANIMATIONS.normal
    const name = pool[ambientCursor % pool.length] ?? pool[0]
    ambientCursor = (ambientCursor + 1) % pool.length
    if (name) petRef.value?.playNamedAnimation(name)
    scheduleAmbientAnimation()
  }, AMBIENT_ANIMATION_DELAY)
}

function playMoodAnimation() {
  petRef.value?.playNamedAnimation(MOOD_ANIMATIONS[mood.value] ?? 'Idle')
}

function playHoverAnimation() {
  clearIdleTimer()
  clearAmbientTimer()
  clearInteractionReturnTimer()
  petRef.value?.playNamedAnimation('Waving')
}

function playActivateAnimation() {
  clearIdleTimer()
  clearAmbientTimer()
  clearInteractionReturnTimer()
  petRef.value?.playNamedAnimation('Jumping')
}

function playDragAnimation(direction: 'left' | 'right' | null) {
  clearIdleTimer()
  clearAmbientTimer()
  clearInteractionReturnTimer()
  if (direction === 'left') {
    petRef.value?.playNamedAnimation('RunLeft')
  } else if (direction === 'right') {
    petRef.value?.playNamedAnimation('RunRight')
  } else {
    petRef.value?.playNamedAnimation('Running')
  }
}

watch(
  info,
  (value) => {
    const before = mood.value
    updateMood(value)
    if (mood.value !== before) {
      playMoodAnimation()
      scheduleAmbientAnimation()
    }
    void broadcastCurrentContext()
  },
  { immediate: true },
)

watch(
  dragDirection,
  (direction) => {
    if (!isDragging.value || direction === null) return
    playDragAnimation(direction)
  },
  { flush: 'sync' },
)

watch(
  isDragging,
  (dragging, previous) => {
    if (dragging === previous) return
    if (dragging) {
      playDragAnimation(dragDirection.value)
      void invoke('set_info_window_visible', { visible: false }).catch((error: unknown) => {
        console.error('拖拽时隐藏系统信息窗口失败:', error)
      })
      return
    }
    petRef.value?.playNamedAnimation('Idle')
    scheduleIdleAnimation()
    scheduleAmbientAnimation()
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
  async () => {
    await nextTick()
    petRef.value?.playNamedAnimation('Idle')
    await broadcastCurrentContext()
  },
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
  playActivateAnimation()
  try {
    await invoke('toggle_chat_window')
  } catch (error) {
    console.error('切换聊天窗口失败:', error)
  } finally {
    scheduleInteractionReturn()
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
  if (hovering) {
    clearAmbientTimer()
    playHoverAnimation()
  } else if (!isDragging.value) {
    scheduleInteractionReturn()
  }
  try {
    await invoke('set_info_window_visible', { visible: hovering })
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
  playActivateAnimation()
  petRef.value.setSizeScale(DEFAULT_PET_SCALE)
  persistPetScale(DEFAULT_PET_SCALE)
}

onMounted(async () => {
  const loaded = await loadSettings()
  await nextTick()
  petRef.value?.setSizeScale(loaded.pet_scale)
  playMoodAnimation()
  scheduleIdleAnimation()
  scheduleAmbientAnimation()
  unlistenScale = await listen<number>(WINDOW_EVENTS.setScale, (event) => {
    petRef.value?.setSizeScale(event.payload)
  })
})

onUnmounted(() => {
  unlistenScale?.()
  if (persistScaleTimer) clearTimeout(persistScaleTimer)
  clearIdleTimer()
  clearAmbientTimer()
  clearInteractionReturnTimer()
})
</script>

<template>
  <main class="pet-window">
    <Pet
      ref="petRef"
      :mood="mood"
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
