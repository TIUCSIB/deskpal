import { describe, expect, it } from 'vitest'
import { isWithinQuietHours, useSystemFeedback } from '@/composables/useSystemFeedback'
import type { QuietHours } from '@/types/settings'
import type { SystemInfo } from '@/types/system'

const QUIET_HOURS: QuietHours = { enabled: false, start: '23:00', end: '08:00' }

function createInfo(overrides: Partial<SystemInfo> = {}): SystemInfo {
  return {
    cpu_usage: 40,
    memory_usage: 60,
    memory_used_mb: 4096,
    memory_total_mb: 8192,
    disk_usage: 50,
    network_down_kbps: 0,
    network_up_kbps: 0,
    network_connected: null,
    battery_percent: null,
    battery_charging: null,
    idle_seconds: null,
    uptime_secs: 3600,
    ...overrides,
  }
}

describe('useSystemFeedback', () => {
  it('only reports sustained CPU load and respects cooldown', () => {
    let timestamp = 0
    const feedback = useSystemFeedback(() => timestamp)

    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), QUIET_HOURS)).toBeNull()
    timestamp = 3 * 60 * 1000 - 1
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), QUIET_HOURS)).toBeNull()
    timestamp += 1
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), QUIET_HOURS)?.kind).toBe('cpu_high')
    timestamp += 1000
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), QUIET_HOURS)).toBeNull()
    timestamp += 30 * 60 * 1000
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), QUIET_HOURS)?.kind).toBe('cpu_high')
  })

  it('requires recovery before restarting a high memory duration', () => {
    let timestamp = 0
    const feedback = useSystemFeedback(() => timestamp)

    feedback.evaluate(createInfo({ memory_usage: 92 }), QUIET_HOURS)
    timestamp = 60 * 1000
    feedback.evaluate(createInfo({ memory_usage: 80 }), QUIET_HOURS)
    timestamp = 3 * 60 * 1000
    expect(feedback.evaluate(createInfo({ memory_usage: 92 }), QUIET_HOURS)).toBeNull()
    timestamp += 3 * 60 * 1000
    expect(feedback.evaluate(createInfo({ memory_usage: 92 }), QUIET_HOURS)?.kind).toBe('memory_high')
  })

  it('keeps tracking during quiet hours without emitting', () => {
    let timestamp = new Date(2026, 0, 1, 23, 0).getTime()
    const feedback = useSystemFeedback(() => timestamp)
    const quietHours: QuietHours = { enabled: true, start: '22:00', end: '08:00' }

    feedback.evaluate(createInfo({ cpu_usage: 90 }), quietHours)
    timestamp += 3 * 60 * 1000
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), quietHours)).toBeNull()
    timestamp = new Date(2026, 0, 2, 8, 0).getTime()
    expect(feedback.evaluate(createInfo({ cpu_usage: 90 }), quietHours)?.kind).toBe('cpu_high')
  })

  it('reports network restoration, low battery and long idle independently', () => {
    let timestamp = 0
    const feedback = useSystemFeedback(() => timestamp)

    feedback.evaluate(createInfo({ network_connected: false }), QUIET_HOURS)
    timestamp += 1
    expect(feedback.evaluate(createInfo({ network_connected: true }), QUIET_HOURS)?.kind).toBe('network_restored')
    timestamp += 1
    expect(feedback.evaluate(createInfo({ battery_percent: 18, battery_charging: false }), QUIET_HOURS)?.kind).toBe('battery_low')
    timestamp += 1
    expect(feedback.evaluate(createInfo({ idle_seconds: 60 * 60 }), QUIET_HOURS)?.kind).toBe('idle_long')
  })

  it('handles same-day and cross-day quiet hours', () => {
    const crossDay: QuietHours = { enabled: true, start: '23:00', end: '08:00' }
    const sameDay: QuietHours = { enabled: true, start: '09:00', end: '17:00' }

    expect(isWithinQuietHours(crossDay, new Date(2026, 0, 1, 1, 0))).toBe(true)
    expect(isWithinQuietHours(crossDay, new Date(2026, 0, 1, 12, 0))).toBe(false)
    expect(isWithinQuietHours(sameDay, new Date(2026, 0, 1, 12, 0))).toBe(true)
    expect(isWithinQuietHours(sameDay, new Date(2026, 0, 1, 18, 0))).toBe(false)
  })
})
