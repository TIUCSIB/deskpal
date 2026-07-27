import { ref } from 'vue'
import { getPetPersonality } from '@/config/petPersonalities'
import { DEFAULT_PET_ROLE } from '@/config/petRoles'
import type { PetRoleId } from '@/types/pet'
import type { SystemInfo, PetMood } from '@/types/system'
import { chooseWeighted } from '@/utils/weightedChoice'

interface ChatMessage {
  text: string
  isUser: boolean
}

function chooseReply(replies: string[], random: () => number): string {
  return chooseWeighted(replies.map((value) => ({ value, weight: 1 })), random) ?? ''
}

/** 根据角色人格、系统状态和心情生成桌宠回复 */
export function generateReply(
  userText: string,
  info: SystemInfo | null,
  mood: PetMood,
  roleId: PetRoleId = DEFAULT_PET_ROLE,
  random: () => number = Math.random,
): string {
  const text = userText.trim().toLowerCase()
  const personality = getPetPersonality(roleId)
  const system = personality.systemReplies

  if (text.includes('内存') || text.includes('memory')) {
    return info
      ? system.memory(`${info.memory_usage.toFixed(1)}%（${info.memory_used_mb}/${info.memory_total_mb} MB）`)
      : system.missing
  }

  if (text.includes('cpu') || text.includes('处理器')) {
    return info ? system.cpu(`${info.cpu_usage.toFixed(1)}%`) : system.missing
  }

  if (text.includes('磁盘') || text.includes('disk')) {
    return info ? system.disk(`${info.disk_usage.toFixed(1)}%`) : system.missing
  }

  if (text.includes('网络') || text.includes('网速') || text.includes('上传') || text.includes('下载')) {
    return info
      ? system.network(`${info.network_down_kbps.toFixed(1)} KB/s`, `${info.network_up_kbps.toFixed(1)} KB/s`)
      : system.missing
  }

  if (text.includes('运行') || text.includes('uptime') || text.includes('多久')) {
    if (!info) return system.missing
    const hours = Math.floor(info.uptime_secs / 3600)
    const minutes = Math.floor((info.uptime_secs % 3600) / 60)
    return system.uptime(hours, minutes)
  }

  if (text.includes('你好') || text.includes('hi') || text.includes('hello')) {
    return chooseReply(personality.greetings, random)
  }

  if (text.includes('累') || text.includes('辛苦')) {
    return chooseReply(personality.supportiveReplies, random)
  }

  if (mood === 'warning' && info) {
    return info.cpu_usage > 80
      ? system.cpuWarning(`${info.cpu_usage.toFixed(1)}%`)
      : system.memoryWarning(`${info.memory_usage.toFixed(1)}%`)
  }

  if (mood === 'sleepy') return chooseReply(personality.sleepyReplies, random)

  const defaultReplies = [
    ...personality.defaultReplies,
    ...(info ? [system.cpu(`现在 ${info.cpu_usage.toFixed(1)}%`)] : ['正在获取系统信息...']),
  ]
  return chooseReply(defaultReplies, random)
}

/** 时间感知的角色问候 */
export function getGreeting(
  roleId: PetRoleId = DEFAULT_PET_ROLE,
  random: () => number = Math.random,
): string {
  const personality = getPetPersonality(roleId)
  return chooseReply(personality.greetings, random)
}

export function useChat() {
  const messages = ref<ChatMessage[]>([])
  const inputText = ref('')

  function sendMessage(info: SystemInfo | null, mood: PetMood, roleId: PetRoleId = DEFAULT_PET_ROLE) {
    const text = inputText.value.trim()
    if (!text) return

    messages.value.push({ text, isUser: true })
    messages.value.push({ text: generateReply(text, info, mood, roleId), isUser: false })
    inputText.value = ''

    if (messages.value.length > 20) messages.value = messages.value.slice(-20)
  }

  function addSystemMessage(text: string) {
    messages.value.push({ text, isUser: false })
  }

  function clearMessages() {
    messages.value = []
  }

  return { messages, inputText, sendMessage, addSystemMessage, clearMessages }
}
