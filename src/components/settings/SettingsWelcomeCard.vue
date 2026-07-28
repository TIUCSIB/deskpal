<script setup lang="ts">
/** SettingsWelcomeCard.vue - 首次使用的设置提示卡。 */
import { Button } from '@/components/ui/button'

defineEmits<{ complete: [] }>()
</script>

<template>
  <section class="settings-welcome" aria-label="使用提示">
    <div>
      <strong>欢迎使用 DeskPal</strong>
      <p>可拖拽桌宠移动位置，点击打开聊天，悬停可互动；提醒和系统反馈会遵守免打扰时间。自定义角色仅支持受限资源包。</p>
    </div>
    <Button size="sm" class="rounded-xl" @click="$emit('complete')">知道了</Button>
  </section>
</template>

<style scoped>
.settings-welcome {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  background: color-mix(in srgb, var(--primary) 10%, var(--background));
  border: 1px solid color-mix(in srgb, var(--primary) 30%, var(--border));
  border-radius: 14px;
}

.settings-welcome strong {
  font-size: 14px;
  line-height: 20px;
}

.settings-welcome p {
  margin: 3px 0 0;
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 18px;
}
</style>
