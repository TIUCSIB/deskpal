import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReminderPayload } from '@/types/window'
import { usePetContextReceiver, useReminderPayloadReceiver } from '@/composables/useWindowBridge'

/** useReminderWindow - 提醒浮窗展示与交互 */
export function useReminderWindow() {
  const { context } = usePetContextReceiver()
  const { payload } = useReminderPayloadReceiver()

  const message = computed(() => payload.value.message || '记得休息一下～')
  const snoozeText = computed(() => `${payload.value.snooze_minutes} 分钟后`)
  const compact = computed(() => context.value.scale < 0.72)

  async function refreshActivePayload() {
    const active = await invoke<ReminderPayload | null>('active_reminder_payload')
    if (active) payload.value = active
  }

  async function dismissReminder() {
    await invoke('dismiss_reminder_window')
  }

  async function snoozeReminder() {
    await invoke('snooze_reminder')
  }

  onMounted(() => {
    void refreshActivePayload()
  })

  return {
    message,
    compact,
    snoozeText,
    dismissReminder,
    snoozeReminder,
  }
}
