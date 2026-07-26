<script setup lang="ts">
/** InfoWindow.vue - 保持最小可读性的系统信息悬浮窗口 */
import { computed } from 'vue'
import InfoPanel from '@/components/InfoPanel.vue'
import { usePetContextReceiver } from '@/composables/useWindowBridge'

const { context } = usePetContextReceiver()
const INFO_MIN_SCALE = 0.78

const effectiveScale = computed(() => Math.max(context.value.scale, INFO_MIN_SCALE))
const compact = computed(() => context.value.scale < 0.72)
const contentStyle = computed(() => ({
  '--info-scale': `${effectiveScale.value}`,
}))
</script>

<template>
  <main class="info-window">
    <div class="info-window__content" :style="contentStyle">
      <InfoPanel :info="context.info" :compact="compact" />
    </div>
  </main>
</template>

<style scoped>
.info-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: transparent;
}

.info-window__content {
  width: 240px;
  height: 144px;
  display: flex;
  align-items: center;
  justify-content: center;
  transform: translateY(4px) scale(var(--info-scale, 1));
  transform-origin: center;
  opacity: 0;
  animation: info-window-in 180ms ease-out forwards;
}

@keyframes info-window-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(calc(var(--info-scale, 1) * 0.97));
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(var(--info-scale, 1));
  }
}
</style>
