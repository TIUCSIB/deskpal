import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePetInteractionState } from '@/composables/usePetInteractionState'

describe('usePetInteractionState', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('keeps feedback session-only and clears its short text after the timeout', () => {
    const state = usePetInteractionState(() => Date.now())

    state.record('click', '收到！')
    expect(state.interactionText.value).toBe('收到！')
    expect(state.interactionLevel.value).toBe(1)

    vi.advanceTimersByTime(3000)
    state.record('pet', '摸摸收到～')
    expect(state.interactionText.value).toBe('摸摸收到～')
    expect(state.interactionLevel.value).toBe(3)

    vi.advanceTimersByTime(3199)
    expect(state.interactionText.value).toBe('摸摸收到～')
    vi.advanceTimersByTime(1)
    expect(state.interactionText.value).toBeNull()
    state.dispose()
  })

  it('caps accumulated interaction level', () => {
    const state = usePetInteractionState()

    for (let index = 0; index < 5; index += 1) state.record('pet', '摸摸收到～')

    expect(state.interactionLevel.value).toBe(8)
    state.dispose()
  })
})
