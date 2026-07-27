import { afterEach, describe, expect, it, vi } from 'vitest'
import { usePetBehavior } from '@/composables/usePetBehavior'

describe('usePetBehavior', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('applies dragging, activation, hover, and mood in priority order', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.setMood('warning')
    expect(behavior.animationName.value).toBe('Failed')

    behavior.setHovering(true)
    expect(behavior.animationName.value).toBe('Waving')

    behavior.activate()
    expect(behavior.animationName.value).toBe('Jumping')

    behavior.setDragging('left')
    expect(behavior.animationName.value).toBe('RunLeft')

    behavior.setDragging(null)
    expect(behavior.animationName.value).toBe('Jumping')

    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('Waving')
    behavior.dispose()
  })

  it('returns to the current mood when activation ends', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.activate()
    behavior.setMood('warning')
    expect(behavior.animationName.value).toBe('Jumping')

    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('Failed')
    behavior.dispose()
  })

  it('invalidates an older activation timer when activated again', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.activate()
    vi.advanceTimersByTime(500)
    behavior.activate()
    vi.advanceTimersByTime(400)
    expect(behavior.animationName.value).toBe('Jumping')

    vi.advanceTimersByTime(500)
    expect(behavior.animationName.value).toBe('Idle')
    behavior.dispose()
  })

  it('runs ambient animation only while no higher-priority behavior is active', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.start()
    vi.advanceTimersByTime(15000)
    expect(behavior.animationName.value).toBe('Waiting')

    behavior.setHovering(true)
    vi.advanceTimersByTime(60000)
    expect(behavior.animationName.value).toBe('Waving')

    behavior.setHovering(false)
    vi.advanceTimersByTime(15000)
    expect(behavior.animationName.value).toBe('Waiting')
    behavior.dispose()
  })
})
