import type { PetMood, SystemInfo } from '@/types/system'

export type WindowRole = 'pet' | 'chat' | 'info' | 'settings'

export interface PetContext {
  info: SystemInfo | null
  mood: PetMood
  scale: number
}

export const WINDOW_EVENTS = {
  petContext: 'pet://context',
  settingsUpdated: 'pet://settings-updated',
  setScale: 'pet://set-scale',
  focusChatInput: 'chat://focus-input',
} as const
