/** Rust 端返回的系统信息结构 */
export interface SystemInfo {
  cpu_usage: number
  memory_usage: number
  memory_used_mb: number
  memory_total_mb: number
  disk_usage: number
  uptime_secs: number
}

/** 桌宠心情状态 */
export type PetMood = 'happy' | 'normal' | 'sleepy' | 'warning'
