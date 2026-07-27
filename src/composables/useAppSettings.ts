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

  async function loadSettings() {
    const [loadedSettings, installedRoles] = await Promise.all([
      invoke<AppSettings>('load_app_settings'),
      invoke<InstalledPetRole[]>('list_installed_role_packs'),
    ])
    replaceInstalledPetRoles(installedRoles)
    settings.value = loadedSettings
    ready.value = true
    return settings.value
  }

  onMounted(async () => {
    unlisten = await listen<AppSettings>(WINDOW_EVENTS.settingsUpdated, (event) => {
      settings.value = event.payload
      ready.value = true
    })
  })

  onUnmounted(() => unlisten?.())

  return { settings, ready, loadSettings }
}
