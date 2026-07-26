/**
 * useWindowBridge.ts - 多窗口事件桥接
 * 集中管理桌宠、聊天和信息窗口之间的类型化事件。
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { emitTo, listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PetContext } from '@/types/window'
import { WINDOW_EVENTS } from '@/types/window'

const INITIAL_CONTEXT: PetContext = {
  info: null,
  mood: 'normal',
  scale: 1,
}

export async function broadcastPetContext(context: PetContext) {
  await Promise.allSettled([
    emitTo('chat', WINDOW_EVENTS.petContext, context),
    emitTo('info', WINDOW_EVENTS.petContext, context),
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
