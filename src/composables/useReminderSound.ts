import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WINDOW_EVENTS } from '@/types/window'

/** useReminderSound - 新提醒展示时播放短促提示音 */
export function useReminderSound() {
  let unlisten: UnlistenFn | null = null
  let audioContext: AudioContext | null = null

  function play() {
    try {
      audioContext ??= new AudioContext()
      const oscillator = audioContext.createOscillator()
      const gain = audioContext.createGain()
      const now = audioContext.currentTime

      oscillator.type = 'sine'
      oscillator.frequency.setValueAtTime(620, now)
      oscillator.frequency.exponentialRampToValueAtTime(900, now + 0.11)
      gain.gain.setValueAtTime(0.0001, now)
      gain.gain.exponentialRampToValueAtTime(0.12, now + 0.015)
      gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.15)
      oscillator.connect(gain).connect(audioContext.destination)
      oscillator.start(now)
      oscillator.stop(now + 0.16)
    } catch {
      // 音频设备不可用时不能影响提醒窗口展示。
    }
  }

  async function start() {
    if (unlisten) return
    unlisten = await listen(WINDOW_EVENTS.reminderTriggered, play)
  }

  function dispose() {
    unlisten?.()
    unlisten = null
    void audioContext?.close()
    audioContext = null
  }

  onMounted(() => { void start() })
  onUnmounted(dispose)

  return { start, dispose, play }
}
