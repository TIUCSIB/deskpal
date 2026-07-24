import { ref } from 'vue'
import type { SystemInfo, PetMood } from '@/types/system'

interface ChatMessage {
  text: string
  isUser: boolean
}

/** 根据系统状态和心情生成桌宠回复 */
export function generateReply(
  userText: string,
  info: SystemInfo | null,
  mood: PetMood,
): string {
  const text = userText.trim().toLowerCase()

  // —— 关键词匹配 ——
  if (text.includes('内存') || text.includes('memory')) {
    if (!info) return '我还没拿到内存数据呢...'
    return `现在内存用了 ${info.memory_usage.toFixed(1)}%（${info.memory_used_mb}/${info.memory_total_mb} MB）`
  }

  if (text.includes('cpu') || text.includes('处理器')) {
    if (!info) return 'CPU 数据还没到...'
    return `当前 CPU 使用率 ${info.cpu_usage.toFixed(1)}%`
  }

  if (text.includes('磁盘') || text.includes('disk')) {
    if (!info) return '磁盘数据还没到...'
    return `磁盘已经用了 ${info.disk_usage.toFixed(1)}% 啦`
  }

  if (text.includes('运行') || text.includes('uptime') || text.includes('多久')) {
    if (!info) return '还不知道运行了多久...'
    const h = Math.floor(info.uptime_secs / 3600)
    const m = Math.floor((info.uptime_secs % 3600) / 60)
    return `电脑已经运行了 ${h} 小时 ${m} 分钟`
  }

  if (text.includes('你好') || text.includes('hi') || text.includes('hello')) {
    return '你好呀！有什么我可以帮你的吗？'
  }

  if (text.includes('累') || text.includes('辛苦')) {
    return '你辛苦啦～记得适当休息哦！'
  }

  // —— 根据心情生成闲聊 ——
  if (mood === 'warning' && info) {
    if (info.cpu_usage > 80) {
      return `CPU 负载有点高（${info.cpu_usage.toFixed(1)}%），看看是不是有程序卡住了？`
    }
    return `内存快满了（${info.memory_usage.toFixed(1)}%），建议清理一下后台程序～`
  }

  if (mood === 'sleepy') {
    const sleepyReplies = [
      '这么晚了还不睡吗？注意身体哦～',
      '夜深了，明天再忙吧～',
      '我也困了...要不要一起休息？',
    ]
    return sleepyReplies[Math.floor(Math.random() * sleepyReplies.length)]
  }

  // 默认闲聊
  const defaultReplies = [
    '我很好，谢谢关心！',
    '今天心情不错～',
    '有什么想聊的吗？',
    '系统状态一切正常！',
    '我在认真监控你的电脑哦～',
    info ? `悄悄告诉你，CPU 现在 ${info.cpu_usage.toFixed(1)}%` : '正在获取系统信息...',
  ]
  return defaultReplies[Math.floor(Math.random() * defaultReplies.length)]
}

/** 时间感知的打招呼 */
export function getGreeting(): string {
  const hour = new Date().getHours()
  if (hour >= 5 && hour < 11) return '早上好！新的一天开始了～'
  if (hour >= 11 && hour < 14) return '中午好！记得吃午饭哦～'
  if (hour >= 14 && hour < 18) return '下午好！继续加油～'
  if (hour >= 18 && hour < 22) return '晚上好！辛苦一天了～'
  return '这么晚了，注意休息哦～'
}

export function useChat() {
  const messages = ref<ChatMessage[]>([])
  const inputText = ref('')

  function sendMessage(info: SystemInfo | null, mood: PetMood) {
    const text = inputText.value.trim()
    if (!text) return

    // 添加用户消息
    messages.value.push({ text, isUser: true })

    // 生成回复
    const reply = generateReply(text, info, mood)
    messages.value.push({ text: reply, isUser: false })

    // 清空输入
    inputText.value = ''

    // 限制消息数量，保留最近 20 条
    if (messages.value.length > 20) {
      messages.value = messages.value.slice(-20)
    }
  }

  function addSystemMessage(text: string) {
    messages.value.push({ text, isUser: false })
  }

  function clearMessages() {
    messages.value = []
  }

  return {
    messages,
    inputText,
    sendMessage,
    addSystemMessage,
    clearMessages,
  }
}
