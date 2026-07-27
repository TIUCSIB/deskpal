<script setup lang="ts">
/** ReminderListItem.vue - 单条提醒设置行 */
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import type { Reminder } from '@/types/settings'

const props = defineProps<{
  reminder: Reminder
  scheduleText: string
  pauseText: string
}>()

const emit = defineEmits<{
  'update:enabled': [boolean]
  edit: []
  preview: []
  delete: []
}>()
</script>

<template>
  <article class="grid gap-3 rounded-xl border border-border/70 bg-background/55 p-3">
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0 grid gap-1">
        <strong class="truncate text-sm font-medium text-foreground">{{ props.reminder.message }}</strong>
        <span class="text-xs text-muted-foreground">{{ props.scheduleText }}</span>
        <span v-if="props.pauseText" class="text-xs text-amber-600 dark:text-amber-400">
          {{ props.pauseText }}
        </span>
      </div>
      <Checkbox
        :model-value="props.reminder.enabled"
        :aria-label="`${props.reminder.message}提醒开关`"
        @update:model-value="emit('update:enabled', Boolean($event))"
      />
    </div>

    <div class="flex flex-wrap gap-2">
      <Button variant="outline" size="sm" class="rounded-lg" @click="emit('edit')">编辑</Button>
      <Button variant="outline" size="sm" class="rounded-lg" @click="emit('preview')">测试</Button>
      <Button variant="ghost" size="sm" class="rounded-lg text-destructive hover:text-destructive" @click="emit('delete')">
        删除
      </Button>
    </div>
  </article>
</template>
