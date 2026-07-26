import {
  DEFAULT_SETTINGS_WINDOW_HEIGHT,
  DEFAULT_SETTINGS_WINDOW_WIDTH,
  MIN_SETTINGS_WINDOW_HEIGHT,
  MIN_SETTINGS_WINDOW_WIDTH,
} from '@/types/settings'

export interface WindowRect {
  x: number
  y: number
  width: number
  height: number
}

export interface NormalizedWindowSize {
  width: number
  height: number
  usedDefault: boolean
}

export function normalizeWindowSize(width: number, height: number): NormalizedWindowSize {
  if (!Number.isFinite(width) || !Number.isFinite(height) || isAbnormalWindowSize(width, height)) {
    return {
      width: DEFAULT_SETTINGS_WINDOW_WIDTH,
      height: DEFAULT_SETTINGS_WINDOW_HEIGHT,
      usedDefault: true,
    }
  }

  return {
    width: Math.max(Math.round(width), MIN_SETTINGS_WINDOW_WIDTH),
    height: Math.max(Math.round(height), MIN_SETTINGS_WINDOW_HEIGHT),
    usedDefault: false,
  }
}

export function centerPosition(width: number, height: number, rect: WindowRect) {
  return {
    x: rect.x + Math.max(Math.round((rect.width - width) / 2), 0),
    y: rect.y + Math.max(Math.round((rect.height - height) / 2), 0),
  }
}

export function clampPosition(x: number, y: number, width: number, height: number, rect: WindowRect) {
  const maxX = rect.x + Math.max(rect.width - width, 0)
  const maxY = rect.y + Math.max(rect.height - height, 0)

  return {
    x: Math.min(Math.max(Math.round(x), rect.x), maxX),
    y: Math.min(Math.max(Math.round(y), rect.y), maxY),
  }
}

function isAbnormalWindowSize(width: number, height: number) {
  return width < MIN_SETTINGS_WINDOW_WIDTH / 2 || height < MIN_SETTINGS_WINDOW_HEIGHT / 2
}
