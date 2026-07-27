import { describe, expect, it } from 'vitest'
import { derivePetMood } from '@/composables/usePetState'
import type { SystemInfo } from '@/types/system'

function createInfo(overrides: Partial<SystemInfo> = {}): SystemInfo {
  return {
    ...overrides,
    cpu_usage: overrides.cpu_usage ?? 40,
    memory_usage: overrides.memory_usage ?? 60,
    memory_used_mb: overrides.memory_used_mb ?? 4096,
    memory_total_mb: overrides.memory_total_mb ?? 8192,
    disk_usage: overrides.disk_usage ?? 50,
    network_down_kbps: overrides.network_down_kbps ?? 0,
    network_up_kbps: overrides.network_up_kbps ?? 0,
    network_connected: overrides.network_connected ?? true,
    battery_percent: overrides.battery_percent ?? null,
    battery_charging: overrides.battery_charging ?? null,
    idle_seconds: overrides.idle_seconds ?? null,
    uptime_secs: overrides.uptime_secs ?? 3600,
  }
}

describe('derivePetMood', () => {
  it('prioritizes warning above other conditions', () => {
    expect(derivePetMood(createInfo({ cpu_usage: 81 }), 2)).toBe('warning')
    expect(derivePetMood(createInfo({ memory_usage: 86 }), 2)).toBe('warning')
  })

  it('uses sleepy mood during the overnight period', () => {
    expect(derivePetMood(createInfo({ cpu_usage: 20, memory_usage: 20 }), 0)).toBe('sleepy')
    expect(derivePetMood(createInfo({ cpu_usage: 20, memory_usage: 20 }), 5)).toBe('sleepy')
  })

  it('uses happy mood only for low system usage outside overnight hours', () => {
    expect(derivePetMood(createInfo({ cpu_usage: 29, memory_usage: 49 }), 9)).toBe('happy')
    expect(derivePetMood(createInfo({ cpu_usage: 30, memory_usage: 49 }), 9)).toBe('normal')
    expect(derivePetMood(createInfo({ cpu_usage: 29, memory_usage: 50 }), 9)).toBe('normal')
  })
})
