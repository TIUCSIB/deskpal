import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SystemInfo } from '@/types/system'

const POLL_INTERVAL = 2000 // 2 秒轮询一次

export function useSystemInfo() {
  const info = ref<SystemInfo | null>(null)
  const error = ref<string | null>(null)
  let timer: ReturnType<typeof setInterval> | null = null

  async function fetchInfo() {
    try {
      info.value = await invoke<SystemInfo>('get_system_info')
      error.value = null
    } catch (e) {
      error.value = String(e)
    }
  }

  onMounted(() => {
    fetchInfo()
    timer = setInterval(fetchInfo, POLL_INTERVAL)
  })

  onUnmounted(() => {
    if (timer) clearInterval(timer)
  })

  return { info, error }
}
