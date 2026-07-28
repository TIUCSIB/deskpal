import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getPetRole,
  replaceInstalledPetRoles,
  type InstalledPetRole,
} from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import { DEFAULT_APP_SETTINGS, type AppSettings } from '@/types/settings'
import { WINDOW_EVENTS } from '@/types/window'

type EventHandler<T> = (event: { payload: T }) => void

const mocks = vi.hoisted(() => {
  const handlers = new Map<string, EventHandler<unknown>>()
  const unlisten = vi.fn()
  const invoke = vi.fn()
  const listen = vi.fn(async (eventName: string, handler: EventHandler<unknown>) => {
    handlers.set(eventName, handler)
    return unlisten
  })

  return { handlers, invoke, listen, unlisten }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mocks.invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))

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

let appSettings: ReturnType<typeof useAppSettings> | null = null

const Host = defineComponent({
  setup() {
    appSettings = useAppSettings()
    return () => h('div')
  },
})

async function flushListeners() {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

function updatedSettings(overrides: Partial<AppSettings>): AppSettings {
  return { ...DEFAULT_APP_SETTINGS, ...overrides }
}

describe('useAppSettings', () => {
  beforeEach(() => {
    appSettings = null
    replaceInstalledPetRoles([])
    mocks.handlers.clear()
    mocks.invoke.mockReset()
    mocks.listen.mockClear()
    mocks.unlisten.mockReset()
    mocks.listen.mockImplementation(async (eventName: string, handler: EventHandler<unknown>) => {
      mocks.handlers.set(eventName, handler)
      return mocks.unlisten
    })
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('refreshes validated installed roles before applying a changed role ID', async () => {
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === 'list_installed_role_packs') return [TINY_CRT]
      return { ...DEFAULT_APP_SETTINGS }
    })
    const wrapper = mount(Host)
    await flushListeners()

    expect(getPetRole('tiny-crt').id).toBe('guga')

    const handler = mocks.handlers.get(WINDOW_EVENTS.settingsUpdated)
    handler?.({ payload: updatedSettings({ pet_role: 'tiny-crt' }) })
    await flushListeners()

    expect(mocks.invoke).toHaveBeenCalledWith('list_installed_role_packs')
    expect(appSettings?.settings.value.pet_role).toBe('tiny-crt')
    expect(getPetRole('tiny-crt')).toMatchObject({
      id: 'tiny-crt',
      spritesheetUrl: 'role-pack://tiny-crt/spritesheet.webp',
    })
    wrapper.unmount()
  })

  it('does not refresh installed roles for unrelated settings updates', async () => {
    const wrapper = mount(Host)
    await flushListeners()

    const handler = mocks.handlers.get(WINDOW_EVENTS.settingsUpdated)
    handler?.({ payload: updatedSettings({ info_mode: 'always' }) })
    await flushListeners()

    expect(mocks.invoke).not.toHaveBeenCalledWith('list_installed_role_packs')
    expect(appSettings?.settings.value.info_mode).toBe('always')
    wrapper.unmount()
  })
})
