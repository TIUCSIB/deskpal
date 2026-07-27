<script setup lang="ts">
/** ReminderBubble.vue - 桌宠提醒气泡 */
import { BellIcon } from '@lucide/vue'

const props = defineProps<{
  message: string
  compact: boolean
  snoozeText: string
}>()

const emit = defineEmits<{
  dismiss: []
  snooze: []
  pauseUntilTomorrow: []
}>()
</script>

<template>
  <section class="reminder-bubble" aria-label="桌宠提醒">
    <div class="reminder-bubble__surface">
      <div class="reminder-bubble__header">
        <BellIcon class="reminder-bubble__icon" />
        <span>提醒</span>
      </div>

      <p class="reminder-bubble__message" :class="{ 'reminder-bubble__message--compact': props.compact }">
        {{ props.message }}
      </p>

      <div class="reminder-bubble__footer">
        <button
          class="reminder-bubble__pause"
          type="button"
          title="明天再提醒"
          @click="emit('pauseUntilTomorrow')"
        >
          明天再提醒
        </button>
        <div class="reminder-bubble__actions">
          <button
            class="reminder-bubble__button reminder-bubble__button--secondary"
            type="button"
            title="稍后提醒"
            @click="emit('snooze')"
          >
            {{ props.snoozeText }}
          </button>
          <button
            class="reminder-bubble__button reminder-bubble__button--primary"
            type="button"
            title="知道了"
            @click="emit('dismiss')"
          >
            ✓
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.reminder-bubble {
  width: 232px;
  max-width: 100%;
  color: #1c1c1e;
  pointer-events: auto;
}

.reminder-bubble__surface {
  width: 100%;
  display: grid;
  gap: 7px;
  padding: 8px 6px 8px 12px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.97);
  border: 1px solid rgba(60, 60, 67, 0.16);
  border-radius: 20px;
  transition:
    padding 180ms ease,
    border-radius 180ms ease,
    opacity 160ms ease;
}

.reminder-bubble__header {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: #3a3a3c;
  font-size: 12px;
  line-height: 1;
}

.reminder-bubble__icon {
  width: 14px;
  height: 14px;
}

.reminder-bubble__message {
  display: -webkit-box;
  margin: 0;
  overflow: hidden;
  color: #1c1c1e;
  font-size: 12px;
  line-height: 1.35;
  letter-spacing: 0;
  word-break: break-word;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.reminder-bubble__message--compact {
  font-size: 12px;
}

.reminder-bubble__footer {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
}

.reminder-bubble__pause {
  min-width: 0;
  justify-self: start;
  padding: 0;
  overflow: hidden;
  color: #8e8e93;
  background: transparent;
  border: 0;
  cursor: pointer;
  font-size: 11px;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.reminder-bubble__pause:hover {
  color: #007aff;
}

.reminder-bubble__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.reminder-bubble__button {
  height: 30px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 999px;
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
}

.reminder-bubble__button--secondary {
  min-width: 44px;
  padding: 0 8px;
  color: #007aff;
  background: #f2f2f7;
}

.reminder-bubble__button--primary {
  width: 30px;
  color: #fff;
  background: #007aff;
  font-size: 16px;
  font-weight: 700;
}

</style>
