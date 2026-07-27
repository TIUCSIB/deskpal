import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { OverlaySide } from '@/types/window'
import { useOverlayTransition } from '@/composables/useOverlayTransition'

const mocks = vi.hoisted(() => ({
  handler: null as ((event: { payload: OverlaySide }) => void) | null,
  unlisten: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_eventName: string, handler: (event: { payload: OverlaySide }) => void) => {
    mocks.handler = handler
    return mocks.unlisten
  }),
}))

let overlay: ReturnType<typeof useOverlayTransition> | null = null

const OverlayHost = defineComponent({
  setup() {
    overlay = useOverlayTransition()
    return () => h('div')
  },
})

async function mountOverlay() {
  const wrapper = mount(OverlayHost)
  await Promise.resolve()
  await nextTick()
  return wrapper
}

describe('useOverlayTransition', () => {
  beforeEach(() => {
    overlay = null
    mocks.handler = null
    mocks.unlisten.mockReset()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('maps a native placement direction to CSS variables', async () => {
    const wrapper = await mountOverlay()

    mocks.handler?.({ payload: 'left' })
    await nextTick()

    expect(overlay?.side.value).toBe('left')
    expect(overlay?.transitionStyle.value).toMatchObject({
      '--overlay-enter-x': '8px',
      '--overlay-enter-y': '0px',
      '--overlay-origin': 'right center',
    })
    wrapper.unmount()
  })

  it('alternates the animation name for every presentation without remounting content', async () => {
    const wrapper = await mountOverlay()

    mocks.handler?.({ payload: 'above' })
    await nextTick()
    const firstRevision = overlay?.revision.value

    mocks.handler?.({ payload: 'right' })
    await nextTick()

    expect(firstRevision).toBe(1)
    expect(overlay?.revision.value).toBe(2)
    expect(overlay?.side.value).toBe('right')
    expect(overlay?.transitionStyle.value).toMatchObject({
      '--overlay-enter-x': '-8px',
      '--overlay-origin': 'left center',
    })
    wrapper.unmount()
  })

  it('removes its Tauri listener when the window component unmounts', async () => {
    const wrapper = await mountOverlay()

    wrapper.unmount()

    expect(mocks.unlisten).toHaveBeenCalledOnce()
  })
})
