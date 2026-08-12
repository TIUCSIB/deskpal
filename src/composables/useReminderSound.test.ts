import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useReminderSound } from '@/composables/useReminderSound'

const mocks = vi.hoisted(() => ({
  handler: null as (() => void) | null,
  unlisten: vi.fn(),
  oscillator: {
    type: 'sine',
    frequency: {
      setValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
    start: vi.fn(),
    stop: vi.fn(),
  },
  gain: {
    gain: {
      setValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
  },
  close: vi.fn(async () => undefined),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_eventName: string, handler: () => void) => {
    mocks.handler = handler
    return mocks.unlisten
  }),
}))

class MockAudioContext {
  currentTime = 0
  destination = {}

  createOscillator() {
    return mocks.oscillator
  }

  createGain() {
    return mocks.gain
  }

  close() {
    return mocks.close()
  }
}

const Host = defineComponent({
  setup() {
    useReminderSound()
    return () => h('div')
  },
})

describe('useReminderSound', () => {
  beforeEach(() => {
    mocks.handler = null
    mocks.unlisten.mockReset()
    mocks.oscillator.connect.mockImplementation(() => mocks.gain)
    mocks.gain.connect.mockImplementation(() => mocks.gain)
    vi.stubGlobal('AudioContext', MockAudioContext)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.clearAllMocks()
  })

  it('plays a short tone when the reminder event arrives', async () => {
    const wrapper = mount(Host)
    await Promise.resolve()
    await nextTick()

    mocks.handler?.()

    expect(mocks.oscillator.start).toHaveBeenCalledOnce()
    expect(mocks.oscillator.stop).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('cleans up the listener when unmounted', async () => {
    const wrapper = mount(Host)
    await Promise.resolve()
    await nextTick()

    wrapper.unmount()

    expect(mocks.unlisten).toHaveBeenCalledOnce()
    expect(mocks.close).not.toHaveBeenCalled()
  })
})
