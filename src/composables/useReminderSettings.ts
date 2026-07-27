import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import { REMINDER_PRESETS, type ReminderPreset } from '@/config/reminderPresets'
import {
  DEFAULT_QUIET_HOURS,
  DEFAULT_REMINDER_INTERVAL_MINUTES,
  DEFAULT_REMINDER_MESSAGE,
  DEFAULT_REMINDER_SNOOZE_MINUTES,
  type AppSettings,
  type FixedTimeRepeat,
  type QuietHours,
  type Reminder,
  type ReminderInput,
  type ReminderSchedule,
} from '@/types/settings'

const INTERVAL_OPTIONS = [20, 25, 30, 45, 60, 90, 120]
const SNOOZE_OPTIONS = [5, 10, 15, 20, 30]
const WEEKDAY_LABELS = ['', '一', '二', '三', '四', '五', '六', '日']

export interface ReminderDraft {
  id: string | null
  message: string
  scheduleType: ReminderSchedule['type']
  intervalMinutes: number
  time: string
  repeatType: FixedTimeRepeat['type']
  weekdays: number[]
  snoozeMinutes: number
}

function normalizedRepeat(schedule: Extract<ReminderSchedule, { type: 'fixed_time' }>): FixedTimeRepeat {
  return schedule.repeat ?? { type: 'daily' }
}

function createDraft(reminder?: Reminder): ReminderDraft {
  if (!reminder) {
    return {
      id: null, message: DEFAULT_REMINDER_MESSAGE, scheduleType: 'interval', intervalMinutes: DEFAULT_REMINDER_INTERVAL_MINUTES,
      time: '09:00', repeatType: 'daily', weekdays: [1, 2, 3, 4, 5], snoozeMinutes: DEFAULT_REMINDER_SNOOZE_MINUTES,
    }
  }
  const repeat = reminder.schedule.type === 'fixed_time' ? normalizedRepeat(reminder.schedule) : { type: 'daily' } as const
  return {
    id: reminder.id,
    message: reminder.message,
    scheduleType: reminder.schedule.type,
    intervalMinutes: reminder.schedule.type === 'interval' ? reminder.schedule.interval_minutes : DEFAULT_REMINDER_INTERVAL_MINUTES,
    time: reminder.schedule.type === 'fixed_time' ? reminder.schedule.time : '09:00',
    repeatType: repeat.type,
    weekdays: repeat.type === 'custom_weekdays' ? repeat.weekdays : [1, 2, 3, 4, 5],
    snoozeMinutes: reminder.snooze_minutes,
  }
}

function createSchedule(value: ReminderDraft): ReminderSchedule {
  if (value.scheduleType === 'interval') return { type: 'interval', interval_minutes: value.intervalMinutes }
  const repeat: FixedTimeRepeat = value.repeatType === 'custom_weekdays'
    ? { type: 'custom_weekdays', weekdays: [...value.weekdays].sort() }
    : { type: value.repeatType }
  return { type: 'fixed_time', time: value.time, repeat }
}

