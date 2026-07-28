<script setup lang="ts">
/** SystemFeedbackWindow.vue - 独立系统反馈浮窗 */
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SystemFeedbackBubble from '@/components/SystemFeedbackBubble.vue'
import { useOverlayTransition } from '@/composables/useOverlayTransition'
import { useSystemFeedbackPayloadReceiver } from '@/composables/useWindowBridge'
import type { SystemFeedbackPayload } from '@/types/systemFeedback'

const { payload } = useSystemFeedbackPayloadReceiver()
const { revision, transitionStyle } = useOverlayTransition()
const fallbackPayload = ref<SystemFeedbackPayload | null>(null)
const activePayload = computed(() => payload.value ?? fallbackPayload.value)

async function refreshActivePayload() {
  fallbackPayload.value = await invoke<SystemFeedbackPayload | null>('active_system_feedback_payload')
}

async function dismissFeedback() {
  if (!activePayload.value) return
  await invoke('dismiss_system_feedback', { id: activePayload.value.id })
  payload.value = null
  fallbackPayload.value = null
}

onMounted(() => {
  void refreshActivePayload()
})
</script>

<template>
  <main
    v-if="activePayload"
    class="system-feedback-window"
    :class="revision % 2 === 0 ? 'system-feedback-window--enter-a' : 'system-feedback-window--enter-b'"
    :style="transitionStyle"
    @keydown.esc="dismissFeedback"
  >
    <SystemFeedbackBubble :payload="activePayload" @close="dismissFeedback" />
  </main>
</template>

<style scoped>
.system-feedback-window {
  display: flex;
  width: 100%;
  height: 100%;
  align-items: flex-end;
  justify-content: center;
  padding: 0 8px;
  background: transparent;
  transform-origin: var(--overlay-origin, center bottom);
}
.system-feedback-window--enter-a { animation: overlay-enter-a 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both; }
.system-feedback-window--enter-b { animation: overlay-enter-b 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both; }
@keyframes overlay-enter-a { from { opacity: 0; transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(0.94); } to { opacity: 1; transform: translate(0) scale(1); } }
@keyframes overlay-enter-b { from { opacity: 0; transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(0.94); } to { opacity: 1; transform: translate(0) scale(1); } }
@media (prefers-reduced-motion: reduce) { .system-feedback-window { animation-duration: 1ms; } }
</style>
