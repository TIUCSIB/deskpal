import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import {
  DEFAULT_REMINDER_INTERVAL_MINUTES,
  DEFAULT_REMINDER_MESSAGE,
  DEFAULT_REMINDER_SNOOZE_MINUTES,
  type AppSettings,
  type Reminder,
  type ReminderInput,
  type ReminderSchedule,
} from '@/types/settings'

const INTERVAL_OPTIONS = [20, 30, 45, 60, 90, 120]
const SNOOZE_OPTIONS = [5, 10, 15, 20, 30]

interface ReminderDraft {
  id: string | null
  message: string
  scheduleType: ReminderSchedule['type']
  intervalMinutes: number
  time: string
  snoozeMinutes: number
}

function createDraft(reminder?: Reminder): ReminderDraft {
  if (!reminder) {
    return {
      id: null,
      message: DEFAULT_REMINDER_MESSAGE,
      scheduleType: 'interval',
      intervalMinutes: DEFAULT_REMINDER_INTERVAL_MINUTES,
      time: '09:00',
      snoozeMinutes: DEFAULT_REMINDER_SNOOZE_MINUTES,
    }
  }
  return {
    id: reminder.id,
    message: reminder.message,
    scheduleType: reminder.schedule.type,
    intervalMinutes: reminder.schedule.type === 'interval'
      ? reminder.schedule.interval_minutes
      : DEFAULT_REMINDER_INTERVAL_MINUTES,
    time: reminder.schedule.type === 'fixed_time' ? reminder.schedule.time : '09:00',
    snoozeMinutes: reminder.snooze_minutes,
  }
}

/** useReminderSettings - 多提醒设置交互 */
export function useReminderSettings(
  settings: { value: AppSettings },
  invokeSetting: (command: string, payload?: Record<string, unknown>) => Promise<AppSettings>,
  setFeedback: (text: string) => void,
) {
  const draft = ref<ReminderDraft | null>(null)
  const deleteTarget = ref<Reminder | null>(null)
  const isEditing = computed(() => draft.value !== null)

  function openCreateEditor() {
    draft.value = createDraft()
  }

  function openEditEditor(reminder: Reminder) {
    draft.value = createDraft(reminder)
  }

  function cancelEditor() {
    draft.value = null
  }

  function updateDraft<K extends keyof ReminderDraft>(key: K, value: ReminderDraft[K]) {
    if (!draft.value) return
    draft.value = { ...draft.value, [key]: value }
  }

  async function saveReminder() {
    const value = draft.value
    if (!value) return
    const message = value.message.trim()
    if (!message) {
      toast.error('请输入提醒文案')
      return
    }
    if (value.scheduleType === 'fixed_time' && !/^\d{2}:\d{2}$/.test(value.time)) {
      toast.error('请选择有效的固定提醒时间')
      return
    }
    const schedule: ReminderSchedule = value.scheduleType === 'interval'
      ? { type: 'interval', interval_minutes: value.intervalMinutes }
      : { type: 'fixed_time', time: value.time }
    const input: ReminderInput = {
      message,
      schedule,
      snooze_minutes: value.snoozeMinutes,
    }
    if (value.id) {
      const current = settings.value.reminders.find((reminder) => reminder.id === value.id)
      if (!current) return
      await invokeSetting('update_reminder', {
        reminder: { ...current, ...input },
      })
      setFeedback('提醒已更新')
    } else {
      await invokeSetting('create_reminder', { input })
      setFeedback('提醒已添加')
    }
    draft.value = null
  }

  async function setReminderEnabled(id: string, enabled: boolean) {
    await invokeSetting('set_reminder_enabled', { id, enabled })
    setFeedback(enabled ? '提醒已开启' : '提醒已关闭')
  }

  async function previewReminder(id: string) {
    try {
      await invoke('preview_reminder_window', { reminderId: id })
      setFeedback('已显示测试提醒')
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '测试提醒显示失败')
    }
  }

  function requestDelete(reminder: Reminder) {
    deleteTarget.value = reminder
  }

  function cancelDelete() {
    deleteTarget.value = null
  }

  async function confirmDelete() {
    const reminder = deleteTarget.value
    if (!reminder) return
    await invokeSetting('delete_reminder', { id: reminder.id })
    if (draft.value?.id === reminder.id) draft.value = null
    deleteTarget.value = null
    setFeedback('提醒已删除')
  }

  function formatSchedule(schedule: ReminderSchedule) {
    return schedule.type === 'interval'
      ? `每 ${schedule.interval_minutes} 分钟`
      : `每天 ${schedule.time}`
  }

  function formatPause(reminder: Reminder) {
    if (!reminder.paused_until) return ''
    const value = new Date(reminder.paused_until)
    if (Number.isNaN(value.getTime())) return '已暂停'
    return `已暂停至 ${value.toLocaleString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      month: 'numeric',
      day: 'numeric',
    })}`
  }

  return {
    intervalOptions: INTERVAL_OPTIONS,
    snoozeOptions: SNOOZE_OPTIONS,
    draft,
    deleteTarget,
    isEditing,
    openCreateEditor,
    openEditEditor,
    cancelEditor,
    updateDraft,
    saveReminder,
    setReminderEnabled,
    previewReminder,
    requestDelete,
    cancelDelete,
    confirmDelete,
    formatSchedule,
    formatPause,
  }
}
