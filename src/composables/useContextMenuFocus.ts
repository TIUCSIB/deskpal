import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WINDOW_EVENTS } from '@/types/window'

const BLUR_HIDE_GUARD_MS = 160

/** 管理右键菜单每次打开的重置、聚焦与失焦关闭。 */
export function useContextMenuFocus(onFocus: () => void | Promise<void>, hide: () => Promise<void>) {
  let unlistenFocus: UnlistenFn | null = null
  let unlistenWindowFocus: UnlistenFn | null = null
  let delayedBlurHide: ReturnType<typeof setTimeout> | null = null
  let focusedAt = 0
  let hasFocused = false

  function cancelDelayedBlurHide() {
    if (delayedBlurHide === null) return
    clearTimeout(delayedBlurHide)
    delayedBlurHide = null
  }

  function hideAfterBlur() {
    if (!hasFocused) return
    hasFocused = false
    void hide()
  }

  function handleWindowBlur() {
    if (!hasFocused) return
    const remaining = BLUR_HIDE_GUARD_MS - (Date.now() - focusedAt)
    if (remaining <= 0) {
      hideAfterBlur()
      return
    }
    cancelDelayedBlurHide()
    delayedBlurHide = setTimeout(() => {
      delayedBlurHide = null
      hideAfterBlur()
    }, remaining)
  }

  async function focusMenu() {
    cancelDelayedBlurHide()
    hasFocused = true
    focusedAt = Date.now()
    await onFocus()
  }

  onMounted(() => {
    void (async () => {
      unlistenFocus = await listen(WINDOW_EVENTS.focusContextMenu, () => {
        void focusMenu()
      })
      unlistenWindowFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
        if (payload) {
          void focusMenu()
          return
        }
        handleWindowBlur()
      })
    })()
  })

  onUnmounted(() => {
    cancelDelayedBlurHide()
    unlistenFocus?.()
    unlistenWindowFocus?.()
  })
}
