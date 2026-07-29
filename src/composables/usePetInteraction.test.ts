import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePetInteraction } from '@/composables/usePetInteraction'

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ startDragging: vi.fn() }),
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

function mouseEvent(type: string, screenX = 0, screenY = 0, altKey = false) {
  const event = new MouseEvent(type, { button: 0, altKey })
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
})
