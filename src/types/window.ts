import type { PetMood, SystemInfo } from '@/types/system'

export type WindowRole = 'pet' | 'chat' | 'info' | 'settings' | 'reminder'

export interface PetContext {
  info: SystemInfo | null
  mood: PetMood
  scale: number
}

export interface ReminderPayload {
  message: string
  snooze_minutes: number
}

export const WINDOW_EVENTS = {
  petContext: 'pet://context',
  settingsUpdated: 'pet://settings-updated',
  setScale: 'pet://set-scale',
  focusChatInput: 'chat://focus-input',
  reminderPayload: 'pet://reminder-payload',
} as const
