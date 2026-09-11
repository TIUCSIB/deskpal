import { defineComponent, h, nextTick, ref } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePixelHitTest } from '@/composables/usePixelHitTest'

type PixelHitTest = ReturnType<typeof usePixelHitTest>

class MockImage {
  complete = true
  crossOrigin: string | null = null
  naturalHeight = 100
  naturalWidth = 100
  onerror: ((event: Event) => void) | null = null
  onload: ((event: Event) => void) | null = null
  private source = ''

  get src() {
    return this.source
  }

  set src(value: string) {
    this.source = value
    this.onload?.(new Event('load'))
  }
}

let pixelHitTest: PixelHitTest | null = null

const Host = defineComponent({
  setup() {
    pixelHitTest = usePixelHitTest(
      ref('http://role-pack.localhost/tiny-crt'),
      ref('0px 0px'),
      ref('100px 100px'),
    )
    return () => h('div')
  },
})

describe('usePixelHitTest', () => {
  const clearRect = vi.fn()
  const drawImage = vi.fn()
  const getImageData = vi.fn()
  let getContext: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    pixelHitTest = null
    clearRect.mockReset()
    drawImage.mockReset()
    getImageData.mockReset()
    vi.stubGlobal('Image', MockImage)
    getContext = vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
      clearRect,
      drawImage,
      getImageData,
    } as unknown as CanvasRenderingContext2D)
  })

  afterEach(() => {
    getContext.mockRestore()
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('keeps pixel-level transparency checks for readable role sprites', async () => {
    getImageData.mockReturnValue({ data: new Uint8ClampedArray([0, 0, 0, 0]) } as ImageData)
    const wrapper = mount(Host)
    await nextTick()

    expect(pixelHitTest?.hitTest(20, 30)).toBe(false)
    expect(drawImage).toHaveBeenCalledWith(expect.any(MockImage), 20, 30, 1, 1, 0, 0, 1, 1)

    // 换一帧后同一容器坐标对应不同源图像素，缓存须失效并重新回读
    pixelHitTest?.setFrameKey('0px -100px')
    getImageData.mockReturnValue({ data: new Uint8ClampedArray([0, 0, 0, 255]) } as ImageData)
    expect(pixelHitTest?.hitTest(20, 30)).toBe(true)
    wrapper.unmount()
  })

  it('reuses the cached alpha result for a repeated pixel within one frame', async () => {
    getImageData.mockReturnValue({ data: new Uint8ClampedArray([0, 0, 0, 255]) } as ImageData)
    const wrapper = mount(Host)
    await nextTick()

    expect(pixelHitTest?.hitTest(20, 30)).toBe(true)
    expect(pixelHitTest?.hitTest(20, 30)).toBe(true)
    expect(pixelHitTest?.hitTest(20, 30)).toBe(true)

    // 同一帧内同一像素只回读一次；getImageData 是同步的 GPU→CPU 回读，
    // 在 pointermove 频率下重复调用会构成明显开销。
    expect(getImageData).toHaveBeenCalledOnce()
    expect(drawImage).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('invalidates the pixel cache when the frame changes', async () => {
    getImageData.mockReturnValue({ data: new Uint8ClampedArray([0, 0, 0, 0]) } as ImageData)
    const wrapper = mount(Host)
    await nextTick()

    expect(pixelHitTest?.hitTest(20, 30)).toBe(false)
    pixelHitTest?.setFrameKey('0px -100px')
    expect(pixelHitTest?.hitTest(20, 30)).toBe(false)

    // 帧变化后必须重新回读，否则会返回上一帧的陈旧结果
    expect(getImageData).toHaveBeenCalledTimes(2)
    wrapper.unmount()
  })

  it('falls back to container interaction when canvas pixel reads are blocked', async () => {
    getImageData.mockImplementation(() => {
      throw new DOMException('Canvas is tainted.', 'SecurityError')
    })
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const wrapper = mount(Host)
    await nextTick()

    expect(pixelHitTest?.hitTest(20, 30)).toBe(true)
    expect(pixelHitTest?.hitTest(40, 50)).toBe(true)
    expect(getImageData).toHaveBeenCalledOnce()
    expect(warn).toHaveBeenCalledOnce()
    wrapper.unmount()
  })
})
