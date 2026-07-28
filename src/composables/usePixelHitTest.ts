/**
 * usePixelHitTest.ts - 像素级点击穿透检测
 * 通过离屏 canvas 读取精灵表源图像素 alpha 值，
 * 判断鼠标点击位置是否落在非透明区域。
 */
import { onMounted, ref, watch } from 'vue'

/**
 * @param imageUrl 精灵表图片 URL（由 Vite import 提供）
 * @param backgroundPosition 当前帧的 background-position 值
 * @param backgroundSize 当前帧的 background-size 值
 */
export function usePixelHitTest(
  imageUrl: { readonly value: string },
  backgroundPosition: { readonly value: string },
  backgroundSize: { readonly value: string },
) {
  const image = new Image()
  const isReady = ref(false)
  const pixelReadFailed = ref(false)
  const canvas = document.createElement('canvas')
  canvas.width = 1
  canvas.height = 1
  const ctx = canvas.getContext('2d', { willReadFrequently: true })!

  function loadImage(url: string) {
    isReady.value = false
    pixelReadFailed.value = false
    image.onload = () => { isReady.value = true }
    image.onerror = () => { isReady.value = false }
    image.crossOrigin = 'anonymous'
    image.src = url
    if (image.complete && image.naturalWidth > 0) isReady.value = true
  }

  onMounted(() => loadImage(imageUrl.value))
  watch(() => imageUrl.value, loadImage)

  /**
   * 检测相对于 .pet 容器的坐标是否落在非透明像素上
   * @param divX 相对于宠物容器的 X 坐标
   * @param divY 相对于宠物容器的 Y 坐标
   * @returns true = 非透明（应响应），false = 透明（应忽略）
   */
  function hitTest(divX: number, divY: number): boolean {
    if (!isReady.value || pixelReadFailed.value) return true

    const bgSize = backgroundSize.value.split(' ').map(parseFloat)
    const bgPos = backgroundPosition.value.split(' ').map(parseFloat)
    const scale = bgSize[0] / image.naturalWidth
    if (!Number.isFinite(scale) || scale <= 0) return false

    const srcX = Math.round((divX - bgPos[0]) / scale)
    const srcY = Math.round((divY - bgPos[1]) / scale)
    if (srcX < 0 || srcX >= image.naturalWidth || srcY < 0 || srcY >= image.naturalHeight) {
      return false
    }

    try {
      ctx.clearRect(0, 0, 1, 1)
      ctx.drawImage(image, srcX, srcY, 1, 1, 0, 0, 1, 1)
      const alpha = ctx.getImageData(0, 0, 1, 1).data[3]
      return alpha > 0
    } catch (error) {
      pixelReadFailed.value = true
      console.warn('宠物精灵图无法进行像素命中检测，已启用容器交互。', error)
      return true
    }
  }

  return { hitTest }
}
