import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import InfoPanel from '@/components/InfoPanel.vue'
import type { SystemInfo } from '@/types/system'

function createInfo(overrides: Partial<SystemInfo> = {}): SystemInfo {
  return {
    cpu_usage: 12.5,
    memory_usage: 45.2,
    memory_used_mb: 4096,
    memory_total_mb: 8192,
    disk_usage: 61.7,
    network_down_kbps: 128.4,
    network_up_kbps: 32.1,
    network_connected: true,
    battery_percent: 73,
    battery_charging: true,
    idle_seconds: 3661,
    uptime_secs: 7200,
    ...overrides,
  }
}

describe('InfoPanel', () => {
  it('shows Chinese network, battery, idle, and throughput information', () => {
    const wrapper = mount(InfoPanel, { props: { info: createInfo() } })

    expect(wrapper.text()).toContain('已连接 · ↓ 128.4 / ↑ 32.1 KB/s')
    expect(wrapper.text()).toContain('73% · 已接通电源')
    expect(wrapper.text()).toContain('1小时1分钟')
    expect(wrapper.get('[aria-label^="网络："]').attributes('aria-label')).toBe(
      '网络：已连接，↓ 128.4 / ↑ 32.1 KB/s',
    )
    expect(wrapper.get('[aria-label^="电池："]').attributes('aria-label')).toBe(
      '电池：电量 73% ，已接通电源',
    )
  })

  it('hides battery when the device has no battery hardware', () => {
    const wrapper = mount(InfoPanel, {
      props: { info: createInfo({ battery_percent: null, battery_charging: null }) },
    })

    expect(wrapper.text()).not.toContain('电池')
    expect(wrapper.find('[aria-label^="电池："]').exists()).toBe(false)
  })

  it('labels disconnected and unavailable system states', () => {
    const wrapper = mount(InfoPanel, {
      props: {
        info: createInfo({
          network_connected: false,
          idle_seconds: null,
          battery_charging: false,
        }),
      },
    })

    expect(wrapper.text()).toContain('未连接 · ↓ 128.4 / ↑ 32.1 KB/s')
    expect(wrapper.text()).toContain('73% · 使用电池')
    expect(wrapper.text()).toContain('暂不可用')
    expect(wrapper.findAll('[aria-label="空闲时间：暂不可用"]')).toHaveLength(1)
  })
})
