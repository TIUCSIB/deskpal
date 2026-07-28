<script setup lang="ts">
/** ReminderEditor.vue - 提醒新增与编辑表单 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import ReminderWeekdayPicker from '@/components/settings/ReminderWeekdayPicker.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { ReminderDraft } from '@/composables/useReminderSettings'

const props = defineProps<{
  draft: ReminderDraft
  intervalOptions: number[]
  snoozeOptions: number[]
}>()

const emit = defineEmits<{
  'update:message': [string]
  'update:schedule-type': [ReminderDraft['scheduleType']]
  'update:interval-minutes': [number]
  'update:time': [string]
  'update:repeat-type': [ReminderDraft['repeatType']]
  'toggle:weekday': [number]
  'update:snooze-minutes': [number]
  save: []
  cancel: []
}>()

const MESSAGE_ID = 'reminder-editor-message'
const SCHEDULE_ID = 'reminder-editor-schedule'
const INTERVAL_ID = 'reminder-editor-interval'
const TIME_ID = 'reminder-editor-time'
const REPEAT_ID = 'reminder-editor-repeat'
const SNOOZE_ID = 'reminder-editor-snooze'
</script>

<template>
  <div class="grid gap-4 rounded-xl border border-primary/25 bg-primary/5 p-3">
    <div class="flex items-center justify-between gap-3">
      <strong class="text-sm font-medium text-foreground">{{ props.draft.id ? '编辑提醒' : '添加提醒' }}</strong>
      <span class="text-xs text-muted-foreground">{{ props.draft.id ? '修改后立即生效' : '保存后立即生效' }}</span>
    </div>

    <div class="grid gap-2">
      <Label :for="MESSAGE_ID" class="text-sm text-foreground">提醒文案</Label>
      <Input :id="MESSAGE_ID" :model-value="props.draft.message" class="h-10 rounded-xl bg-background/70" maxlength="80" placeholder="例如 记得喝水，起来活动一下吧" @update:model-value="emit('update:message', String($event))" />
    </div>

    <div class="grid gap-2">
      <Label :for="SCHEDULE_ID" class="text-sm text-foreground">提醒方式</Label>
      <Select :model-value="props.draft.scheduleType" @update:model-value="(value) => value && emit('update:schedule-type', value as ReminderDraft['scheduleType'])">
        <SelectTrigger :id="SCHEDULE_ID" class="h-10 rounded-xl bg-background/70"><SelectValue placeholder="请选择提醒方式" /></SelectTrigger>
        <SelectContent><SelectItem value="interval">间隔提醒</SelectItem><SelectItem value="fixed_time">固定时间</SelectItem></SelectContent>
      </Select>
    </div>

    <div v-if="props.draft.scheduleType === 'interval'" class="grid gap-2">
      <Label :for="INTERVAL_ID" class="text-sm text-foreground">提醒间隔</Label>
      <Select :model-value="String(props.draft.intervalMinutes)" @update:model-value="(value) => value && emit('update:interval-minutes', Number(value))">
        <SelectTrigger :id="INTERVAL_ID" class="h-10 rounded-xl bg-background/70"><SelectValue placeholder="请选择提醒间隔" /></SelectTrigger>
        <SelectContent><SelectItem v-for="option in props.intervalOptions" :key="option" :value="String(option)">每 {{ option }} 分钟</SelectItem></SelectContent>
      </Select>
    </div>

    <template v-else>
      <div class="grid gap-2">
        <Label :for="TIME_ID" class="text-sm text-foreground">提醒时间</Label>
        <Input :id="TIME_ID" type="time" :model-value="props.draft.time" class="h-10 rounded-xl bg-background/70" @update:model-value="emit('update:time', String($event))" />
      </div>
      <div class="grid gap-2">
        <Label :for="REPEAT_ID" class="text-sm text-foreground">重复日期</Label>
        <Select :model-value="props.draft.repeatType" @update:model-value="(value) => value && emit('update:repeat-type', value as ReminderDraft['repeatType'])">
          <SelectTrigger :id="REPEAT_ID" class="h-10 rounded-xl bg-background/70"><SelectValue placeholder="请选择重复日期" /></SelectTrigger>
          <SelectContent><SelectItem value="daily">每天</SelectItem><SelectItem value="weekdays">工作日</SelectItem><SelectItem value="custom_weekdays">自定义</SelectItem></SelectContent>
        </Select>
      </div>
      <ReminderWeekdayPicker v-if="props.draft.repeatType === 'custom_weekdays'" :model-value="props.draft.weekdays" @toggle="emit('toggle:weekday', $event)" />
    </template>

    <div class="grid gap-2">
      <Label :for="SNOOZE_ID" class="text-sm text-foreground">稍后提醒时长</Label>
      <Select :model-value="String(props.draft.snoozeMinutes)" @update:model-value="(value) => value && emit('update:snooze-minutes', Number(value))">
        <SelectTrigger :id="SNOOZE_ID" class="h-10 rounded-xl bg-background/70"><SelectValue placeholder="请选择稍后提醒时长" /></SelectTrigger>
        <SelectContent><SelectItem v-for="option in props.snoozeOptions" :key="option" :value="String(option)">{{ option }} 分钟后</SelectItem></SelectContent>
      </Select>
    </div>

    <div class="flex flex-wrap justify-end gap-2"><Button variant="ghost" class="rounded-xl" @click="emit('cancel')">取消</Button><Button class="rounded-xl" @click="emit('save')">保存提醒</Button></div>
  </div>
</template>
