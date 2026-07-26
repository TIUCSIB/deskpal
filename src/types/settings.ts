/** settings.ts - 应用设置类型 */

export type InfoMode = 'auto' | 'always' | 'hidden'

export interface SavedPosition {
  x: number
  y: number
}

export interface SavedWindowBounds {
  x: number
  y: number
  width: number
  height: number
}

export interface AppSettings {
  main_position: SavedPosition | null
  settings_window_bounds: SavedWindowBounds | null
  pet_scale: number
  info_mode: InfoMode
  size_locked: boolean
  shortcut_enabled: boolean
  launch_at_startup: boolean
  main_window_always_on_top: boolean
  main_window_show_in_taskbar: boolean
  chat_shortcut: string
}

export const DEFAULT_PET_SCALE = 0.85
export const DEFAULT_CHAT_SHORTCUT = 'Ctrl+Alt+D'
export const DEFAULT_SETTINGS_WINDOW_WIDTH = 500
export const DEFAULT_SETTINGS_WINDOW_HEIGHT = 560
export const MIN_SETTINGS_WINDOW_WIDTH = 460
export const MIN_SETTINGS_WINDOW_HEIGHT = 520

export const DEFAULT_APP_SETTINGS: AppSettings = {
  main_position: null,
  settings_window_bounds: null,
  pet_scale: DEFAULT_PET_SCALE,
  info_mode: 'auto',
  size_locked: false,
  shortcut_enabled: true,
  launch_at_startup: false,
  main_window_always_on_top: true,
  main_window_show_in_taskbar: false,
  chat_shortcut: DEFAULT_CHAT_SHORTCUT,
}
