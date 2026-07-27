<script setup lang="ts">
/** ReminderSettingsSection.vue - 多提醒列表、预设与编辑入口 */
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import ReminderEditor from '@/components/settings/ReminderEditor.vue'
import ReminderListItem from '@/components/settings/ReminderListItem.vue'
import ReminderPresetGrid from '@/components/settings/ReminderPresetGrid.vue'
import SettingsActionRow from '@/components/settings/SettingsActionRow.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import type { ReminderDraft } from '@/composables/useReminderSettings'
import type { ReminderPreset } from '@/config/reminderPresets'
import type { Reminder, ReminderSchedule } from '@/types/settings'

const props = defineProps<{
  reminders: Reminder[]
  draft: ReminderDraft | null
  deleteTarget: Reminder | null
  intervalOptions: number[]
  snoozeOptions: number[]
  presets: ReminderPreset[]
  formatSchedule: (schedule: ReminderSchedule) => string
  formatPause: (reminder: Reminder) => string
}>()

const emit = defineEmits<{
  create: []
  createPreset: [ReminderPreset]
  edit: [Reminder]
  cancel: []
  save: []
  'update:draft-message': [string]
  'update:draft-schedule-type': [ReminderDraft['scheduleType']]
  'update:draft-interval-minutes': [number]
  'update:draft-time': [string]
  'update:draft-repeat-type': [ReminderDraft['repeatType']]
  'toggle:weekday': [number]
  'update:draft-snooze-minutes': [number]
  'update:enabled': [id: string, enabled: boolean]
  preview: [id: string]
  requestDelete: [Reminder]
  cancelDelete: []
  confirmDelete: []
}>()
</script>

<template>
  <SettingsSection title="提醒">
    <div class="flex items-center justify-between gap-3">
      <p class="m-0 text-sm leading-5 text-muted-foreground">可添加间隔或固定时间提醒。</p>
      <Button size="sm" class="shrink-0 rounded-xl" :disabled="Boolean(props.draft)" @click="emit('create')">添加提醒</Button>
    </div>
    <div v-if="!props.draft" class="grid gap-2"><p class="m-0 text-xs text-muted-foreground">一键添加常用提醒</p><ReminderPresetGrid :presets="props.presets" @select="emit('createPreset', $event)" /></div>

    <ReminderEditor
      v-if="props.draft"
      :draft="props.draft"
      :interval-options="props.intervalOptions"
      :snooze-options="props.snoozeOptions"
      @update:message="emit('update:draft-message', $event)"
      @update:schedule-type="emit('update:draft-schedule-type', $event)"
      @update:interval-minutes="emit('update:draft-interval-minutes', $event)"
      @update:time="emit('update:draft-time', $event)"
      @update:repeat-type="emit('update:draft-repeat-type', $event)"
      @toggle:weekday="emit('toggle:weekday', $event)"
      @update:snooze-minutes="emit('update:draft-snooze-minutes', $event)"
      @save="emit('save')"
      @cancel="emit('cancel')"
    />

    <div v-if="props.reminders.length" class="grid gap-2"><ReminderListItem v-for="reminder in props.reminders" :key="reminder.id" :reminder="reminder" :schedule-text="props.formatSchedule(reminder.schedule)" :pause-text="props.formatPause(reminder)" @update:enabled="emit('update:enabled', reminder.id, $event)" @edit="emit('edit', reminder)" @preview="emit('preview', reminder.id)" @delete="emit('requestDelete', reminder)" /></div>
    <div v-else class="rounded-xl border border-dashed border-border px-3 py-5 text-center text-sm text-muted-foreground">暂无提醒，添加一条开始使用。</div>
    <SettingsActionRow v-if="props.reminders.length && !props.draft" align="start"><Button variant="outline" class="rounded-xl" @click="emit('create')">添加另一条提醒</Button></SettingsActionRow>
  </SettingsSection>

  <AlertDialog :open="Boolean(props.deleteTarget)" @update:open="(open) => !open && emit('cancelDelete')">
    <AlertDialogContent><AlertDialogHeader><AlertDialogTitle>删除这条提醒吗？</AlertDialogTitle><AlertDialogDescription>{{ props.deleteTarget?.message }} 将不再触发，此操作无法撤销。</AlertDialogDescription></AlertDialogHeader><AlertDialogFooter><AlertDialogCancel @click="emit('cancelDelete')">取消</AlertDialogCancel><AlertDialogAction variant="destructive" @click="emit('confirmDelete')">删除</AlertDialogAction></AlertDialogFooter></AlertDialogContent>
  </AlertDialog>
</template>
