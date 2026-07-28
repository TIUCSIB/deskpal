import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  replaceInstalledPetRoles,
  type InstalledPetRole,
} from '@/config/petRoles'
import ContextMenuWindow from '@/windows/ContextMenuWindow.vue'
import { DEFAULT_APP_SETTINGS } from '@/types/settings'

const mocks = vi.hoisted(() => {
  const settings = { value: { pet_role: 'tiny-crt' } }
  const ready = { value: true }
  return {
    invoke: vi.fn(),
    listen: vi.fn(async () => vi.fn()),
    onFocusChanged: vi.fn(async () => vi.fn()),
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

describe('ContextMenuWindow', () => {
  beforeEach(() => {
    replaceInstalledPetRoles([TINY_CRT])
    mocks.settings.value = { ...DEFAULT_APP_SETTINGS, pet_role: 'tiny-crt' }
    mocks.ready.value = true
    mocks.invoke.mockReset()
    mocks.listen.mockClear()
    mocks.onFocusChanged.mockClear()
    mocks.loadSettings.mockClear()
  })

  afterEach(() => {
    replaceInstalledPetRoles([])
    vi.clearAllMocks()
  })

  it('renders installed roles and marks the current selection', async () => {
    const wrapper = mount(ContextMenuWindow)
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Tiny CRT')
    })

    expect(wrapper.get('[aria-pressed="true"]').text()).toContain('Tiny CRT')
    expect(wrapper.find('[aria-label="当前角色"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('switches roles through the validated native command', async () => {
    const wrapper = mount(ContextMenuWindow)
    await vi.waitFor(() => {
      expect(wrapper.findAll('.context-menu__role')).toHaveLength(4)
    })

    await wrapper.get('[aria-pressed="false"]').trigger('click')

    expect(mocks.invoke).toHaveBeenCalledWith('set_pet_role', { role: 'guga' })
    expect(mocks.invoke).toHaveBeenCalledWith('hide_main_context_menu')
    wrapper.unmount()
  })

  it('closes when Escape is pressed', async () => {
    const wrapper = mount(ContextMenuWindow)
    await wrapper.trigger('keydown', { key: 'Escape' })

    expect(mocks.invoke).toHaveBeenCalledWith('hide_main_context_menu')
    wrapper.unmount()
  })
})
