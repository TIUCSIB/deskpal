import { computed, ref } from 'vue'
import type { PetInteractionKind } from '@/config/petPersonalities'

const INTERACTION_TEXT_DURATION_MS = 3200
const MAX_INTERACTION_LEVEL = 8

/** usePetInteractionState - 当前运行会话中的轻量互动状态 */
export function usePetInteractionState(now: () => number = Date.now) {
  const interactionText = ref<string | null>(null)
  const interactionLevel = ref(0)
  const lastInteractionAt = ref(0)
  let clearTextTimer: ReturnType<typeof setTimeout> | null = null

  const isRecentlyInteracted = computed(() => now() - lastInteractionAt.value < INTERACTION_TEXT_DURATION_MS)

  function record(kind: PetInteractionKind, text: string) {
    lastInteractionAt.value = now()
    interactionLevel.value = Math.min(MAX_INTERACTION_LEVEL, interactionLevel.value + (kind === 'pet' ? 2 : 1))
    interactionText.value = text
    if (clearTextTimer) clearTimeout(clearTextTimer)
    clearTextTimer = setTimeout(() => {
      interactionText.value = null
      clearTextTimer = null
    }, INTERACTION_TEXT_DURATION_MS)
  }

  function dispose() {
    if (clearTextTimer) clearTimeout(clearTextTimer)
  }

  return {
    interactionText,
    interactionLevel,
    isRecentlyInteracted,
    record,
    dispose,
  }
}
