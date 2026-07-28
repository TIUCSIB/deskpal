import { defineComponent, h, ref } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useContextMenuFocus } from '@/composables/useContextMenuFocus'

const mocks = vi.hoisted(() => ({
  focusChangedHandler: null as ((event: { payload: boolean }) => void) | null,
  focusEventHandler: null as (() => void) | null,
  listen: vi.fn(),
  onFocusChanged: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onFocusChanged: mocks.onFocusChanged }),
}))

const TestHost = defineComponent({
  setup() {
    const focusCount = ref(0)
    const hideCount = ref(0)
    const hide = vi.fn(async () => { hideCount.value += 1 })
    useContextMenuFocus(() => { focusCount.value += 1 }, hide)
    return () => h('output', {
      'data-focus-count': focusCount.value,
      'data-hide-count': hideCount.value,
    })
  },
})

describe('useContextMenuFocus', () => {
  beforeEach(() => {
    mocks.focusChangedHandler = null
    mocks.focusEventHandler = null
    mocks.listen.mockImplementation(async (_eventName: string, handler: () => void) => {
      mocks.focusEventHandler = handler
      return vi.fn()
    })
    mocks.onFocusChanged.mockImplementation(async (handler: (event: { payload: boolean }) => void) => {
      mocks.focusChangedHandler = handler
      return vi.fn()
    })
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  it('treats native focus events as a new menu session', async () => {
    const wrapper = mount(TestHost)
    await flushPromises()

    mocks.focusEventHandler?.()
    await flushPromises()

    expect(wrapper.attributes('data-focus-count')).toBe('1')
    wrapper.unmount()
  })

  it('closes after a blur inside the short focus guard', async () => {
    vi.useFakeTimers()
    const wrapper = mount(TestHost)
    await flushPromises()

    mocks.focusChangedHandler?.({ payload: true })
    mocks.focusChangedHandler?.({ payload: false })
    await vi.advanceTimersByTimeAsync(160)

    expect(wrapper.attributes('data-hide-count')).toBe('1')
    wrapper.unmount()
  })

  it('cancels a pending blur close when focus returns', async () => {
    vi.useFakeTimers()
    const wrapper = mount(TestHost)
    await flushPromises()

    mocks.focusChangedHandler?.({ payload: true })
    mocks.focusChangedHandler?.({ payload: false })
    mocks.focusChangedHandler?.({ payload: true })
    await vi.advanceTimersByTimeAsync(160)

    expect(wrapper.attributes('data-hide-count')).toBe('0')
    wrapper.unmount()
  })
})
