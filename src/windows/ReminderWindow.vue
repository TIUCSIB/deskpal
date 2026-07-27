<script setup lang="ts">
/** ReminderWindow.vue - 独立提醒浮窗 */
import ReminderBubble from '@/components/ReminderBubble.vue'
import { useReminderWindow } from '@/composables/useReminderWindow'

const {
  message,
  compact,
  snoozeText,
  dismissReminder,
  snoozeReminder,
  pauseUntilTomorrow,
} = useReminderWindow()
</script>

<template>
  <main class="reminder-window" @keydown.esc="dismissReminder">
    <ReminderBubble
      :message="message"
      :compact="compact"
      :snooze-text="snoozeText"
      @dismiss="dismissReminder"
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
  opacity: 0;
  transform: translateY(0) scale(1);
  transform-origin: bottom center;
  animation: reminder-bubble-pop 260ms cubic-bezier(0.2, 1.3, 0.32, 1) forwards;
}

@keyframes reminder-bubble-pop {
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
