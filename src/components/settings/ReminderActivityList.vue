<script setup lang="ts">
/** ReminderActivityList.vue - 最近提醒活动列表 */
import type { ReminderActivityEvent } from '@/types/settings'

const props = defineProps<{ events: ReminderActivityEvent[]; hasMoreEvents: boolean }>()
const emit = defineEmits<{ showAll: [] }>()

function formatTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function eventLabel(event: ReminderActivityEvent) {
  if (event.kind === 'shown') return '已触发'
  if (event.kind === 'completed') return '已完成'
  if (event.kind === 'snoozed') return '已推迟'
  if (event.kind === 'skipped') return '已跳过'
  if (event.kind === 'quiet_deferred') return '免打扰顺延'
  return event.kind
}
</script>

<template>
  <div v-if="props.events.length" class="activity-list">
    <article v-for="event in props.events" :key="event.id" class="activity-list__item">
      <div class="min-w-0">
        <strong>{{ event.message }}</strong>
        <span v-if="event.reason">{{ event.reason }}</span>
      </div>
      <div class="activity-list__meta">
        <span>{{ eventLabel(event) }}</span>
        <time :datetime="event.occurredAt">{{ formatTime(event.occurredAt) }}</time>
      </div>
    </article>
  </div>
  <p v-else class="activity-list__empty">暂无提醒活动记录。</p>
  <button v-if="props.hasMoreEvents" type="button" class="activity-list__more" @click="emit('showAll')">查看全部记录</button>
</template>

<style scoped>
.activity-list { display: grid; gap: 8px; }
.activity-list__item { display: flex; justify-content: space-between; gap: 10px; padding: 9px 10px; border: 1px solid hsl(var(--border)); border-radius: 10px; background: hsl(var(--background) / 0.55); }
.activity-list__item strong, .activity-list__item span { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.activity-list__item strong { font-size: 12px; }
.activity-list__item span { color: hsl(var(--muted-foreground)); font-size: 11px; }
.activity-list__meta { flex: 0 0 auto; text-align: right; }
.activity-list__empty { margin: 0; color: hsl(var(--muted-foreground)); font-size: 12px; text-align: center; }
.activity-list__more { border: 0; background: transparent; color: hsl(var(--primary)); cursor: pointer; font-size: 12px; }
</style>
