/**
 * usePetInteraction.ts - 桌宠拖拽与点击判定
 * 使用统一阈值区分点击和原生窗口拖拽。
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const DRAG_THRESHOLD = 5
const DRAG_ANIMATION_HOLD_MS = 1100

export type DragDirection = 'left' | 'right'

export function usePetInteraction() {
  const isDragging = ref(false)
  const dragDirection = ref<DragDirection | null>(null)
  let dragStartX = 0
  let dragStartY = 0
  let mouseDownOnPet = false
  let dragActive = false
  let suppressNextClick = false
  let dragReleaseTimer: ReturnType<typeof setTimeout> | null = null

  function exceedsThreshold(event: MouseEvent): boolean {
    const dx = Math.abs(event.screenX - dragStartX)
    const dy = Math.abs(event.screenY - dragStartY)
    return dx >= DRAG_THRESHOLD || dy >= DRAG_THRESHOLD
  }

  function handlePetPress(event: MouseEvent) {
    if (event.button !== 0) return
    dragStartX = event.screenX
    dragStartY = event.screenY
    mouseDownOnPet = true
    suppressNextClick = false
  }

  function shouldActivate(event: MouseEvent): boolean {
    const shouldSuppress = suppressNextClick || exceedsThreshold(event)
    mouseDownOnPet = false
    suppressNextClick = false
    return !shouldSuppress
  }

  async function handleWindowMouseMove(event: MouseEvent) {
    if (event.buttons !== 1 || !mouseDownOnPet || dragActive) return
    if (!exceedsThreshold(event)) return

    if (dragReleaseTimer) {
      clearTimeout(dragReleaseTimer)
      dragReleaseTimer = null
    }

    dragActive = true
    dragDirection.value = event.screenX >= dragStartX ? 'right' : 'left'
    isDragging.value = true
    suppressNextClick = true
    try {
      await getCurrentWindow().startDragging()
    } catch (error) {
      console.error('桌宠窗口拖拽失败:', error)
    } finally {
      dragActive = false
      mouseDownOnPet = false
      dragReleaseTimer = setTimeout(() => {
        isDragging.value = false
        dragDirection.value = null
        suppressNextClick = false
        dragReleaseTimer = null
      }, DRAG_ANIMATION_HOLD_MS)
    }
  }

  function handleWindowMouseUp() {
    if (!dragActive) mouseDownOnPet = false
  }

  onMounted(() => {
    window.addEventListener('mousemove', handleWindowMouseMove)
    window.addEventListener('mouseup', handleWindowMouseUp)
  })

  onUnmounted(() => {
    window.removeEventListener('mousemove', handleWindowMouseMove)
    window.removeEventListener('mouseup', handleWindowMouseUp)
  })

  return { handlePetPress, shouldActivate, isDragging, dragDirection }
}
