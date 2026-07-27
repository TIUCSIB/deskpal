import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import type { ReminderActivity } from '@/types/settings'

const EMPTY_ACTIVITY: ReminderActivity = {
  stats: { todayCompletionRate: null, currentStreakDays: 0, frequentlyPostponed: [] },
  events: [],
  hasMoreEvents: false,
}

/** useReminderActivity - 提醒完成与推迟记录 */
export function useReminderActivity() {
  const activity = ref<ReminderActivity>({ ...EMPTY_ACTIVITY })
  const loading = ref(false)
  let unlisten: UnlistenFn | null = null
  let disposed = false

  async function loadActivity(includeAllEvents = false) {
    loading.value = true
    try {
      activity.value = await invoke<ReminderActivity>('get_reminder_activity', { includeAllEvents })
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '提醒活动记录加载失败')
    } finally {
      loading.value = false
    }
  }

  async function clearActivity() {
    try {
      await invoke('clear_reminder_activity')
      await loadActivity()
      toast('提醒活动记录已清除')
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : '清除提醒活动记录失败')
    }
  }

  onMounted(async () => {
    disposed = false
    unlisten = await listen('pet://reminder-activity-updated', () => {
      void loadActivity()
    })
    if (disposed) unlisten?.()
    else void loadActivity()
  })

  onUnmounted(() => {
    disposed = true
    unlisten?.()
  })

  return { activity, loading, loadActivity, clearActivity }
}
