import type { QuietHours } from '@/types/settings'
import type { SystemInfo } from '@/types/system'
import type { SystemFeedbackKind, SystemFeedbackPayload } from '@/types/systemFeedback'

const CPU_HIGH_THRESHOLD = 85
const CPU_RECOVERY_THRESHOLD = 75
const MEMORY_HIGH_THRESHOLD = 90
const MEMORY_RECOVERY_THRESHOLD = 80
const HIGH_LOAD_DURATION_MS = 3 * 60 * 1000
const HIGH_LOAD_COOLDOWN_MS = 30 * 60 * 1000
const EVENT_COOLDOWN_MS = 2 * 60 * 60 * 1000
const LOW_BATTERY_THRESHOLD = 20
const BATTERY_RECOVERY_THRESHOLD = 25
const LONG_IDLE_SECONDS = 60 * 60
const IDLE_RECOVERY_SECONDS = 10 * 60

interface SustainedSignal {
  startedAt: number | null
  active: boolean
}

function createSignal(): SustainedSignal {
  return { startedAt: null, active: false }
}

/** 判断当前时间是否处于跨日或同日的免打扰时段内。 */
export function isWithinQuietHours(quietHours: QuietHours, now: Date): boolean {
  if (!quietHours.enabled || quietHours.start === quietHours.end) return false
  const minutes = now.getHours() * 60 + now.getMinutes()
  const [startHour, startMinute] = quietHours.start.split(':').map(Number)
  const [endHour, endMinute] = quietHours.end.split(':').map(Number)
  if (![startHour, startMinute, endHour, endMinute].every(Number.isFinite)) return false
  const start = startHour * 60 + startMinute
  const end = endHour * 60 + endMinute
  return start < end ? minutes >= start && minutes < end : minutes >= start || minutes < end
}

/** useSystemFeedback - 以持续时长、冷却和免打扰约束主动系统反馈 */
export function useSystemFeedback(now: () => number = Date.now) {
  const loadSignals: Record<'cpu_high' | 'memory_high', SustainedSignal> = {
    cpu_high: createSignal(),
    memory_high: createSignal(),
  }
  const lastNotifiedAt = new Map<SystemFeedbackKind, number>()
  let previousNetworkConnected: boolean | null = null
  let batteryLow = false
  let idleLong = false

  function canNotify(kind: SystemFeedbackKind, timestamp: number, cooldown: number) {
    const last = lastNotifiedAt.get(kind)
    return last === undefined || timestamp - last >= cooldown
  }

  function createPayload(
    kind: SystemFeedbackKind,
    title: string,
    message: string,
    timestamp: number,
    cooldown: number,
    allowed: boolean,
  ): SystemFeedbackPayload | null {
    if (!allowed || !canNotify(kind, timestamp, cooldown)) return null
    lastNotifiedAt.set(kind, timestamp)
    return {
      id: `${kind}-${timestamp}`,
      kind,
      severity: kind === 'cpu_high' || kind === 'memory_high' || kind === 'battery_low' ? 'warning' : 'notice',
      title,
      message,
      occurred_at: timestamp,
    }
  }

  function evaluateLoad(
    kind: 'cpu_high' | 'memory_high',
    value: number,
    highThreshold: number,
    recoveryThreshold: number,
    timestamp: number,
    allowed: boolean,
  ): SystemFeedbackPayload | null {
    const signal = loadSignals[kind]
    if (value >= highThreshold) {
      signal.startedAt ??= timestamp
      signal.active = true
      if (timestamp - signal.startedAt < HIGH_LOAD_DURATION_MS) return null
      const label = kind === 'cpu_high' ? 'CPU' : '内存'
      return createPayload(
        kind,
        `${label} 持续高负载`,
        `当前${label}占用 ${value.toFixed(1)}%，已持续约 3 分钟。`,
        timestamp,
        HIGH_LOAD_COOLDOWN_MS,
        allowed,
      )
    }
    if (value <= recoveryThreshold) {
      signal.startedAt = null
      signal.active = false
    }
    return null
  }

  /** 处理一帧系统信息；返回本次可展示的单条反馈。 */
  function evaluate(info: SystemInfo, quietHours: QuietHours): SystemFeedbackPayload | null {
    const timestamp = now()
    const quiet = isWithinQuietHours(quietHours, new Date(timestamp))
    const candidates = [
      evaluateLoad('cpu_high', info.cpu_usage, CPU_HIGH_THRESHOLD, CPU_RECOVERY_THRESHOLD, timestamp, !quiet),
      evaluateLoad('memory_high', info.memory_usage, MEMORY_HIGH_THRESHOLD, MEMORY_RECOVERY_THRESHOLD, timestamp, !quiet),
      evaluateNetwork(info, timestamp, !quiet),
      evaluateBattery(info, timestamp, !quiet),
      evaluateIdle(info, timestamp, !quiet),
    ]
    return candidates.find((candidate): candidate is SystemFeedbackPayload => candidate !== null) ?? null
  }

  function evaluateNetwork(info: SystemInfo, timestamp: number, allowed: boolean): SystemFeedbackPayload | null {
    if (info.network_connected === null) return null
    const restored = previousNetworkConnected === false && info.network_connected
    previousNetworkConnected = info.network_connected
    if (!restored) return null
    return createPayload('network_restored', '网络已恢复', '网络连接已恢复，可以继续正常使用啦。', timestamp, EVENT_COOLDOWN_MS, allowed)
  }

  function evaluateBattery(info: SystemInfo, timestamp: number, allowed: boolean): SystemFeedbackPayload | null {
    if (info.battery_percent === null || info.battery_charging !== false) {
      batteryLow = false
      return null
    }
    if (info.battery_percent <= LOW_BATTERY_THRESHOLD) batteryLow = true
    if (info.battery_percent >= BATTERY_RECOVERY_THRESHOLD) batteryLow = false
    if (!batteryLow) return null
    return createPayload('battery_low', '电量偏低', `当前电量 ${info.battery_percent}% ，记得接通电源。`, timestamp, EVENT_COOLDOWN_MS, allowed)
  }

  function evaluateIdle(info: SystemInfo, timestamp: number, allowed: boolean): SystemFeedbackPayload | null {
    if (info.idle_seconds === null) return null
    if (info.idle_seconds >= LONG_IDLE_SECONDS) idleLong = true
    if (info.idle_seconds <= IDLE_RECOVERY_SECONDS) idleLong = false
    if (!idleLong) return null
    return createPayload('idle_long', '已经很久没有操作了', '休息时别忘了活动一下，回来再继续陪我吧。', timestamp, EVENT_COOLDOWN_MS, allowed)
  }

  return { evaluate }
}
