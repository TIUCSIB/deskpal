import { ref } from 'vue'
import type { SystemInfo, PetMood } from '@/types/system'

/**
 * 桌宠状态管理
 * 根据系统指标和时间自动切换心情
 */
export function usePetState() {
  const mood = ref<PetMood>('normal')

  /** 根据系统信息更新心情 */
  function updateMood(info: SystemInfo | null) {
    if (!info) return

    // 系统压力大 → warning
    if (info.cpu_usage > 80 || info.memory_usage > 85) {
      mood.value = 'warning'
      return
    }

    // 深夜时段 → sleepy
    const hour = new Date().getHours()
    if (hour >= 0 && hour < 6) {
      mood.value = 'sleepy'
      return
    }

    // 轻度负载 → happy
    if (info.cpu_usage < 30 && info.memory_usage < 50) {
      mood.value = 'happy'
      return
    }

    // 其他情况 → normal
    mood.value = 'normal'
  }

  return { mood, updateMood }
}
