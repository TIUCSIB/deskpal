<script setup lang="ts">
/** ReminderWeekdayPicker.vue - 固定提醒的自定义星期选择器 */
const props = defineProps<{ modelValue: number[] }>()
const emit = defineEmits<{ toggle: [weekday: number] }>()

const DAYS = [
  { value: 1, label: '一' }, { value: 2, label: '二' }, { value: 3, label: '三' }, { value: 4, label: '四' },
  { value: 5, label: '五' }, { value: 6, label: '六' }, { value: 7, label: '日' },
]
</script>

<template>
  <div class="weekday-picker" role="group" aria-label="选择提醒日期">
    <button
      v-for="day in DAYS"
      :key="day.value"
      class="weekday-picker__day"
      :class="{ 'weekday-picker__day--selected': props.modelValue.includes(day.value) }"
      type="button"
      :aria-pressed="props.modelValue.includes(day.value)"
      @click="emit('toggle', day.value)"
    >
      周{{ day.label }}
    </button>
  </div>
</template>

<style scoped>
.weekday-picker { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: 6px; }
.weekday-picker__day { height: 32px; border: 1px solid hsl(var(--border)); border-radius: 8px; background: hsl(var(--background)); color: hsl(var(--muted-foreground)); cursor: pointer; font-size: 12px; }
.weekday-picker__day--selected { border-color: hsl(var(--primary)); background: hsl(var(--primary)); color: hsl(var(--primary-foreground)); }
</style>
