/**
 * usePixelHitTest.ts - 像素级点击穿透检测
 * 通过离屏 canvas 读取精灵表源图像素 alpha 值，
 * 判断鼠标点击位置是否落在非透明区域。
 */
import { ref, onMounted } from 'vue'

/**
 * @param imageUrl 精灵表图片 URL（由 Vite import 提供）
 * @param backgroundPosition 当前帧的 background-position 值
 * @param backgroundSize 当前帧的 background-size 值
 */
export function usePixelHitTest(
  imageUrl: string,
  backgroundPosition: { readonly value: string },
  backgroundSize: { readonly value: string },
) {
  const image = new Image()
  image.src = imageUrl

  /** 图片是否加载完成 */
  const isReady = ref(false)

  /** 离屏 canvas，用于读取单像素 */
  const canvas = document.createElement('canvas')
  canvas.width = 1
  canvas.height = 1
  const ctx = canvas.getContext('2d', { willReadFrequently: true })!

  onMounted(() => {
    if (image.complete && image.naturalWidth > 0) {
      isReady.value = true
    } else {
      image.onload = () => { isReady.value = true }
    }
  })

  /**
   * 检测相对于 .pet 容器的坐标是否落在非透明像素上
   * @param divX 相对于宠物容器的 X 坐标
   * @param divY 相对于宠物容器的 Y 坐标
   * @returns true = 非透明（应响应），false = 透明（应忽略）
   */
  function hitTest(divX: number, divY: number): boolean {
    if (!isReady.value) return true

    // 从 background-size / background-position 反推源图坐标
    const bgSize = backgroundSize.value.split(' ').map(parseFloat)
    const bgPos = backgroundPosition.value.split(' ').map(parseFloat)
    const scale = bgSize[0] / image.naturalWidth

    if (!Number.isFinite(scale) || scale <= 0) return false

    const srcX = Math.round((divX - bgPos[0]) / scale)
    const srcY = Math.round((divY - bgPos[1]) / scale)

    // 超出源图范围，视为透明
    if (srcX < 0 || srcX >= image.naturalWidth || srcY < 0 || srcY >= image.naturalHeight) {
      return false
    }

    // 读取单像素 alpha 值
    ctx.clearRect(0, 0, 1, 1)
    ctx.drawImage(image, srcX, srcY, 1, 1, 0, 0, 1, 1)
    const alpha = ctx.getImageData(0, 0, 1, 1).data[3]
    return alpha > 0
  }

  return { hitTest }
}
