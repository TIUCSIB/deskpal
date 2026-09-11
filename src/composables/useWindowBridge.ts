/**
 * useWindowBridge.ts - 多窗口事件桥接
 * 集中管理桌宠、聊天、提醒和信息窗口之间的类型化事件。
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { emitTo, listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  PetContext,
  PetContextRecipient,
  ReminderPayload,
} from '@/types/window'
import type { SystemFeedbackPayload } from '@/types/systemFeedback'
import { WINDOW_EVENTS } from '@/types/window'

const INITIAL_CONTEXT: PetContext = {
  info: null,
  mood: 'normal',
  roleId: 'guga',
  scale: 1,
  interactionText: null,
  interactionLevel: 0,
}

const INITIAL_REMINDER: ReminderPayload = {
  reminder_id: '',
  message: '',
  snooze_minutes: 5,
}

export function sendPetContext(recipient: PetContextRecipient, context: PetContext) {
  return emitTo(recipient, WINDOW_EVENTS.petContext, context)
}

/**
 * 广播桌宠上下文到所有订阅窗口。
 *
 * 投递目标必须与实际调用 `usePetContextReceiver` 的窗口严格一致：
 * `reminder` 与 `feedback` 窗口不消费上下文，向它们投递只会产生无接收方的
 * IPC 序列化与跨窗口派发开销。由于 `SystemInfo` 以 1 秒周期变化，此处每次
 * 多投递一个窗口就等于每秒多一次无效 IPC。
 *
 * 当前订阅方：`chat`（显示宠物与系统信息）、`info`（悬停系统信息浮窗）。
 */
export async function broadcastPetContext(context: PetContext) {
  await Promise.allSettled([
    sendPetContext('chat', context),
    sendPetContext('info', context),
  ])
}

export function usePetContextReceiver(recipient: PetContextRecipient) {
  const context = ref<PetContext>({ ...INITIAL_CONTEXT })
  const ready = ref(false)
  let unlisten: UnlistenFn | null = null
  let startPromise: Promise<void> | null = null
  let disposed = false

  function start() {
    if (unlisten || startPromise) return startPromise ?? Promise.resolve()

    disposed = false
    startPromise = listen<PetContext>(WINDOW_EVENTS.petContext, (event) => {
      context.value = event.payload
      ready.value = true
    }).then((nextUnlisten) => {
      startPromise = null
      if (disposed) {
        nextUnlisten()
        return
      }
      unlisten = nextUnlisten
      void emitTo('main', WINDOW_EVENTS.petContextRequest, { recipient }).catch((error: unknown) => {
        console.error('请求当前桌宠状态失败:', error)
      })
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

  return { context, ready, start, dispose }
}

export function useSystemFeedbackPayloadReceiver() {
  const payload = ref<SystemFeedbackPayload | null>(null)
  let unlisten: UnlistenFn | null = null
  let startPromise: Promise<void> | null = null
  let disposed = false

  function start() {
    if (unlisten || startPromise) return startPromise ?? Promise.resolve()

    disposed = false
    startPromise = listen<SystemFeedbackPayload>(WINDOW_EVENTS.systemFeedbackPayload, (event) => {
      payload.value = event.payload
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

  return { payload, start, dispose }
}

export function useReminderPayloadReceiver() {
  const payload = ref<ReminderPayload>({ ...INITIAL_REMINDER })
  let unlisten: UnlistenFn | null = null
  let startPromise: Promise<void> | null = null
  let disposed = false

  function start() {
    if (unlisten || startPromise) return startPromise ?? Promise.resolve()

    disposed = false
    startPromise = listen<ReminderPayload>(WINDOW_EVENTS.reminderPayload, (event) => {
      payload.value = event.payload
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

  return { payload, start, dispose }
}
