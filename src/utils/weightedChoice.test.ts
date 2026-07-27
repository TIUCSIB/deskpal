import { describe, expect, it } from 'vitest'
import { chooseWeighted } from '@/utils/weightedChoice'

describe('chooseWeighted', () => {
  const choices = [
    { value: 'idle', weight: 2 },
    { value: 'review', weight: 3 },
    { value: 'waiting', weight: 5 },
  ]

  it('selects values at deterministic weight boundaries', () => {
    expect(chooseWeighted(choices, () => 0)).toBe('idle')
    expect(chooseWeighted(choices, () => 0.2)).toBe('review')
    expect(chooseWeighted(choices, () => 0.9)).toBe('waiting')
  })

  it('ignores invalid weights and returns null without valid choices', () => {
    expect(chooseWeighted([{ value: 'ignored', weight: 0 }], () => 0)).toBeNull()
    expect(chooseWeighted([{ value: 'ignored', weight: Number.NaN }], () => 0)).toBeNull()
  })
})
