import { describe, expect, it } from 'vitest'
import {
  centerPosition,
  clampPosition,
  monitorWorkArea,
  normalizeWindowSize,
} from '@/composables/settingsWindowGeometry'

describe('settingsWindowGeometry', () => {
  it('uses a monitor work area so taskbars are excluded when restoring', () => {
    const area = monitorWorkArea({
      position: { x: 0, y: 0 },
      size: { width: 1920, height: 1080 },
      workArea: {
        position: { x: 0, y: 0 },
        size: { width: 1920, height: 1040 },
      },
    })

    expect(centerPosition(900, 700, area)).toEqual({ x: 510, y: 170 })
    expect(clampPosition(1400, 800, 900, 700, area)).toEqual({ x: 1020, y: 340 })
  })

  it('falls back to the full monitor rectangle when workArea is unavailable', () => {
    expect(monitorWorkArea({
      position: { x: -1600, y: 0 },
      size: { width: 1600, height: 900 },
    })).toEqual({ x: -1600, y: 0, width: 1600, height: 900 })
  })

  it('keeps negative monitor coordinates while clamping restored positions', () => {
    const area = { x: -1600, y: 40, width: 1600, height: 860 }

    expect(clampPosition(20, -100, 700, 500, area)).toEqual({ x: -700, y: 40 })
  })

  it('uses defaults for abnormal saved sizes', () => {
    expect(normalizeWindowSize(10, Number.NaN)).toMatchObject({ usedDefault: true })
  })
})
