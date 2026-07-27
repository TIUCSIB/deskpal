import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import type { AppSettings } from '@/types/settings'

const INTERVAL_OPTIONS = [20, 30, 45, 60, 90, 120]
const SNOOZE_OPTIONS = [5, 10, 15, 20, 30]

/** useReminderSettings - 提醒设置交互 */
export function useReminderSettings(
  settings: { value: AppSettings },
  invokeSetting: (command: string, payload?: Record<string, unknown>) => Promise<AppSettings>,
  setFeedback: (text: string) => void,
) {
  const reminderMessageDraft = ref('')

  function syncDraft() {
    reminderMessageDraft.value = settings.value.reminder.message
  }

  async function handleReminderEnabledChange(enabled: boolean) {
    await invokeSetting('set_reminder_enabled', { enabled })
    setFeedback(enabled ? '提醒功能已开启' : '提醒功能已关闭')
  }

  async function handleReminderIntervalChange(intervalMinutes: number) {
    await invokeSetting('set_reminder_interval', { intervalMinutes })
    setFeedback(`提醒间隔已调整为 ${intervalMinutes} 分钟`)
  }

  async function handleReminderSnoozeChange(snoozeMinutes: number) {
    await invokeSetting('set_reminder_snooze_minutes', { snoozeMinutes })
    setFeedback(`稍后提醒时长已调整为 ${snoozeMinutes} 分钟`)
  }

  function handleReminderDraftInput(value: string) {
    reminderMessageDraft.value = value
  }

  async function applyReminderMessage() {
    const message = reminderMessageDraft.value.trim()
    if (!message) {
      toast.error('请输入提醒文案')
      return
    }
    await invokeSetting('set_reminder_message', { message })
    setFeedback('提醒文案已更新')
  }

  async function previewReminder() {
    try {
      await invoke('preview_reminder_window')
      setFeedback('已显示测试提醒')
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : '测试提醒显示失败'
      toast.error(message)
    }
  }

  return {
    intervalOptions: INTERVAL_OPTIONS,
    snoozeOptions: SNOOZE_OPTIONS,
    reminderMessageDraft,
    syncDraft,
    handleReminderEnabledChange,
    handleReminderIntervalChange,
    handleReminderSnoozeChange,
    handleReminderDraftInput,
    applyReminderMessage,
    previewReminder,
  }
}
