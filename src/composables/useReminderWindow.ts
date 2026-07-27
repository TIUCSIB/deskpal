import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
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

  async function runAction(command: string) {
    if (!payload.value.reminder_id) return
    try {
      await invoke(command, { reminderId: payload.value.reminder_id })
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '提醒操作失败')
    }
  }

  async function dismissReminder() {
    await runAction('dismiss_reminder_window')
  }

  async function snoozeReminder() {
    await runAction('snooze_reminder')
  }

  async function pauseUntilTomorrow() {
    await runAction('pause_reminder_until_tomorrow')
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
    pauseUntilTomorrow,
  }
}
