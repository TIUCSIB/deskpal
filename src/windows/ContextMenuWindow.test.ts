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

function roleButtons(wrapper: ReturnType<typeof mount>) {
  return wrapper.findAll<HTMLButtonElement>('[role="radio"]')
}

describe('ContextMenuWindow', () => {
  const scrollIntoView = vi.fn()

  beforeEach(() => {
    HTMLElement.prototype.scrollIntoView = scrollIntoView
    replaceInstalledPetRoles([TINY_CRT])
    mocks.settings.value = { ...DEFAULT_APP_SETTINGS, pet_role: 'tiny-crt', reminders: REMINDERS }
    mocks.ready.value = true
    mocks.invoke.mockReset()
    mocks.invoke.mockResolvedValue(undefined)
    mocks.listen.mockClear()
    mocks.onFocusChanged.mockClear()
    mocks.focusChangedHandler = null
    mocks.loadSettings.mockClear()
    scrollIntoView.mockClear()
    vi.useRealTimers()
  })

  afterEach(() => {
    replaceInstalledPetRoles([])
    document.body.replaceChildren()
    vi.clearAllMocks()
  })
  it('keeps the root menu compact and shows the selected role summary', () => {
    const wrapper = mount(ContextMenuWindow)

    expect(wrapper.text()).toContain('切换角色')
    expect(wrapper.text()).toContain('Tiny CRT')
    expect(roleButtons(wrapper)).toHaveLength(0)
    wrapper.unmount()
  })
  it('scrolls and focuses the selected role after opening the role view', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '切换角色').trigger('click')

    const selectedRole = wrapper.get('[aria-checked="true"]')
    expect(roleButtons(wrapper)).toHaveLength(4)
    expect(selectedRole.text()).toContain('Tiny CRT')
    expect(scrollIntoView).toHaveBeenCalledWith({ block: 'nearest' })
    expect(document.activeElement).toBe(selectedRole.element)
    wrapper.unmount()
  })
  it('filters roles by display name and shows an empty state', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '切换角色').trigger('click')

    const search = wrapper.get<HTMLInputElement>('input[aria-label="搜索角色"]')
    await search.setValue('tiny')
    expect(roleButtons(wrapper)).toHaveLength(1)
    expect(roleButtons(wrapper)[0]!.text()).toContain('Tiny CRT')

    await search.setValue('不存在')
    expect(roleButtons(wrapper)).toHaveLength(0)
    expect(wrapper.text()).toContain('未找到匹配的角色')
    wrapper.unmount()
  })
  it('supports root and role-list keyboard navigation', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    const chat = findButton(wrapper, '打开聊天')
    await chat.trigger('keydown', { key: 'End' })
    expect(document.activeElement).toBe(findButton(wrapper, '退出').element)

    const roleTrigger = findButton(wrapper, '切换角色')
    await roleTrigger.trigger('keydown', { key: 'ArrowRight' })
    const roles = roleButtons(wrapper)
    expect(document.activeElement).toBe(wrapper.get('[aria-checked="true"]').element)

    const lastRole = roles[roles.length - 1]!
    await roles[0]!.trigger('keydown', { key: 'End' })
    expect(document.activeElement).toBe(lastRole.element)
    await lastRole.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(roles[0]!.element)
    await roles[0]!.trigger('keydown', { key: 'Home' })
    expect(document.activeElement).toBe(roles[0]!.element)
    wrapper.unmount()
  })
  it('moves from search to the role list and clears search before returning', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '切换角色').trigger('click')

    const search = wrapper.get<HTMLInputElement>('input[aria-label="搜索角色"]')
    await search.setValue('tiny')
    await search.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(roleButtons(wrapper)[0]!.element)

    await roleButtons(wrapper)[0]!.trigger('keydown', { key: 'Escape' })
    expect(search.element.value).toBe('')
    expect(roleButtons(wrapper)).toHaveLength(4)
    await roleButtons(wrapper)[0]!.trigger('keydown', { key: 'Escape' })
    expect(roleButtons(wrapper)).toHaveLength(0)
    wrapper.unmount()
  })
  it('returns to the root menu with the mouse back button', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '切换角色').trigger('click')
    await wrapper.findComponent({ name: 'ContextMenuRolePicker' }).trigger('mouseup', { button: 3 })

    expect(roleButtons(wrapper)).toHaveLength(0)
    expect(document.activeElement).toBe(findButton(wrapper, '切换角色').element)
    wrapper.unmount()
  })
  it('switches roles through the validated native command', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '切换角色').trigger('click')
    await wrapper.get('[aria-checked="false"]').trigger('click')

    expect(mocks.invoke).toHaveBeenCalledWith('set_pet_role', { role: 'guga' })
    expect(mocks.invoke).toHaveBeenCalledWith('hide_main_context_menu')
    wrapper.unmount()
  })
  it('confirms before exiting the application', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '退出').trigger('click')

    expect(document.body.textContent).toContain('退出桌宠？')
    expect(mocks.invoke).not.toHaveBeenCalledWith('exit_application')

    const cancel = document.querySelector<HTMLElement>('[data-slot="alert-dialog-cancel"]')
    cancel?.click()
    await flushPromises()
    expect(mocks.invoke).not.toHaveBeenCalledWith('exit_application')

    await findButton(wrapper, '退出').trigger('click')
    const confirm = document.querySelector<HTMLElement>('[data-slot="alert-dialog-action"]')
    confirm?.click()
    await flushPromises()
    expect(mocks.invoke).toHaveBeenCalledWith('exit_application')
    wrapper.unmount()
  })
  it('manages enabled reminders without showing disabled reminders', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')

    expect(wrapper.text()).toContain('全部暂停到明天')
    expect(wrapper.text()).toContain('起来接水')
    expect(wrapper.text()).toContain('伸展一下')
    expect(wrapper.text()).toContain('已暂停')
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

  it('pauses one enabled reminder with fixed feedback', async () => {
    const wrapper = mount(ContextMenuWindow)
    await findButton(wrapper, '管理提醒').trigger('click')
    await findButton(wrapper, '起来接水').trigger('click')
    await flushPromises()

    expect(mocks.invoke.mock.calls).toEqual([
      ['pause_enabled_reminder_until_tomorrow', { reminderId: 'drink-water' }],
      ['show_reminder_paused_confirmation', { reminderId: 'drink-water' }],
      ['hide_main_context_menu'],
    ])
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
  it('returns from reminder management with the mouse back button', async () => {
    const wrapper = mount(ContextMenuWindow, { attachTo: document.body })
    await findButton(wrapper, '管理提醒').trigger('click')
    await wrapper.findComponent({ name: 'ContextMenuReminderPicker' }).trigger('mouseup', { button: 3 })

    expect(wrapper.text()).toContain('管理提醒')
    expect(document.activeElement).toBe(findButton(wrapper, '管理提醒').element)
    wrapper.unmount()
  })
})
