<script setup lang="ts">
/**
 * PetWindow.vue - 桌宠主窗口
 * 只负责精灵、拖拽和打开独立原生界面。
 */
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import Pet from '@/components/Pet.vue'
import { usePetInteraction } from '@/composables/usePetInteraction'
import { usePetState } from '@/composables/usePetState'
import { useSystemInfo } from '@/composables/useSystemInfo'
import { broadcastPetContext } from '@/composables/useWindowBridge'
import { WINDOW_EVENTS } from '@/types/window'

const { info } = useSystemInfo()
const { mood, updateMood } = usePetState()
const { handlePetPress, shouldActivate } = usePetInteraction()
const petRef = ref<InstanceType<typeof Pet> | null>(null)
let unlistenScale: UnlistenFn | null = null

watch(
  info,
  (value) => {
    updateMood(value)
    void broadcastPetContext({ info: value, mood: mood.value })
  },
  { immediate: true },
)

nextTick(() => {
  if (!petRef.value) return
  const pet = petRef.value
  watch(
    () => [pet.frameWidth, pet.frameHeight] as const,
    ([width, height]) => {
      void invoke('resize_main_window', {
        width: Math.round(width),
        height: Math.round(height),
      }).catch((error: unknown) => {
        console.error('调整桌宠窗口尺寸失败:', error)
      })
    },
    { immediate: true },
  )
})

async function handlePetActivate(event: MouseEvent) {
  if (!shouldActivate(event)) return
  try {
    await invoke('toggle_chat_window')
  } catch (error) {
    console.error('切换聊天窗口失败:', error)
  }
}

async function handlePetContextMenu(event: MouseEvent) {
  try {
    await invoke('show_context_menu', {
      x: event.clientX,
      y: event.clientY,
      scale: petRef.value?.sizeScale ?? 1,
    })
  } catch (error) {
    console.error('打开原生菜单失败:', error)
  }
}

async function handlePetHover(hovering: boolean) {
  try {
    if (hovering) {
      await broadcastPetContext({ info: info.value, mood: mood.value })
    }
    await invoke('set_info_window_visible', { visible: hovering })
  } catch (error) {
    console.error('切换系统信息窗口失败:', error)
  }
}

onMounted(async () => {
  unlistenScale = await listen<number>(WINDOW_EVENTS.setScale, (event) => {
    petRef.value?.setSizeScale(event.payload)
  })
})

onUnmounted(() => unlistenScale?.())
</script>

<template>
  <main class="pet-window">
    <Pet
      ref="petRef"
      @press="handlePetPress"
      @activate="handlePetActivate"
      @open-menu="handlePetContextMenu"
      @hover-change="handlePetHover"
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
