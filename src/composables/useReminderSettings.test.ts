import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import { useReminderSettings } from '@/composables/useReminderSettings'
import { DEFAULT_APP_SETTINGS, type AppSettings } from '@/types/settings'

const mocks = vi.hoisted(() => ({
  toastError: vi.fn(),
}))

vi.mock('vue-sonner', () => ({
  toast: { error: mocks.toastError },
}))

function createHarness() {
  const settings = ref<AppSettings>({ ...DEFAULT_APP_SETTINGS, reminders: [] })
  const invokeSetting = vi.fn(async (_command: string, payload?: Record<string, unknown>) => {
    const input = payload?.input as AppSettings['reminders'][number] | undefined
    if (input) {
      settings.value = {
        ...settings.value,
        reminders: [...settings.value.reminders, { ...input, id: 'created', enabled: true, paused_until: null }],
      }
    }
    return settings.value
  })
  const setFeedback = vi.fn()
  return { settings, invokeSetting, setFeedback, state: useReminderSettings(settings, invokeSetting, setFeedback) }
}

describe('useReminderSettings', () => {
  beforeEach(() => {
    mocks.toastError.mockReset()
  })

  it('creates the clock-out preset with its approved weekday schedule', async () => {
    const { state, invokeSetting, setFeedback } = createHarness()
    const clockOut = state.presets.find((preset) => preset.id === 'clock-out')

    await state.createPreset(clockOut!)

    expect(invokeSetting).toHaveBeenCalledWith('create_reminder', {
      input: {
        message: '到下班时间了，别忘了打卡',
        schedule: { type: 'fixed_time', time: '18:00', repeat: { type: 'weekdays' } },
        snooze_minutes: 10,
      },
    })
    expect(setFeedback).toHaveBeenCalledWith('已添加「下班打卡」提醒')
  })

  it('formats ISO custom weekdays in Monday-to-Sunday order', () => {
    const { state } = createHarness()

    expect(state.formatSchedule({
      type: 'fixed_time',
      time: '09:00',
      repeat: { type: 'custom_weekdays', weekdays: [7, 1, 5] },
    })).toBe('周一、周五、周日 09:00')
  })

  it('rejects saving a custom fixed reminder without selected days', async () => {
    const { state, invokeSetting } = createHarness()
    state.openCreateEditor()
    state.updateDraft('scheduleType', 'fixed_time')
    state.updateDraft('repeatType', 'custom_weekdays')
    state.updateDraft('weekdays', [])

    await state.saveReminder()

    expect(mocks.toastError).toHaveBeenCalledWith('请至少选择一个提醒日')
    expect(invokeSetting).not.toHaveBeenCalled()
  })

  it('saves global quiet hours using the Tauri command payload', async () => {
    const { state, invokeSetting, setFeedback } = createHarness()
    state.quietHoursDraft.value = { enabled: true, start: '23:00', end: '08:00' }

    await state.saveQuietHours()

    expect(invokeSetting).toHaveBeenCalledWith('set_reminder_quiet_hours', {
      quietHours: { enabled: true, start: '23:00', end: '08:00' },
    })
    expect(setFeedback).toHaveBeenCalledWith('免打扰时段已保存')
  })
})
