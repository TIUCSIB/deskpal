/**
 * useWindowBridge.ts - 多窗口事件桥接
 * 集中管理桌宠、聊天、提醒和信息窗口之间的类型化事件。
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { emitTo, listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PetContext, ReminderPayload } from '@/types/window'
import { WINDOW_EVENTS } from '@/types/window'

const INITIAL_CONTEXT: PetContext = {
  info: null,
  mood: 'normal',
  scale: 1,
}

const INITIAL_REMINDER: ReminderPayload = {
  reminder_id: '',
  message: '',
  snooze_minutes: 5,
}

export async function broadcastPetContext(context: PetContext) {
  await Promise.allSettled([
    emitTo('chat', WINDOW_EVENTS.petContext, context),
    emitTo('info', WINDOW_EVENTS.petContext, context),
    emitTo('reminder', WINDOW_EVENTS.petContext, context),
  ])
}

export function usePetContextReceiver() {
  const context = ref<PetContext>({ ...INITIAL_CONTEXT })
  let unlisten: UnlistenFn | null = null

  onMounted(async () => {
    unlisten = await listen<PetContext>(WINDOW_EVENTS.petContext, (event) => {
      context.value = event.payload
    })
  })

  onUnmounted(() => unlisten?.())

  return { context }
}

export function useReminderPayloadReceiver() {
  const payload = ref<ReminderPayload>({ ...INITIAL_REMINDER })
  let unlisten: UnlistenFn | null = null

  onMounted(async () => {
    unlisten = await listen<ReminderPayload>(WINDOW_EVENTS.reminderPayload, (event) => {
      payload.value = event.payload
    })
  })

  onUnmounted(() => unlisten?.())

  return { payload }
}
