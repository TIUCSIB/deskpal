import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { replaceInstalledPetRoles, type InstalledPetRole } from '@/config/petRoles'
import {
  DEFAULT_APP_SETTINGS,
  type AppSettings,
} from '@/types/settings'
import { WINDOW_EVENTS } from '@/types/window'

const LOAD_SETTINGS_TIMEOUT_MS = 800
const LOAD_SETTINGS_RETRY_DELAYS = [0, 300, 900, 1800] as const

export function useAppSettings() {
  const settings = ref<AppSettings>({ ...DEFAULT_APP_SETTINGS })
  const ready = ref(false)
  let unlisten: UnlistenFn | null = null

  async function refreshInstalledRoles() {
    const installedRoles = await invoke<InstalledPetRole[]>('list_installed_role_packs')
    replaceInstalledPetRoles(installedRoles)
  }

  function handleInstalledRolesError(message: string, error: unknown) {
    console.error(message, error)
    replaceInstalledPetRoles([])
  }

  async function invokeSettingsWithTimeout() {
    return await Promise.race([
      invoke<AppSettings>('load_app_settings'),
      new Promise<AppSettings>((_, reject) => {
        setTimeout(() => {
          reject(new Error('读取设置超时'))
        }, LOAD_SETTINGS_TIMEOUT_MS)
      }),
    ])
  }

  async function loadSettings() {
    let lastError: unknown = null
    for (const delay of LOAD_SETTINGS_RETRY_DELAYS) {
      if (delay > 0) {
        await new Promise(resolve => setTimeout(resolve, delay))
      }
      try {
        const loadedSettings = await invokeSettingsWithTimeout()
        settings.value = loadedSettings
        ready.value = true
        void refreshInstalledRoles().catch((error: unknown) => {
          handleInstalledRolesError('读取自定义角色失败，已回退为内置角色列表:', error)
        })
        return settings.value
      } catch (error: unknown) {
        lastError = error
      }
    }
    throw lastError instanceof Error ? lastError : new Error('读取设置失败')
  }

  async function applySettingsUpdate(updated: AppSettings) {
    const roleChanged = updated.pet_role !== settings.value.pet_role
    settings.value = updated
    ready.value = true
    if (!roleChanged) return
    void refreshInstalledRoles().catch((error: unknown) => {
      handleInstalledRolesError('同步自定义角色失败:', error)
    })
  }

  onMounted(async () => {
    unlisten = await listen<AppSettings>(WINDOW_EVENTS.settingsUpdated, (event) => {
      void applySettingsUpdate(event.payload)
    })
  })

  onUnmounted(() => unlisten?.())

  return { settings, ready, loadSettings }
}
