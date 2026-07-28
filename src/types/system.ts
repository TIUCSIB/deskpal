/** 可用网络连接状态：已连接、未连接或当前平台无法确定。 */
export type NetworkConnectivity = true | false | null

/** 电池是否正在充电／已接通交流电；无电池或平台不支持时为空。 */
export type BatteryOnAcPower = true | false | null

/** Rust 端返回的实时系统信息结构。 */
export interface SystemInfo {
  cpu_usage: number
  memory_usage: number
  memory_used_mb: number
  memory_total_mb: number
  disk_usage: number
  network_down_kbps: number
  network_up_kbps: number
  network_connected: NetworkConnectivity
  battery_percent: number | null
  battery_charging: BatteryOnAcPower
  idle_seconds: number | null
  uptime_secs: number
}

/** 桌宠心情状态。 */
export type PetMood = 'happy' | 'normal' | 'sleepy' | 'warning'
