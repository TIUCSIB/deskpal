import type { PetMood, SystemInfo } from '@/types/system'

export type WindowRole = 'pet' | 'chat' | 'info'

export interface PetContext {
  info: SystemInfo | null
  mood: PetMood
}

export const WINDOW_EVENTS = {
  petContext: 'pet://context',
  setScale: 'pet://set-scale',
  focusChatInput: 'chat://focus-input',
} as const
