import { computed, onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WINDOW_EVENTS, type OverlaySide } from '@/types/window'

const TRANSITION_STYLES: Record<OverlaySide, Record<string, string>> = {
  above: {
    '--overlay-enter-x': '0px',
    '--overlay-enter-y': '8px',
    '--overlay-origin': 'center bottom',
  },
  below: {
    '--overlay-enter-x': '0px',
    '--overlay-enter-y': '-8px',
    '--overlay-origin': 'center top',
  },
  left: {
    '--overlay-enter-x': '8px',
    '--overlay-enter-y': '0px',
    '--overlay-origin': 'right center',
  },
  right: {
    '--overlay-enter-x': '-8px',
    '--overlay-enter-y': '0px',
    '--overlay-origin': 'left center',
  },
}

/** useOverlayTransition - 浮窗显示方向与重复进入动画 */
export function useOverlayTransition() {
  const side = ref<OverlaySide>('above')
  const revision = ref(0)
  let unlisten: UnlistenFn | null = null
  let startPromise: Promise<void> | null = null
  let disposed = false

  const transitionStyle = computed(() => TRANSITION_STYLES[side.value])

  /** 应用 native 端计算出的浮窗方向 */
  function present(nextSide: OverlaySide) {
    side.value = nextSide
    revision.value += 1
  }

  function start() {
    if (unlisten || startPromise) return startPromise ?? Promise.resolve()

    disposed = false
    startPromise = listen<OverlaySide>(WINDOW_EVENTS.overlayPresent, (event) => {
      present(event.payload)
    }).then((nextUnlisten) => {
      startPromise = null
      if (disposed) {
        nextUnlisten()
        return
      }
      unlisten = nextUnlisten
    })

    return startPromise
  }

  function dispose() {
    disposed = true
    unlisten?.()
    unlisten = null
  }

  onMounted(() => {
    void start()
  })

  onUnmounted(dispose)

  return {
    side,
    revision,
    transitionStyle,
    present,
    start,
    dispose,
  }
}
