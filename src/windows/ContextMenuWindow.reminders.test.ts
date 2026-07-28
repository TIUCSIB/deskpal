import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  replaceInstalledPetRoles,
  type InstalledPetRole,
} from '@/config/petRoles'
import { DEFAULT_APP_SETTINGS, type AppSettings, type Reminder } from '@/types/settings'
import ContextMenuWindow from '@/windows/ContextMenuWindow.vue'

const mocks = vi.hoisted(() => {
  const settings: { value: AppSettings } = { value: { pet_role: 'tiny-crt' } as AppSettings }
  const ready = { value: true }
  return {
    invoke: vi.fn(),
    listen: vi.fn(async () => vi.fn()),
    focusChangedHandler: null as ((event: { payload: boolean }) => void) | null,
    onFocusChanged: vi.fn(async (handler: (event: { payload: boolean }) => void) => {
      mocks.focusChangedHandler = handler
      return vi.fn()
    }),
    loadSettings: vi.fn(async () => ({ pet_role: 'tiny-crt' })),
    settings,
    ready,
  }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mocks.invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onFocusChanged: mocks.onFocusChanged }),
}))
vi.mock('@/composables/useAppSettings', () => ({
  useAppSettings: () => ({
    settings: mocks.settings,
    ready: mocks.ready,
    loadSettings: mocks.loadSettings,
  }),
}))

const REMINDERS: Reminder[] = [
  {
    id: 'drink-water',
    enabled: true,
    message: '起来接水',
    schedule: { type: 'interval', interval_minutes: 30 },
    snooze_minutes: 10,
    paused_until: null,
  },
  {
    id: 'stretch',
    enabled: true,
    message: '伸展一下',
    schedule: { type: 'interval', interval_minutes: 60 },
    snooze_minutes: 10,
    paused_until: '2099-01-01T00:00:00+08:00',
  },
  {
    id: 'disabled-reminder',
    enabled: false,
    message: '不应显示',
    schedule: { type: 'interval', interval_minutes: 30 },
    snooze_minutes: 10,
    paused_until: null,
  },
]

const TINY_CRT: InstalledPetRole = {
  id: 'tiny-crt',
  displayName: 'Tiny CRT',
  description: '一台小巧的复古终端显示器。',
  kind: 'object',
  spritesheetUrl: 'role-pack://tiny-crt/spritesheet.webp',
  spritesheet: {
    width: 1536,
    height: 1872,
    frameWidth: 192,
    frameHeight: 208,
    rowGap: 0,
    animations: [{ name: 'Idle', row: 0, frames: 6, fps: 4 }],
  },
}

function findButton(wrapper: ReturnType<typeof mount>, text: string) {
  const button = wrapper.findAll('button').find(item => item.text().includes(text))
  if (!button) throw new Error(`找不到按钮：${text}`)
  return button
}

describe('ContextMenuWindow reminder management', () => {
  beforeEach(() => {
    replaceInstalledPetRoles([TINY_CRT])
    mocks.settings.value = { ...DEFAULT_APP_SETTINGS, pet_role: 'tiny-crt', reminders: REMINDERS }
    mocks.ready.value = true
    mocks.invoke.mockReset()
    mocks.invoke.mockResolvedValue(undefined)
    mocks.listen.mockClear()
    mocks.onFocusChanged.mockClear()
    mocks.focusChangedHandler = null
    mocks.loadSettings.mockClear()
    vi.useRealTimers()
  })

  afterEach(() => {
    replaceInstalledPetRoles([])
    document.body.replaceChildren()
    vi.clearAllMocks()
  })

  it('manages enabled reminders without showing disabled reminders', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')

    expect(wrapper.text()).toContain('全部暂停到明天')
    expect(wrapper.text()).toContain('起来接水')
    expect(wrapper.text()).toContain('伸展一下')
    expect(wrapper.text()).toContain('恢复提醒')
    expect(wrapper.text()).not.toContain('不应显示')
    wrapper.unmount()
  })

  it('opens the reminder settings section from reminder management', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '打开提醒设置').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['show_main_reminder_settings'],
      ['hide_main_context_menu'],
    ])
    wrapper.unmount()
  })

  it('pauses one enabled reminder with fixed feedback and keeps reminder management open', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '起来接水').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['pause_enabled_reminder_until_tomorrow', { reminderId: 'drink-water' }],
      ['show_reminder_paused_confirmation', { reminderId: 'drink-water' }],
    ])
    expect(wrapper.text()).toContain('打开提醒设置')
    wrapper.unmount()
  })

  it('resumes a paused reminder from reminder management without closing the menu', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '恢复提醒').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['resume_reminder', { id: 'stretch' }],
    ])
    expect(wrapper.text()).toContain('打开提醒设置')
    wrapper.unmount()
  })

  it('pauses all enabled reminders after explicit selection', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '全部暂停到明天').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['pause_all_reminders_until_tomorrow'],
      ['show_reminders_paused_confirmation'],
      ['hide_main_context_menu'],
    ])
    wrapper.unmount()
  })

  it('keeps the reminder menu open when an individual pause fails', async () => {
    mocks.invoke.mockRejectedValueOnce(new Error('暂停失败'))
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '起来接水').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['pause_enabled_reminder_until_tomorrow', { reminderId: 'drink-water' }],
    ])
    expect(wrapper.text()).toContain('起来接水')
    errorSpy.mockRestore()
    wrapper.unmount()
  })

  it('keeps the reminder menu open when resuming a reminder fails', async () => {
    mocks.invoke.mockRejectedValueOnce(new Error('恢复失败'))
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '恢复提醒').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['resume_reminder', { id: 'stretch' }],
    ])
    expect(wrapper.text()).toContain('恢复提醒')
    errorSpy.mockRestore()
    wrapper.unmount()
  })

  it('returns from reminder management with the mouse back button', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '管理提醒').trigger('click')
    await wrapper.findComponent({ name: 'ContextMenuReminderPicker' }).trigger('mouseup', { button: 3 })

    expect(wrapper.text()).toContain('管理提醒')
    expect(document.activeElement).toBe(findButton(wrapper, '管理提醒').element)
    wrapper.unmount()
  })
})
