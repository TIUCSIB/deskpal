<script setup lang="ts">
/** ReminderWindow.vue - 独立提醒浮窗 */
import { computed } from 'vue'
import ReminderBubble from '@/components/ReminderBubble.vue'
import { useOverlayTransition } from '@/composables/useOverlayTransition'
import { useReminderWindow } from '@/composables/useReminderWindow'

const {
  message,
  compact,
  snoozeText,
  completeReminder,
  dismissReminder,
  snoozeReminder,
  pauseUntilTomorrow,
} = useReminderWindow()
const { revision, transitionStyle } = useOverlayTransition()
const animatedStyle = computed(() => transitionStyle.value)
</script>

<template>
  <main
    class="reminder-window"
    :class="revision % 2 === 0 ? 'reminder-window--enter-a' : 'reminder-window--enter-b'"
    :style="animatedStyle"
    @keydown.esc="dismissReminder"
  >
    <ReminderBubble
      :message="message"
      :compact="compact"
      :snooze-text="snoozeText"
      @complete="completeReminder"
      @snooze="snoozeReminder"
      @pause-until-tomorrow="pauseUntilTomorrow"
    />
  </main>
</template>

<style scoped>
.reminder-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0 8px 0;
  background: transparent;
  transform-origin: var(--overlay-origin, center bottom);
}

.reminder-window--enter-a {
  animation: overlay-enter-a 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both;
}

.reminder-window--enter-b {
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
  .reminder-window {
    animation-duration: 1ms;
  }
}
</style>
