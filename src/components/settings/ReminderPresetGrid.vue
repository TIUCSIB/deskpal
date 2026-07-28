<script setup lang="ts">
/** ReminderPresetGrid.vue - 一键添加常用提醒 */
import type { Component } from 'vue'
import {
  BriefcaseBusinessIcon,
  CoffeeIcon,
  EyeIcon,
  GlassWaterIcon,
  PersonStandingIcon,
  TimerIcon,
} from '@lucide/vue'
import { Button } from '@/components/ui/button'
import type { ReminderPreset, ReminderPresetIcon } from '@/config/reminderPresets'

const props = defineProps<{ presets: ReminderPreset[]; disabled?: boolean }>()
const emit = defineEmits<{ select: [preset: ReminderPreset] }>()

const PRESET_ICONS: Record<ReminderPresetIcon, Component> = {
  water: GlassWaterIcon,
  activity: PersonStandingIcon,
  rest: CoffeeIcon,
  'eye-care': EyeIcon,
  'clock-out': BriefcaseBusinessIcon,
  pomodoro: TimerIcon,
}
</script>

<template>
  <div class="preset-grid">
    <Button
      v-for="preset in props.presets"
      :key="preset.id"
      variant="outline"
      class="preset-grid__item"
      :aria-label="`添加${preset.label}提醒`"
      :disabled="props.disabled"
      @click="emit('select', preset)"
    >
      <component :is="PRESET_ICONS[preset.icon]" class="preset-grid__icon" aria-hidden="true" />
      <span class="preset-grid__content">
        <strong>{{ preset.label }}</strong>
        <span>{{ preset.description }}</span>
      </span>
    </Button>
  </div>
</template>

<style scoped>
.preset-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.preset-grid__item { height: auto; min-height: 70px; display: flex; align-items: center; justify-content: flex-start; gap: 10px; padding: 10px; border-radius: 12px; text-align: left; white-space: normal; }
.preset-grid__icon { width: 20px; height: 20px; flex: 0 0 auto; color: hsl(var(--primary)); }
.preset-grid__content { display: grid; min-width: 0; gap: 3px; }
.preset-grid__content strong { font-size: 13px; }
.preset-grid__content span { color: hsl(var(--muted-foreground)); font-size: 11px; font-weight: 400; line-height: 1.35; }
</style>
