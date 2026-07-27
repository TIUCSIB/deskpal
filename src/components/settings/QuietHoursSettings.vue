<script setup lang="ts">
/** QuietHoursSettings.vue - 提醒免打扰时段设置 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import SettingsToggleRow from '@/components/settings/SettingsToggleRow.vue'
import type { QuietHours } from '@/types/settings'

const props = defineProps<{ draft: QuietHours }>()
const emit = defineEmits<{
  'update:enabled': [boolean]
  'update:start': [string]
  'update:end': [string]
  save: []
}>()
</script>

<template>
  <section class="quiet-hours">
    <SettingsToggleRow
      id="reminder-quiet-hours"
      label="免打扰时段"
      description="在此期间不弹出提醒。"
      :checked="props.draft.enabled"
      @update:checked="emit('update:enabled', $event)"
    />
    <div v-if="props.draft.enabled" class="quiet-hours__times">
      <div class="grid gap-2">
        <Label for="quiet-hours-start">开始时间</Label>
        <Input id="quiet-hours-start" type="time" :model-value="props.draft.start" @update:model-value="emit('update:start', String($event))" />
      </div>
      <div class="grid gap-2">
        <Label for="quiet-hours-end">结束时间</Label>
        <Input id="quiet-hours-end" type="time" :model-value="props.draft.end" @update:model-value="emit('update:end', String($event))" />
      </div>
    </div>
    <Button variant="outline" size="sm" class="rounded-lg" @click="emit('save')">保存免打扰设置</Button>
  </section>
</template>

<style scoped>
.quiet-hours { display: grid; gap: 12px; }
.quiet-hours__times { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
</style>