/** useReminderSettings - 多提醒、免打扰与预设交互 */
export function useReminderSettings(
  settings: { value: AppSettings },
  invokeSetting: (command: string, payload?: Record<string, unknown>) => Promise<AppSettings>,
  setFeedback: (text: string) => void,
) {
  const draft = ref<ReminderDraft | null>(null)
  const deleteTarget = ref<Reminder | null>(null)
  const quietHoursDraft = ref<QuietHours>({ ...DEFAULT_QUIET_HOURS })
  const isEditing = computed(() => draft.value !== null)

  watch(() => settings.value.quiet_hours, () => syncQuietHoursDraft(), { immediate: true, deep: true })

  function syncQuietHoursDraft(value = settings.value.quiet_hours) {
    quietHoursDraft.value = { ...(value ?? DEFAULT_QUIET_HOURS) }
  }

  function openCreateEditor() { draft.value = createDraft() }
  function openEditEditor(reminder: Reminder) { draft.value = createDraft(reminder) }
  function cancelEditor() { draft.value = null }

  function updateDraft<K extends keyof ReminderDraft>(key: K, value: ReminderDraft[K]) {
    if (draft.value) draft.value = { ...draft.value, [key]: value }
  }

  function toggleWeekday(weekday: number) {
    if (!draft.value) return
    const weekdays = draft.value.weekdays.includes(weekday)
      ? draft.value.weekdays.filter((value) => value !== weekday)
      : [...draft.value.weekdays, weekday]
    updateDraft('weekdays', weekdays)
  }

  async function saveReminder() {
    const value = draft.value
    if (!value) return
    const message = value.message.trim()
    if (!message) return toast.error('请输入提醒文案')
    if (value.scheduleType === 'fixed_time' && !/^([01]\d|2[0-3]):[0-5]\d$/.test(value.time)) return toast.error('请选择有效的固定提醒时间')
    if (value.repeatType === 'custom_weekdays' && !value.weekdays.length) return toast.error('请至少选择一个提醒日')
    const input: ReminderInput = { message, schedule: createSchedule(value), snooze_minutes: value.snoozeMinutes }
    if (value.id) {
      const current = settings.value.reminders.find((reminder) => reminder.id === value.id)
      if (!current) return
      await invokeSetting('update_reminder', { reminder: { ...current, ...input } })
      setFeedback('提醒已更新')
    } else {
      await invokeSetting('create_reminder', { input })
      setFeedback('提醒已添加')
    }
    draft.value = null
  }

  async function createPreset(preset: ReminderPreset) {
    await invokeSetting('create_reminder', { input: preset.input })
    setFeedback(`已添加「${preset.label}」提醒`)
  }

  async function saveQuietHours() {
    const value = quietHoursDraft.value
    if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(value.start) || !/^([01]\d|2[0-3]):[0-5]\d$/.test(value.end)) {
      return toast.error('请选择有效的免打扰时间')
    }
    await invokeSetting('set_reminder_quiet_hours', { quietHours: value })
    syncQuietHoursDraft()
    setFeedback(value.enabled ? '免打扰时段已保存' : '免打扰已关闭')
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

  function requestDelete(reminder: Reminder) { deleteTarget.value = reminder }
  function cancelDelete() { deleteTarget.value = null }

  async function confirmDelete() {
    const reminder = deleteTarget.value
    if (!reminder) return
    await invokeSetting('delete_reminder', { id: reminder.id })
    if (draft.value?.id === reminder.id) draft.value = null
    deleteTarget.value = null
    setFeedback('提醒已删除')
  }

  function formatSchedule(schedule: ReminderSchedule) {
    if (schedule.type === 'interval') return `每 ${schedule.interval_minutes} 分钟`
    const repeat = normalizedRepeat(schedule)
    if (repeat.type === 'daily') return `每天 ${schedule.time}`
    if (repeat.type === 'weekdays') return `工作日 ${schedule.time}`
    const days = [...repeat.weekdays].sort().map((day) => `周${WEEKDAY_LABELS[day]}`).join('、')
    return `${days || '未选择日期'} ${schedule.time}`
  }

  function formatPause(reminder: Reminder) {
    if (!reminder.paused_until) return ''
    const value = new Date(reminder.paused_until)
    if (Number.isNaN(value.getTime())) return '已暂停'
    return `已暂停至 ${value.toLocaleString('zh-CN', { hour: '2-digit', minute: '2-digit', month: 'numeric', day: 'numeric' })}`
  }

  return {
    intervalOptions: INTERVAL_OPTIONS, snoozeOptions: SNOOZE_OPTIONS, presets: REMINDER_PRESETS, draft, deleteTarget, quietHoursDraft, isEditing,
    syncQuietHoursDraft, openCreateEditor, openEditEditor, cancelEditor, updateDraft, toggleWeekday, saveReminder, createPreset, saveQuietHours,
    setReminderEnabled, previewReminder, requestDelete, cancelDelete, confirmDelete, formatSchedule, formatPause,
  }
}
