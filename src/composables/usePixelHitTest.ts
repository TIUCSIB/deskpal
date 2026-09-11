/**
 * usePixelHitTest.ts - 像素级点击穿透检测
 * 通过离屏 canvas 读取精灵表源图像素 alpha 值，
 * 判断鼠标点击位置是否落在非透明区域。
 *
 * 性能约束说明（供后续维护参考）：
 *
 * `hitTest` 由 pointermove 驱动，而调用方 `Pet.vue` 已按 requestAnimationFrame
 * 节流，因此本函数实际以约 60 Hz 上限被调用。真正的开销点是 `getImageData`
 * —— 同步的 GPU→CPU 回读。本模块用两级缓存压制它：
 *
 * 1. **字符串解析缓存**（`parsedSizeFor` / `parsedPosFor`）：background-size /
 *    position 仅在字符串变化时才重新 `split` + `parseFloat`。
 * 2. **逐像素 alpha 缓存**（`alphaCache`）：同一帧内同一源图像素的 alpha 只回读
 *    一次。有效场景是同一帧内的连续查询 —— 如 mousedown 与 click 相邻触发、
 *    pointerenter 后紧跟 pointermove。
 *
 * 注意：逐像素缓存按帧失效（见 `setFrameKey`），而动画帧周期（约 80–125ms）
 * 远大于 RAF 节流周期（约 16ms），故跨帧复用收益很小 —— 缓存的价值集中在
 * 同帧内的重复查询上。若后续发现同帧重复查询不足，可考虑放宽为按帧 ID 分桶
 * 保留最近若干帧，但需评估内存与失效逻辑的复杂度。
 *
 * **降级语义**：`getImageData` 可能因 GPU 上下文重建等原因失败。降级时不能
 * 一律返回 `true` —— 那会把宠物周围的透明区域也判为命中，导致浮窗在空白处
 * 乱弹。正确做法是沿用最近一次**成功**读取的结果，并按 `READ_RETRY_INTERVAL_MS`
 * 定期重试以自动恢复；从未成功过时保守返回 `false`。
 */
import { onMounted, ref, watch } from 'vue'

/** 逐像素 alpha 缓存上限。超过后清空，避免长时间运行内存无限增长。 */
const ALPHA_CACHE_LIMIT = 4096

/**
 * 像素回读失败后的重试间隔（毫秒）。
 *
 * `getImageData` 的失败通常是暂时性的 —— GPU 上下文重建、合成器切换、
 * 标签页被系统挂起等。若不设重试，一次失败即永久降级，用户只能重启应用。
 * 该间隔需大于一次帧循环（约 80–125 ms），避免失败期间每个 pointermove
 * 都重试一次而放大卡顿。
 */
