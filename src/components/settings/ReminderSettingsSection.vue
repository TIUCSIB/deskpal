<script setup lang="ts">
/**
 * ReminderSettingsSection.vue - 提醒设置分组
 * 提供提醒开关、文案、间隔和测试入口。
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import SettingsActionRow from '@/components/settings/SettingsActionRow.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import SettingsToggleRow from '@/components/settings/SettingsToggleRow.vue'

const props = defineProps<{
  enabled: boolean
  messageDraft: string
  intervalMinutes: number
  snoozeMinutes: number
  intervalOptions: number[]
  snoozeOptions: number[]
}>()

const emit = defineEmits<{
  'update:enabled': [boolean]
  'update:message-draft': [string]
  'apply-message': []
  'update:interval': [number]
  'update:snooze': [number]
  preview: []
}>()

const MESSAGE_ID = 'settings-reminder-message'
const INTERVAL_ID = 'settings-reminder-interval'
const SNOOZE_ID = 'settings-reminder-snooze'
</script>

<template>
  <SettingsSection title="提醒">
    <SettingsToggleRow
      id="settings-reminder-enabled"
      label="启用间隔提醒"
      description="提醒会以独立气泡形式显示在桌宠附近。"
      :checked="props.enabled"
      @update:checked="emit('update:enabled', $event)"
    />

    <div class="grid gap-2">
      <Label :for="MESSAGE_ID" class="text-sm leading-5 text-foreground">
        提醒文案
      </Label>
      <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
        <Input
          :id="MESSAGE_ID"
          :model-value="props.messageDraft"
          class="h-10 rounded-xl bg-background/70"
          placeholder="例如 记得喝水，起来活动一下吧"
          @update:model-value="(value) => emit('update:message-draft', String(value))"
        />
        <Button class="h-10 rounded-xl px-4" @click="emit('apply-message')">
          保存
        </Button>
      </div>
    </div>

    <div class="grid gap-2">
      <Label :for="INTERVAL_ID" class="text-sm leading-5 text-foreground">
        提醒间隔
      </Label>
      <Select :model-value="String(props.intervalMinutes)" @update:model-value="(value) => value && emit('update:interval', Number(value))">
        <SelectTrigger :id="INTERVAL_ID" class="h-10 w-full rounded-xl bg-background/70">
          <SelectValue placeholder="请选择提醒间隔" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="option in props.intervalOptions" :key="option" :value="String(option)">
            {{ option }} 分钟
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label :for="SNOOZE_ID" class="text-sm leading-5 text-foreground">
        稍后提醒时长
      </Label>
      <Select :model-value="String(props.snoozeMinutes)" @update:model-value="(value) => value && emit('update:snooze', Number(value))">
        <SelectTrigger :id="SNOOZE_ID" class="h-10 w-full rounded-xl bg-background/70">
          <SelectValue placeholder="请选择稍后提醒时长" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="option in props.snoozeOptions" :key="option" :value="String(option)">
            {{ option }} 分钟
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <SettingsActionRow align="start">
      <Button variant="outline" class="rounded-xl" @click="emit('preview')">
        测试提醒
      </Button>
    </SettingsActionRow>
  </SettingsSection>
</template>
