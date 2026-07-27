/** settings.ts - 应用设置类型 */
import { DEFAULT_PET_ROLE } from '@/config/petRoles'
import type { PetRoleId } from '@/types/pet'

export type InfoMode = 'auto' | 'always' | 'hidden'

export type ReminderSchedule =
  | { type: 'interval', interval_minutes: number }
  | { type: 'fixed_time', time: string }

export interface Reminder {
  id: string
  enabled: boolean
  message: string
  schedule: ReminderSchedule
  snooze_minutes: number
  paused_until: string | null
}

export interface ReminderInput {
  id?: string
  message: string
  schedule: ReminderSchedule
  snooze_minutes: number
}

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
  pet_role: PetRoleId
  info_mode: InfoMode
  size_locked: boolean
  shortcut_enabled: boolean
  launch_at_startup: boolean
  main_window_always_on_top: boolean
  main_window_show_in_taskbar: boolean
  chat_shortcut: string
  reminders: Reminder[]
}

export const DEFAULT_PET_SCALE = 0.85
export const DEFAULT_CHAT_SHORTCUT = 'Ctrl+Alt+D'
export const DEFAULT_SETTINGS_WINDOW_WIDTH = 500
export const DEFAULT_SETTINGS_WINDOW_HEIGHT = 560
export const MIN_SETTINGS_WINDOW_WIDTH = 460
export const MIN_SETTINGS_WINDOW_HEIGHT = 520
export const DEFAULT_REMINDER_MESSAGE = '记得喝水，起来活动一下吧'
export const DEFAULT_REMINDER_INTERVAL_MINUTES = 30
export const DEFAULT_REMINDER_SNOOZE_MINUTES = 5

export const DEFAULT_APP_SETTINGS: AppSettings = {
  main_position: null,
  settings_window_bounds: null,
  pet_scale: DEFAULT_PET_SCALE,
  pet_role: DEFAULT_PET_ROLE,
  info_mode: 'auto',
  size_locked: false,
  shortcut_enabled: true,
  launch_at_startup: false,
  main_window_always_on_top: true,
  main_window_show_in_taskbar: false,
  chat_shortcut: DEFAULT_CHAT_SHORTCUT,
  reminders: [],
}
