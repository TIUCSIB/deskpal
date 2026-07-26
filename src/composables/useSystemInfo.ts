/** useSystemInfo.ts - 不重叠的实时系统信息轮询 */
import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SystemInfo } from '@/types/system'

const POLL_INTERVAL = 1000

export function useSystemInfo() {
  const info = ref<SystemInfo | null>(null)
  const error = ref<string | null>(null)
  let timer: ReturnType<typeof setTimeout> | null = null
  let stopped = false

  /** 完成当前请求后再安排下一次，避免响应乱序 */
  async function poll() {
    try {
      const snapshot = await invoke<SystemInfo>('get_system_info')
      if (!stopped) info.value = snapshot
      error.value = null
    } catch (caught) {
      if (!stopped) error.value = String(caught)
    } finally {
      if (!stopped) timer = setTimeout(poll, POLL_INTERVAL)
    }
  }

  onMounted(() => {
    stopped = false
    void poll()
  })

  onUnmounted(() => {
    stopped = true
    if (timer) clearTimeout(timer)
  })

  return { info, error }
}
