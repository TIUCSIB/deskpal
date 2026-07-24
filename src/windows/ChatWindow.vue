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
let hasFocusedSinceShow = false

async function hideChat() {
  await invoke('hide_chat_window')
}

onMounted(async () => {
  unlistenFocusInput = await listen(WINDOW_EVENTS.focusChatInput, () => {
    hasFocusedSinceShow = true
    chatRef.value?.focusInput()
  })
  unlistenWindowFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload) {
      hasFocusedSinceShow = true
      return
    }
    if (!hasFocusedSinceShow) return
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
      @close="hideChat"
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
  padding: 8px;
  background: transparent;
}
</style>
