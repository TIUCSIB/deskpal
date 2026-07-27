<script setup lang="ts">
/** ChatWindow.vue - 独立聊天输入窗口 */
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import ChatBubble from '@/components/ChatBubble.vue'
import { useOverlayTransition } from '@/composables/useOverlayTransition'
import { usePetContextReceiver } from '@/composables/useWindowBridge'
import { WINDOW_EVENTS } from '@/types/window'

const { context } = usePetContextReceiver('chat')
const { revision, transitionStyle } = useOverlayTransition()
const chatRef = ref<InstanceType<typeof ChatBubble> | null>(null)
let unlistenFocusInput: UnlistenFn | null = null
let unlistenWindowFocus: UnlistenFn | null = null
const BLUR_HIDE_GUARD_MS = 160
let hasFocusedSinceShow = false
let lastFocusAt = 0
const animatedStyle = computed(() => transitionStyle.value)

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
  <main
    class="chat-window"
    :class="revision % 2 === 0 ? 'chat-window--enter-a' : 'chat-window--enter-b'"
    :style="animatedStyle"
    @keydown.esc="hideChat"
  >
    <ChatBubble
      ref="chatRef"
      :info="context.info"
      :mood="context.mood"
      :role-id="context.roleId"
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
  transform-origin: var(--overlay-origin, center bottom);
}

.chat-window--enter-a {
  animation: overlay-enter-a 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both;
}

.chat-window--enter-b {
  animation: overlay-enter-b 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both;
}

@keyframes overlay-enter-a {
  from {
    opacity: 0;
    transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(0.94);
  }

  to {
    opacity: 1;
    transform: translate(0) scale(1);
  }
}

@keyframes overlay-enter-b {
  from {
    opacity: 0;
    transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(0.94);
  }

  to {
    opacity: 1;
    transform: translate(0) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .chat-window {
    animation-duration: 1ms;
  }
}
</style>
