<script setup lang="ts">
/** ChatWindow.vue - 独立聊天输入窗口 */
import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import ChatBubble from '@/components/ChatBubble.vue'
import { usePetContextReceiver } from '@/composables/useWindowBridge'
import { WINDOW_EVENTS } from '@/types/window'

const { context } = usePetContextReceiver()
const chatRef = ref<InstanceType<typeof ChatBubble> | null>(null)
let unlistenFocusInput: UnlistenFn | null = null
let unlistenWindowFocus: UnlistenFn | null = null
const BLUR_HIDE_GUARD_MS = 160
let hasFocusedSinceShow = false
let lastFocusAt = 0

async function hideChat() {
  chatRef.value?.resetSession()
  await invoke('hide_chat_window')
}

onMounted(async () => {
  unlistenFocusInput = await listen(WINDOW_EVENTS.focusChatInput, () => {
    hasFocusedSinceShow = true
    lastFocusAt = Date.now()
    chatRef.value?.focusInput()
  })
  unlistenWindowFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload) {
      hasFocusedSinceShow = true
      lastFocusAt = Date.now()
      return
    }
    if (!hasFocusedSinceShow) return
    if (Date.now() - lastFocusAt < BLUR_HIDE_GUARD_MS) return
    hasFocusedSinceShow = false
    void hideChat()
  })
})

onUnmounted(() => {
  unlistenFocusInput?.()
  unlistenWindowFocus?.()
})
</script>

<template>
  <main class="chat-window" @keydown.esc="hideChat">
    <ChatBubble
      ref="chatRef"
      :info="context.info"
      :mood="context.mood"
    />
  </main>
</template>

<style scoped>
.chat-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0 8px 0;
  background: transparent;
  opacity: 0;
  transform: translateY(0) scale(1);
  transform-origin: bottom center;
  animation: chat-bubble-pop 260ms cubic-bezier(0.2, 1.3, 0.32, 1) forwards;
}

@keyframes chat-bubble-pop {
  0% {
    opacity: 0;
    transform: translateY(10px) scale(0.9);
  }

  72% {
    opacity: 1;
    transform: translateY(-1px) scale(1.015);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