const READ_RETRY_INTERVAL_MS = 1000

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

  /** 逐像素 alpha 缓存：key = `${srcX},${srcY}`，值为是否非透明。 */
  let alphaCache = new Map<string, boolean>()
  /** 缓存所属的帧标识，帧变化时整体失效。 */
  let alphaCacheFrame = ''
  /** background-size / position 的解析缓存，避免每次调用都做字符串解析。 */
  let parsedSizeFor = ''
  let parsedSize: number[] = []
  let parsedPosFor = ''
  let parsedPos: number[] = []
  /**
   * 最近一次成功读取到的命中结果。
   *
   * 回读失败期间以它作为降级返回值：指针位置与上次成功判定相差通常不超过
   * 几个像素，沿用上一次结果比"一律视为命中"或"一律视为未命中"都更接近真实。
   * `null` 表示本次会话尚未有过成功读取，此时保守返回 `false`（未命中）——
   * 宁可暂不弹出浮窗，也不要让空白区域误触发。
   */
  let lastKnownHit: boolean | null = null
  /** 上次回读失败的时间戳，用于控制重试节奏。 */
  let lastReadFailureAt = 0

  function loadImage(url: string) {
    isReady.value = false
    pixelReadFailed.value = false
    alphaCache = new Map()
    alphaCacheFrame = ''
    lastKnownHit = null
    lastReadFailureAt = 0
    image.onload = () => { isReady.value = true }
    image.onerror = () => { isReady.value = false }
    image.crossOrigin = 'anonymous'
    image.src = url
    if (image.complete && image.naturalWidth > 0) isReady.value = true
  }

  onMounted(() => loadImage(imageUrl.value))
  watch(() => imageUrl.value, loadImage)

  /**
   * 标记当前帧。帧切换后逐像素缓存必须失效 —— 同一容器坐标在不同帧
   * 对应不同源图像素，缓存跨帧复用会得到错误的命中结果。
   */
  function setFrameKey(frameKey: string) {
    if (frameKey === alphaCacheFrame) return
    alphaCacheFrame = frameKey
    alphaCache.clear()
  }

  /**
   * 检测相对于 .pet 容器的坐标是否落在非透明像素上
   * @param divX 相对于宠物容器的 X 坐标
   * @param divY 相对于宠物容器的 Y 坐标
   * @returns true = 非透明（应响应），false = 透明（应忽略）
   */
  function hitTest(divX: number, divY: number): boolean {
    // 精灵图尚未就绪时无法判定像素，保守地视为未命中，
    // 避免拖动经过空白区域时反复误触发 hover。
    // 图片加载完成后 isReady 会置真，后续 pointermove 会重新评测。
    if (!isReady.value) return false
    // 回读处于失败状态：仅在超过重试间隔后才尝试恢复，期间沿用上次成功结果。
    if (pixelReadFailed.value && Date.now() - lastReadFailureAt < READ_RETRY_INTERVAL_MS) {
      return lastKnownHit ?? false
    }

    // 仅在字符串实际变化时重新解析（帧切换时字符串会变，常态下命中缓存）
    const sizeValue = backgroundSize.value
    if (sizeValue !== parsedSizeFor) {
      parsedSizeFor = sizeValue
      parsedSize = sizeValue.split(' ').map(parseFloat)
    }
    const posValue = backgroundPosition.value
    if (posValue !== parsedPosFor) {
      parsedPosFor = posValue
      parsedPos = posValue.split(' ').map(parseFloat)
    }

    const scale = parsedSize[0] / image.naturalWidth
    if (!Number.isFinite(scale) || scale <= 0) return false

    const srcX = Math.round((divX - parsedPos[0]) / scale)
    const srcY = Math.round((divY - parsedPos[1]) / scale)
    if (srcX < 0 || srcX >= image.naturalWidth || srcY < 0 || srcY >= image.naturalHeight) {
      lastKnownHit = false
      return false
    }

    const cacheKey = `${srcX},${srcY}`
    const cached = alphaCache.get(cacheKey)
    if (cached !== undefined) {
      lastKnownHit = cached
      return cached
    }

    try {
      ctx.clearRect(0, 0, 1, 1)
      ctx.drawImage(image, srcX, srcY, 1, 1, 0, 0, 1, 1)
      const alpha = ctx.getImageData(0, 0, 1, 1).data[3]
      const hit = alpha > 0
      // 读取成功：退出降级状态并记录结果，供后续失败期间沿用。
      pixelReadFailed.value = false
      lastKnownHit = hit
      if (alphaCache.size >= ALPHA_CACHE_LIMIT) alphaCache.clear()
      alphaCache.set(cacheKey, hit)
      return hit
    } catch (error) {
      // 降级：不再永久返回"命中"（那会让空白区域也触发浮窗），
      // 改为沿用上次成功结果；若从未成功过则保守判为未命中。
      pixelReadFailed.value = true
      lastReadFailureAt = Date.now()
      console.warn('宠物精灵图无法进行像素命中检测，已降级为沿用上次结果。', error)
      return lastKnownHit ?? false
    }
  }

  return { hitTest, isReady, setFrameKey }
}
