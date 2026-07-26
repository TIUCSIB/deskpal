import { describe, expect, it } from 'vitest'
import { useChat } from '@/composables/useChat'
import type { SystemInfo } from '@/types/system'

const INFO: SystemInfo = {
  cpu_usage: 12.5,
  memory_usage: 45.2,
  memory_used_mb: 4096,
  memory_total_mb: 8192,
  disk_usage: 61.7,
  network_down_kbps: 128.4,
  network_up_kbps: 32.1,
  uptime_secs: 3661,
}

describe('useChat', () => {
  it('adds user message and reply immediately', () => {
    const chat = useChat()
    chat.inputText.value = 'CPU 使用率是多少'

    chat.sendMessage(INFO, 'normal')

    expect(chat.inputText.value).toBe('')
    expect(chat.messages.value).toHaveLength(2)
    expect(chat.messages.value[0]).toEqual({ text: 'CPU 使用率是多少', isUser: true })
    expect(chat.messages.value[1]).toEqual({ text: '当前 CPU 使用率 12.5%', isUser: false })
  })

  it('keeps only latest messages', () => {
    const chat = useChat()
    chat.clearMessages()

    for (let i = 0; i < 12; i += 1) {
      chat.inputText.value = `hello ${i}`
      chat.sendMessage(INFO, 'normal')
    }

    expect(chat.messages.value).toHaveLength(20)
    expect(chat.messages.value[0].text).toBe('hello 2')
  })

  it('clears messages', () => {
    const chat = useChat()
    chat.clearMessages()
    chat.inputText.value = '你好'
    chat.sendMessage(INFO, 'normal')

    chat.clearMessages()

    expect(chat.messages.value).toHaveLength(0)
  })

  it('adds system message as pet message', () => {
    const chat = useChat()
    chat.clearMessages()

    chat.addSystemMessage('系统状态一切正常')

    expect(chat.messages.value).toEqual([{ text: '系统状态一切正常', isUser: false }])
  })
})
