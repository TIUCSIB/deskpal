import { describe, expect, it } from 'vitest'
import { REMINDER_PRESETS } from '@/config/reminderPresets'

describe('REMINDER_PRESETS', () => {
  it('includes the approved interval reminder shortcuts', () => {
    const intervals = Object.fromEntries(
      REMINDER_PRESETS.flatMap((preset) => {
        const schedule = preset.input.schedule
        return schedule.type === 'interval' ? [[preset.id, schedule.interval_minutes]] : []
      }),
    )

    expect(intervals).toMatchObject({ water: 30, sedentary: 60, rest: 90, 'eye-care': 45, pomodoro: 25 })
  })

  it('creates a weekday clock-out reminder at 18:00', () => {
    const clockOut = REMINDER_PRESETS.find((preset) => preset.id === 'clock-out')

    expect(clockOut?.input.schedule).toEqual({
      type: 'fixed_time',
      time: '18:00',
      repeat: { type: 'weekdays' },
    })
  })
})
