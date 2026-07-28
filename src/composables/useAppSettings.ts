import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { replaceInstalledPetRoles, type InstalledPetRole } from '@/config/petRoles'
import {
  DEFAULT_APP_SETTINGS,
  type AppSettings,
} from '@/types/settings'
import { WINDOW_EVENTS } from '@/types/window'

export function useAppSettings() {
  const settings = ref<AppSettings>({ ...DEFAULT_APP_SETTINGS })
  const ready = ref(false)
  let unlisten: UnlistenFn | null = null

  async function refreshInstalledRoles() {
    const installedRoles = await invoke<InstalledPetRole[]>('list_installed_role_packs')
    replaceInstalledPetRoles(installedRoles)
  }

  async function loadSettings() {
    const [loadedSettings] = await Promise.all([
      invoke<AppSettings>('load_app_settings'),
      refreshInstalledRoles(),
    ])
    settings.value = loadedSettings
    ready.value = true
    return settings.value
  }

  async function applySettingsUpdate(updated: AppSettings) {
    if (updated.pet_role !== settings.value.pet_role) {
      try {
        await refreshInstalledRoles()
      } catch (error) {
        console.error('同步自定义角色失败:', error)
      }
    }
    settings.value = updated
    ready.value = true
  }

  onMounted(async () => {
    unlisten = await listen<AppSettings>(WINDOW_EVENTS.settingsUpdated, (event) => {
      void applySettingsUpdate(event.payload)
    })
  })

  onUnmounted(() => unlisten?.())

  return { settings, ready, loadSettings }
}
