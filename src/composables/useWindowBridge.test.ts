import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PetContext, ReminderPayload } from '@/types/window'
import {
  broadcastPetContext,
  usePetContextReceiver,
  useReminderPayloadReceiver,
} from '@/composables/useWindowBridge'
import { WINDOW_EVENTS } from '@/types/window'

type EventHandler<T> = (event: { payload: T }) => void

const mocks = vi.hoisted(() => {
  const handlers = new Map<string, EventHandler<unknown>>()
  const unlisten = vi.fn()
  const listen = vi.fn(async (eventName: string, handler: EventHandler<unknown>) => {
    handlers.set(eventName, handler)
    return unlisten
  })
  const emitTo = vi.fn(async (_target: string, _event: string, _payload?: unknown) => {})

  return {
    handlers,
    unlisten,
    resolveListen: null as ((nextUnlisten: typeof unlisten) => void) | null,
    listen,
    emitTo,
  }
})

vi.mock('@tauri-apps/api/event', () => ({
  listen: mocks.listen,
  emitTo: mocks.emitTo,
}))

const CONTEXT: PetContext = {
  info: null,
  mood: 'happy',
  roleId: 'monthly-salary-cat',
  scale: 0.85,
  interactionText: null,
  interactionLevel: 0,
}

let contextReceiver: ReturnType<typeof usePetContextReceiver> | null = null
let reminderReceiver: ReturnType<typeof useReminderPayloadReceiver> | null = null

const ContextHost = defineComponent({
  setup() {
    contextReceiver = usePetContextReceiver('info')
    return () => h('div')
  },
})

const ReminderHost = defineComponent({
  setup() {
    reminderReceiver = useReminderPayloadReceiver()
    return () => h('div')
  },
})

async function flushListeners() {
  await Promise.resolve()
  await nextTick()
}

describe('useWindowBridge', () => {
  beforeEach(() => {
    contextReceiver = null
    reminderReceiver = null
    mocks.handlers.clear()
    mocks.unlisten.mockReset()
    mocks.resolveListen = null
    mocks.listen.mockReset()
    mocks.listen.mockImplementation(async (eventName: string, handler: EventHandler<unknown>) => {
      mocks.handlers.set(eventName, handler)
      return mocks.unlisten
    })
    mocks.emitTo.mockReset()
    mocks.emitTo.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('requests the latest context after its receiver is ready', async () => {
    const wrapper = mount(ContextHost)
    await flushListeners()

    expect(mocks.listen).toHaveBeenCalledWith(WINDOW_EVENTS.petContext, expect.any(Function))
    expect(mocks.emitTo).toHaveBeenCalledWith('main', WINDOW_EVENTS.petContextRequest, {
      recipient: 'info',
    })
    expect(contextReceiver?.ready.value).toBe(false)

    const handler = mocks.handlers.get(WINDOW_EVENTS.petContext)
    handler?.({ payload: CONTEXT })
    await nextTick()

    expect(contextReceiver?.context.value).toEqual(CONTEXT)
    expect(contextReceiver?.ready.value).toBe(true)
    wrapper.unmount()
  })

  it('cleans up a context listener that resolves after unmount', async () => {
    mocks.listen.mockImplementation(
      (eventName: string, handler: EventHandler<unknown>) => new Promise<typeof mocks.unlisten>((resolve) => {
        mocks.handlers.set(eventName, handler)
        mocks.resolveListen = resolve
      }),
    )
    const wrapper = mount(ContextHost)

    wrapper.unmount()
    mocks.resolveListen?.(mocks.unlisten)
    await flushListeners()

    expect(mocks.unlisten).toHaveBeenCalledOnce()
    expect(mocks.emitTo).not.toHaveBeenCalled()
  })

  it('cleans up a reminder listener that resolves after unmount', async () => {
    mocks.listen.mockImplementation(
      (eventName: string, handler: EventHandler<unknown>) => new Promise<typeof mocks.unlisten>((resolve) => {
        mocks.handlers.set(eventName, handler)
        mocks.resolveListen = resolve
      }),
    )
    const wrapper = mount(ReminderHost)

    wrapper.unmount()
    mocks.resolveListen?.(mocks.unlisten)
    await flushListeners()

    expect(mocks.unlisten).toHaveBeenCalledOnce()
    expect(reminderReceiver?.payload.value).toEqual({
      reminder_id: '',
      message: '',
      snooze_minutes: 5,
    } satisfies ReminderPayload)
  })

  it('broadcasts context only to windows that subscribe to it', async () => {
    await broadcastPetContext(CONTEXT)

    const recipients = mocks.emitTo.mock.calls.map((call) => call[0])

    // 必须覆盖全部订阅方，否则窗口会停留在陈旧上下文
    expect(recipients).toContain('chat')
    expect(recipients).toContain('info')

    // 不得投递到无接收方的窗口：SystemInfo 以 1 秒周期变化，
    // 每次多投递一个窗口就等于每秒多一次无效 IPC 序列化与派发。
    expect(recipients).not.toContain('reminder')
    expect(recipients).not.toContain('feedback')
    expect(mocks.emitTo).toHaveBeenCalledTimes(2)
  })
})
