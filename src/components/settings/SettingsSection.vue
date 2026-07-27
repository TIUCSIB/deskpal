<script setup lang="ts">
/**
 * SettingsSection.vue - 设置分组卡片
 * 提供统一的分组标题和内容间距。
 */
const props = defineProps<{
  title: string
}>()
</script>

<template>
  <section class="rounded-2xl border border-border bg-card/95 p-4">
    <div class="mb-3 text-xs leading-none text-muted-foreground">
      {{ props.title }}
    </div>
    <div class="grid gap-3">
      <slot />
    </div>
  </section>
</template>
