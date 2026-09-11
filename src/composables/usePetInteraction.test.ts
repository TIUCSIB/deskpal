import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePetInteraction } from '@/composables/usePetInteraction'

/** 拖拽 Promise 的延迟 resolve 句柄，用于模拟原生拖拽的阻塞时长。 */
let resolveDrag: (() => void) | null = null

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    startDragging: () =>
      new Promise<void>((resolve) => {
        resolveDrag = resolve
      }),
  }),
}))

let interaction: ReturnType<typeof usePetInteraction> | null = null
let now = 0
let passthroughEnabled = false

const InteractionHost = defineComponent({
  setup() {
    interaction = usePetInteraction(() => now, {
      leftClickPassthrough: () => passthroughEnabled,
    })
    return () => h('div')
  },
})

function mouseEvent(type: string, screenX = 0, screenY = 0, altKey = false, buttons = 0) {
  const event = new MouseEvent(type, { button: 0, altKey, buttons })
  Object.defineProperties(event, {
    screenX: { value: screenX },
    screenY: { value: screenY },
  })
  return event
}

describe('usePetInteraction', () => {
  beforeEach(() => {
    interaction = null
    now = 0
    passthroughEnabled = false
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('keeps stationary clicks eligible for activation', async () => {
    const wrapper = mount(InteractionHost)
    await nextTick()

    interaction?.handlePetPress(mouseEvent('mousedown'))
    expect(interaction?.shouldActivate(mouseEvent('click'))).toBe(true)
    wrapper.unmount()
  })

  it('does not accept a dragged click as an activation', async () => {
    const wrapper = mount(InteractionHost)
    await nextTick()

    interaction?.handlePetPress(mouseEvent('mousedown', 0, 0))
    expect(interaction?.shouldActivate(mouseEvent('click', 5, 0))).toBe(false)
    wrapper.unmount()
  })

  it('rate limits click feedback without affecting click eligibility', async () => {
    const wrapper = mount(InteractionHost)
    await nextTick()

    expect(interaction?.tryTriggerClickFeedback()).toBe(true)
    now = 1499
    expect(interaction?.tryTriggerClickFeedback()).toBe(false)
    now = 1500
    expect(interaction?.tryTriggerClickFeedback()).toBe(true)
    wrapper.unmount()
  })

  it('blocks normal left click activation when left-click passthrough is enabled', async () => {
    passthroughEnabled = true
    const wrapper = mount(InteractionHost)
    await nextTick()

    interaction?.handlePetPress(mouseEvent('mousedown'))
    expect(interaction?.shouldActivate(mouseEvent('click'))).toBe(false)
    wrapper.unmount()
  })

  it('suppresses chat activation even for Alt + 左键 when left-click passthrough is enabled', async () => {
    passthroughEnabled = true
    const wrapper = mount(InteractionHost)
    await nextTick()

    interaction?.handlePetPress(mouseEvent('mousedown', 0, 0, true))
    expect(interaction?.shouldActivate(mouseEvent('click', 0, 0, true))).toBe(false)
    wrapper.unmount()
  })

  it('resets isDragging immediately on drag end, while keeping the animation hold', async () => {
    vi.useFakeTimers()
    try {
      const wrapper = mount(InteractionHost)
      await nextTick()

      interaction?.handlePetPress(mouseEvent('mousedown', 0, 0))
      window.dispatchEvent(mouseEvent('mousemove', 10, 0, false, 1))
      await nextTick()
      expect(interaction?.isDragging.value).toBe(true)
      expect(interaction?.isDragAnimating.value).toBe(true)

      resolveDrag?.()
      await vi.advanceTimersByTimeAsync(0)

      // 拖拽结束后 isDragging 立即复位：它同时作为 hover 判定的互斥条件，
      // 延迟复位会在落地动画期间持续吞掉鼠标悬停事件。
      expect(interaction?.isDragging.value).toBe(false)
      // 动画保持状态仍需覆盖完整时长，供落地动作播放。
      expect(interaction?.isDragAnimating.value).toBe(true)
      expect(interaction?.dragDirection.value).toBe('right')

      await vi.advanceTimersByTimeAsync(1100)
      expect(interaction?.isDragAnimating.value).toBe(false)
      expect(interaction?.dragDirection.value).toBe(null)
      wrapper.unmount()
    } finally {
      vi.useRealTimers()
    }
  })
})
