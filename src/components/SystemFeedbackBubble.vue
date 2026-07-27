<script setup lang="ts">
/** SystemFeedbackBubble.vue - 系统状态的轻量反馈气泡 */
import { computed } from 'vue'
import type { SystemFeedbackPayload } from '@/types/systemFeedback'

const props = defineProps<{ payload: SystemFeedbackPayload }>()
const emit = defineEmits<{ close: [] }>()

const icon = computed(() => (props.payload.severity === 'warning' ? '!' : '✓'))
</script>

<template>
  <section
    class="system-feedback-bubble"
    :class="`system-feedback-bubble--${payload.severity}`"
    aria-live="polite"
    :aria-label="payload.title"
  >
    <span class="system-feedback-bubble__icon" aria-hidden="true">{{ icon }}</span>
    <div class="system-feedback-bubble__content">
      <strong class="system-feedback-bubble__title">{{ payload.title }}</strong>
      <p class="system-feedback-bubble__message">{{ payload.message }}</p>
    </div>
    <button class="system-feedback-bubble__close" type="button" aria-label="关闭系统反馈" @click="emit('close')">×</button>
  </section>
</template>

<style scoped>
.system-feedback-bubble {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 20px;
  gap: 8px;
  align-items: start;
  box-sizing: border-box;
  width: 232px;
  padding: 11px 12px;
  color: #1c1c1e;
  background: rgba(255, 255, 255, 0.97);
  border: 1px solid rgba(60, 60, 67, 0.16);
  border-radius: 14px;
  box-shadow: 0 8px 24px rgba(28, 28, 30, 0.13);
}

.system-feedback-bubble--warning { border-color: rgba(255, 149, 0, 0.45); }
.system-feedback-bubble__icon {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  color: #fff;
  background: #34c759;
  border-radius: 50%;
  font-size: 14px;
  font-weight: 800;
}
.system-feedback-bubble--warning .system-feedback-bubble__icon { background: #ff9500; }
.system-feedback-bubble__content { min-width: 0; }
.system-feedback-bubble__title { display: block; font-size: 12px; line-height: 1.4; }
.system-feedback-bubble__message { margin: 2px 0 0; color: #636366; font-size: 11px; line-height: 1.45; }
.system-feedback-bubble__close {
  width: 20px;
  height: 20px;
  padding: 0;
  color: #8e8e93;
  background: transparent;
  border: 0;
  border-radius: 50%;
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}
.system-feedback-bubble__close:hover { color: #1c1c1e; background: rgba(60, 60, 67, 0.08); }
</style>
