/** reminderPresets.ts - 常用提醒预设 */
import type { ReminderInput } from '@/types/settings'

export interface ReminderPreset {
  id: string
  label: string
  description: string
  input: ReminderInput
}

export const REMINDER_PRESETS: ReminderPreset[] = [
  {
    id: 'water',
    label: '喝水',
    description: '每 30 分钟补充水分',
    input: { message: '记得喝水，补充一点水分吧', schedule: { type: 'interval', interval_minutes: 30 }, snooze_minutes: 5 },
  },
  {
    id: 'sedentary',
    label: '久坐活动',
    description: '每 60 分钟起来活动',
    input: { message: '坐了一会儿，起来活动一下吧', schedule: { type: 'interval', interval_minutes: 60 }, snooze_minutes: 10 },
  },
  {
    id: 'rest',
    label: '休息',
    description: '每 90 分钟放松片刻',
    input: { message: '该休息一会儿了，放松一下吧', schedule: { type: 'interval', interval_minutes: 90 }, snooze_minutes: 10 },
  },
  {
    id: 'eye-care',
    label: '护眼',
    description: '每 45 分钟远眺护眼',
    input: { message: '看看远处，让眼睛休息一下', schedule: { type: 'interval', interval_minutes: 45 }, snooze_minutes: 5 },
  },
  {
    id: 'clock-out',
    label: '下班打卡',
    description: '工作日 18:00 提醒',
    input: { message: '到下班时间了，别忘了打卡', schedule: { type: 'fixed_time', time: '18:00', repeat: { type: 'weekdays' } }, snooze_minutes: 10 },
  },
  {
    id: 'pomodoro',
    label: '番茄钟',
    description: '每 25 分钟专注休息',
    input: { message: '一个番茄钟完成了，休息一下吧', schedule: { type: 'interval', interval_minutes: 25 }, snooze_minutes: 5 },
  },
]
