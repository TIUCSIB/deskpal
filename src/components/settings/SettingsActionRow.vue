<script setup lang="ts">
/**
 * SettingsActionRow.vue - 设置操作按钮容器
 * 统一按钮组的对齐与换行规则。
 */
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  wrap?: boolean
  align?: 'start' | 'end'
}>(), {
  wrap: false,
  align: 'end',
})
</script>

<template>
  <div
    :class="cn(
      'mt-3 flex gap-2',
      props.wrap ? 'flex-wrap' : '',
      props.align === 'start' ? 'justify-start' : 'justify-end',
    )"
  >
    <slot />
  </div>
</template>
