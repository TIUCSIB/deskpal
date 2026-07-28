import { defineComponent, h, nextTick } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useSettingsSectionFocus } from '@/composables/useSettingsSectionFocus'
import type { SettingsSection } from '@/types/window'

const mocks = vi.hoisted(() => ({
  focusHandler: null as ((event: { payload: SettingsSection }) => void) | null,
  listen: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))

const TestHost = defineComponent({
  setup() {
    const { activeSection, setSectionRef } = useSettingsSectionFocus()
    return () => h('div', [
      h('button', {
        ref: (element: unknown) => setSectionRef('display', element),
        'data-active': activeSection.value === 'display',
      }, '显示'),
      h('button', {
        ref: (element: unknown) => setSectionRef('reminder', element),
        'data-active': activeSection.value === 'reminder',
      }, '提醒'),
    ])
  },
})

describe('useSettingsSectionFocus', () => {
  beforeEach(() => {
    mocks.focusHandler = null
    mocks.listen.mockImplementation(async (_eventName: string, handler: (event: { payload: SettingsSection }) => void) => {
      mocks.focusHandler = handler
      return vi.fn()
    })
  })

  afterEach(() => {
    document.body.replaceChildren()
    vi.clearAllMocks()
  })

  it('opens and focuses the requested settings section', async () => {
    const wrapper = mount(TestHost, { attachTo: document.body })
    await flushPromises()

    mocks.focusHandler?.({ payload: 'reminder' })
    await nextTick()

    const reminder = wrapper.get('button:nth-child(2)')
    expect(reminder.attributes('data-active')).toBe('true')
    expect(document.activeElement).toBe(reminder.element)
    wrapper.unmount()
  })
})
