import type { PetRoleId } from '@/types/pet'
import type { PetMood, SystemInfo } from '@/types/system'

export type WindowRole = 'pet' | 'chat' | 'info' | 'settings' | 'reminder'

export interface PetContext {
  info: SystemInfo | null
  mood: PetMood
  roleId: PetRoleId
  scale: number
}

export interface ReminderPayload {
  reminder_id: string
  message: string
  snooze_minutes: number
}

export type OverlaySide = 'above' | 'below' | 'left' | 'right'

export const WINDOW_EVENTS = {
  petContext: 'pet://context',
  settingsUpdated: 'pet://settings-updated',
  setScale: 'pet://set-scale',
  focusChatInput: 'chat://focus-input',
  reminderPayload: 'pet://reminder-payload',
  overlayPresent: 'overlay://present',
} as const
