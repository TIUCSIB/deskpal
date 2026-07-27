import type { PetRoleId } from '@/types/pet'
import type { PetMood, SystemInfo } from '@/types/system'

export type WindowRole = 'pet' | 'chat' | 'info' | 'settings' | 'reminder' | 'feedback'

export interface PetContext {
  info: SystemInfo | null
  mood: PetMood
  roleId: PetRoleId
  scale: number
  interactionText: string | null
  interactionLevel: number
}

export type PetContextRecipient = 'chat' | 'info' | 'reminder' | 'feedback'

export interface PetContextRequest {
  recipient: PetContextRecipient
}

export interface ReminderPayload {
  reminder_id: string
  message: string
  snooze_minutes: number
}

export type OverlaySide = 'above' | 'below' | 'left' | 'right'

export const WINDOW_EVENTS = {
  petContext: 'pet://context',
  petContextRequest: 'pet://context-request',
  settingsUpdated: 'pet://settings-updated',
  setScale: 'pet://set-scale',
  focusChatInput: 'chat://focus-input',
  reminderPayload: 'pet://reminder-payload',
  systemFeedbackPayload: 'pet://system-feedback-payload',
  overlayPresent: 'overlay://present',
} as const
