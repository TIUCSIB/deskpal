<script setup lang="ts">
/** ReminderPresetGrid.vue - 一键添加常用提醒 */
import { Button } from '@/components/ui/button'
import type { ReminderPreset } from '@/config/reminderPresets'

const props = defineProps<{ presets: ReminderPreset[]; disabled?: boolean }>()
const emit = defineEmits<{ select: [preset: ReminderPreset] }>()
</script>

<template>
  <div class="preset-grid">
    <Button
      v-for="preset in props.presets"
      :key="preset.id"
      variant="outline"
      class="preset-grid__item"
      :disabled="props.disabled"
      @click="emit('select', preset)"
    >
      <strong>{{ preset.label }}</strong>
      <span>{{ preset.description }}</span>
    </Button>
  </div>
</template>

<style scoped>
.preset-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.preset-grid__item { height: auto; min-height: 62px; display: grid; justify-items: start; gap: 3px; padding: 10px; border-radius: 12px; text-align: left; white-space: normal; }
.preset-grid__item span { color: hsl(var(--muted-foreground)); font-size: 11px; font-weight: 400; line-height: 1.35; }
</style>
