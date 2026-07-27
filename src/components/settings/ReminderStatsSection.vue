<script setup lang="ts">
/** ReminderStatsSection.vue - 提醒完成统计 */
import type { ReminderActivityStats } from '@/types/settings'

const props = defineProps<{ stats: ReminderActivityStats }>()

function formatCompletionRate(rate: number | null) {
  if (rate === null) return '暂无数据'
  return `${Math.round(rate <= 1 ? rate * 100 : rate)}%`
}
</script>

<template>
  <section class="reminder-stats" aria-label="提醒统计">
    <div class="reminder-stats__metric">
      <span>今日完成率</span>
      <strong>{{ formatCompletionRate(props.stats.todayCompletionRate) }}</strong>
    </div>
    <div class="reminder-stats__metric">
      <span>连续完成</span>
      <strong>{{ props.stats.currentStreakDays }} 天</strong>
    </div>
    <div v-if="props.stats.frequentlyPostponed.length" class="reminder-stats__postponed">
      <span>常被推迟</span>
      <p v-for="item in props.stats.frequentlyPostponed" :key="item.reminderId">
        {{ item.message }}（{{ item.snoozeCount }} 次）
      </p>
    </div>
  </section>
</template>

<style scoped>
.reminder-stats { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.reminder-stats__metric, .reminder-stats__postponed { display: grid; gap: 3px; padding: 10px; border: 1px solid hsl(var(--border)); border-radius: 12px; background: hsl(var(--background) / 0.55); }
.reminder-stats__metric span, .reminder-stats__postponed span { color: hsl(var(--muted-foreground)); font-size: 11px; }
.reminder-stats__metric strong { font-size: 16px; }
.reminder-stats__postponed { grid-column: 1 / -1; }
.reminder-stats__postponed p { margin: 0; overflow: hidden; font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
</style>
