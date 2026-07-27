import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi'
import { getCurrentWindow, currentMonitor, primaryMonitor } from '@tauri-apps/api/window'
import { useAppSettings } from '@/composables/useAppSettings'
import { useReminderSettings } from '@/composables/useReminderSettings'
import { useRoleSettings } from '@/composables/useRoleSettings'
import {
  centerPosition,
  clampPosition,
  normalizeWindowSize,
  monitorWorkArea,
  type WindowRect,
} from '@/composables/settingsWindowGeometry'
import {
  DEFAULT_CHAT_SHORTCUT,
  DEFAULT_PET_SCALE,
  type AppSettings,
  type InfoMode,
} from '@/types/settings'

const currentWindow = getCurrentWindow()
const SAVE_BOUNDS_DELAY = 120
const RESTORE_BOUNDS_LOCK_DELAY = 180

type Unlisten = (() => void) | null
export type ReminderSettings = ReturnType<typeof useReminderSettings>

/** useSettingsWindow - 设置窗口状态与交互 */
export function useSettingsWindow() {
  const { settings, ready, loadSettings } = useAppSettings()
  const shortcutDraft = ref(DEFAULT_CHAT_SHORTCUT)
  const infoModeOptions: Array<{ label: string; value: InfoMode }> = [
    { label: '自动显示', value: 'auto' },
    { label: '始终显示', value: 'always' },
    { label: '隐藏', value: 'hidden' },
  ]
  let unlistenClose: Unlisten = null
  let unlistenMoved: Unlisten = null
  let unlistenResized: Unlisten = null
  let restoreBoundsPending = false
  let saveBoundsTimer: ReturnType<typeof setTimeout> | null = null

  const scaleText = computed(() => settings.value.pet_scale.toFixed(2))
  const shortcutSummary = computed(() => {
    if (!settings.value.shortcut_enabled) return '当前已关闭，保存后可随时重新启用'
    return `当前快捷键：${settings.value.chat_shortcut}`
  })

  watch(() => settings.value.chat_shortcut, (shortcut) => {
    shortcutDraft.value = shortcut
  }, { immediate: true })

  function setFeedback(text: string) {
    if (text) toast(text)
  }

  function clearSaveBoundsTimer() {
    if (!saveBoundsTimer) return
    clearTimeout(saveBoundsTimer)
    saveBoundsTimer = null
  }

  function scheduleSaveBounds() {
    if (restoreBoundsPending) return
    clearSaveBoundsTimer()
    saveBoundsTimer = setTimeout(() => {
      saveBoundsTimer = null
      void saveWindowBounds()
    }, SAVE_BOUNDS_DELAY)
  }

  async function withBoundsRestoreLock<T>(task: () => Promise<T>) {
    restoreBoundsPending = true
    clearSaveBoundsTimer()
    try {
      return await task()
    } finally {
      setTimeout(() => {
        restoreBoundsPending = false
      }, RESTORE_BOUNDS_LOCK_DELAY)
    }
  }

  async function closeWindow() {
    await invoke('hide_settings_window')
  }

  async function saveWindowBounds() {
    if (restoreBoundsPending) return
    const position = await currentWindow.innerPosition()
    const size = await currentWindow.innerSize()
    settings.value = await invoke<AppSettings>('save_settings_window_bounds', {
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
    })
  }

  async function resolveMonitorRect(): Promise<WindowRect | null> {
    const monitor = (await currentMonitor()) ?? (await primaryMonitor())
    if (!monitor) return null
    return monitorWorkArea(monitor)
  }

  async function restoreWindowBounds() {
    const bounds = settings.value.settings_window_bounds
    if (!bounds) return
    const size = normalizeWindowSize(bounds.width, bounds.height)
    const monitorRect = await resolveMonitorRect()
    const nextPosition = monitorRect
      ? size.usedDefault
        ? centerPosition(size.width, size.height, monitorRect)
        : clampPosition(bounds.x, bounds.y, size.width, size.height, monitorRect)
      : { x: bounds.x, y: bounds.y }
    const recovered = size.usedDefault || nextPosition.x !== bounds.x || nextPosition.y !== bounds.y
    await withBoundsRestoreLock(async () => {
      await currentWindow.setSize(new PhysicalSize(size.width, size.height))
      await currentWindow.setPosition(new PhysicalPosition(nextPosition.x, nextPosition.y))
    })
    if (recovered) setFeedback('检测到旧的设置窗口位置或大小异常，已自动恢复到可见范围')
  }

  async function invokeSetting(command: string, payload?: Record<string, unknown>) {
    try {
      const updated = await invoke<AppSettings>(command, payload)
      settings.value = updated
      return updated
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '设置保存失败')
      throw error
    }
  }

  const reminderSettings = useReminderSettings(settings, invokeSetting, setFeedback)
  const roleSettings = useRoleSettings(settings, invokeSetting, setFeedback)

  async function handleInfoModeChange(mode: InfoMode) {
    const updated = await invokeSetting('set_info_mode', { mode })
    setFeedback(`信息窗已切换为${updated.info_mode === 'always' ? '始终显示' : updated.info_mode === 'hidden' ? '隐藏' : '自动显示'}`)
  }

  async function handleScaleChange(scale: number) {
    await invokeSetting('save_pet_scale', { scale })
    setFeedback(`宠物大小已调整为 ${scale.toFixed(2)}`)
  }

  async function handleSizeLockedChange(locked: boolean) {
    await invokeSetting('set_size_locked', { locked })
    setFeedback(locked ? '已锁定宠物大小' : '已允许滚轮调整大小')
  }

  async function handleShortcutEnabledChange(enabled: boolean) {
    const updated = await invokeSetting('set_shortcut_enabled', { enabled })
    if (enabled && !updated.shortcut_enabled) return setFeedback('快捷键被占用或注册失败，已自动关闭')
    setFeedback(enabled ? '聊天快捷键已开启' : '聊天快捷键已关闭')
  }

  async function handleAlwaysOnTopChange(enabled: boolean) {
    await invokeSetting('set_main_window_always_on_top', { enabled })
    setFeedback(enabled ? '桌宠窗口已置顶' : '桌宠窗口已取消置顶')
  }

  async function handleTaskbarChange(enabled: boolean) {
    await invokeSetting('set_main_window_show_in_taskbar', { enabled })
    setFeedback(enabled ? '桌宠已显示在任务栏' : '桌宠已从任务栏隐藏')
  }

  async function handleLaunchAtStartupChange(enabled: boolean) {
    await invokeSetting('set_launch_at_startup', { enabled })
    setFeedback(enabled ? '已开启开机自动启动' : '已关闭开机自动启动')
  }

  async function applyShortcut() {
    const shortcut = shortcutDraft.value.trim()
    if (!shortcut) return setFeedback('请输入快捷键')
    const updated = await invokeSetting('set_chat_shortcut', { shortcut })
    if (updated.chat_shortcut !== shortcut) return setFeedback('快捷键未能应用，已恢复到上一个可用值')
    setFeedback(updated.shortcut_enabled ? '快捷键已更新并立即生效' : '快捷键已保存，启用后生效')
  }

  function handleShortcutDraftInput(value: string) {
    shortcutDraft.value = value
  }

  async function restoreDefaultScale() {
    await invokeSetting('save_pet_scale', { scale: DEFAULT_PET_SCALE })
    setFeedback('已恢复默认大小')
  }

  async function resetPosition() {
    await invokeSetting('reset_main_window_position')
    setFeedback('桌宠位置已重置')
  }

  async function resetSettingsWindowBounds() {
    await withBoundsRestoreLock(() => invokeSetting('reset_settings_window_bounds'))
    setFeedback('设置窗口已恢复默认位置和大小')
  }

  async function resetAllSettings() {
    await withBoundsRestoreLock(() => invokeSetting('reset_all_settings'))
    setFeedback('已恢复全部默认设置')
  }

  async function exportPortableSettings() {
    try {
      const exported = await invoke<boolean>('export_portable_settings')
      if (exported) setFeedback('设置已导出')
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '设置导出失败')
    }
  }

  async function importPortableSettings() {
    try {
      const imported = await invoke<boolean>('import_portable_settings')
      if (!imported) return
      settings.value = await invoke<AppSettings>('load_app_settings')
      setFeedback('设置已导入；设备相关配置已保留')
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '设置导入失败')
    }
  }

  async function completeOnboarding() {
    try {
      settings.value = await invoke<AppSettings>('complete_settings_onboarding')
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '保存欢迎引导状态失败')
    }
  }

  onMounted(async () => {
    await loadSettings()
    await restoreWindowBounds()
    unlistenClose = await currentWindow.onCloseRequested((event) => {
      event.preventDefault()
      void closeWindow()
    })
    unlistenMoved = await currentWindow.onMoved(scheduleSaveBounds)
    unlistenResized = await currentWindow.onResized(scheduleSaveBounds)
  })

  onUnmounted(() => {
    clearSaveBoundsTimer()
    unlistenClose?.()
    unlistenMoved?.()
    unlistenResized?.()
  })

  return {
    settings,
    ready,
    scaleText,
    shortcutDraft,
    shortcutSummary,
    infoModeOptions,
    closeWindow,
    handleInfoModeChange,
    handleScaleChange,
    handleSizeLockedChange,
    handleShortcutEnabledChange,
    handleAlwaysOnTopChange,
    handleTaskbarChange,
    handleLaunchAtStartupChange,
    handleShortcutDraftInput,
    applyShortcut,
    restoreDefaultScale,
    resetPosition,
    resetSettingsWindowBounds,
    resetAllSettings,
    exportPortableSettings,
    importPortableSettings,
    completeOnboarding,
    reminderSettings,
    ...roleSettings,
  }
}
