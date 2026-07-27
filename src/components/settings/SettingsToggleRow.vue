<script setup lang="ts">
/**
 * SettingsToggleRow.vue - 设置开关行
 * 统一 Checkbox 与标签说明的排列方式。
 */
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'

const props = defineProps<{
  id: string
  label: string
  checked: boolean
  description?: string
}>()

const emit = defineEmits<{
  'update:checked': [boolean]
}>()

function handleUpdate(value: boolean | 'indeterminate') {
  emit('update:checked', value === true)
}
</script>

<template>
  <div class="flex items-start gap-3">
    <Checkbox
      :id="props.id"
      :model-value="props.checked"
      class="mt-0.5"
      @update:model-value="handleUpdate"
    />

    <div class="grid min-w-0 gap-1.5">
      <Label :for="props.id" class="cursor-pointer text-sm leading-5 font-medium text-foreground">
        {{ props.label }}
      </Label>
      <p v-if="props.description" class="text-xs leading-5 text-muted-foreground">
        {{ props.description }}
      </p>
    </div>
  </div>
</template>
