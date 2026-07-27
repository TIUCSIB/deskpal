/** systemFeedback.ts - 主动系统反馈的会话类型 */

export type SystemFeedbackKind = 'cpu_high' | 'memory_high' | 'network_restored' | 'battery_low' | 'idle_long'
export type SystemFeedbackSeverity = 'warning' | 'notice'

export interface SystemFeedbackPayload {
  id: string
  kind: SystemFeedbackKind
  severity: SystemFeedbackSeverity
  title: string
  message: string
  occurred_at: number
}
