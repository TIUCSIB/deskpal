import { ref } from 'vue'
import type { SystemInfo, PetMood } from '@/types/system'

/** 根据系统指标和小时数推导宠物心情 */
export function derivePetMood(info: SystemInfo, hour: number): PetMood {
  if (info.cpu_usage > 80 || info.memory_usage > 85) return 'warning'
  if (hour >= 0 && hour < 6) return 'sleepy'
  if (info.cpu_usage < 30 && info.memory_usage < 50) return 'happy'
  return 'normal'
}

/** usePetState - 根据系统状态维护宠物心情 */
export function usePetState() {
  const mood = ref<PetMood>('normal')

  /** 根据当前系统信息更新心情 */
  function updateMood(info: SystemInfo | null) {
    if (!info) return
    mood.value = derivePetMood(info, new Date().getHours())
  }

  return { mood, updateMood }
}
