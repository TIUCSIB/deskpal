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
    vi.advanceTimersByTime(900)
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
    vi.advanceTimersByTime(900)
    vi.advanceTimersByTime(60000)
    expect(behavior.animationName.value).toBe('Waving')

    behavior.setHovering(false)
    vi.advanceTimersByTime(15000)
    expect(behavior.animationName.value).toBe('Waiting')
    behavior.dispose()
  })

  it('uses a role-specific mood baseline without changing interaction priority', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior(() => 0)

    behavior.setRole('monthly-salary-cat', ['Idle', 'RunLeft', 'RunRight', 'Waving', 'Jumping', 'Failed', 'Waiting', 'Review'])
    expect(behavior.animationName.value).toBe('Waiting')

    behavior.setHovering(true)
    expect(behavior.animationName.value).toBe('Waiting')
    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('Waving')
    behavior.dispose()
  })

  it('waits before turning a raw hover into petting feedback', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.setMood('warning')
    behavior.setHovering(true)
    vi.advanceTimersByTime(899)
    expect(behavior.animationName.value).toBe('Failed')

    vi.advanceTimersByTime(1)
    expect(behavior.petting.value).toBe(true)
    expect(behavior.animationName.value).toBe('Waving')
    behavior.dispose()
  })

  it('cancels pending petting when the pointer leaves or a drag begins', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.setHovering(true)
    vi.advanceTimersByTime(400)
    behavior.setHovering(false)
    vi.advanceTimersByTime(600)
    expect(behavior.petting.value).toBe(false)
    expect(behavior.animationName.value).toBe('Idle')

    behavior.setHovering(true)
    behavior.setDragging('right')
    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('RunRight')
    expect(behavior.petting.value).toBe(false)
    behavior.dispose()
  })

  it('prioritizes click feedback over active petting', () => {
    vi.useFakeTimers()
    const behavior = usePetBehavior()

    behavior.setHovering(true)
    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('Waving')

    behavior.triggerClickFeedback()
    expect(behavior.animationName.value).toBe('Jumping')
    vi.advanceTimersByTime(900)
    expect(behavior.animationName.value).toBe('Waving')
    behavior.dispose()
  })

  it('falls back to Idle when a role does not provide a configured animation', () => {
    const behavior = usePetBehavior()

    behavior.setRole('broom-witch', ['Idle', 'RunLeft', 'RunRight'])
    behavior.setMood('happy')

    expect(behavior.animationName.value).toBe('Idle')
    behavior.dispose()
  })
})
