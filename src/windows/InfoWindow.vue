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
  position: relative;
  width: 240px;
  height: 144px;
  display: flex;
  align-items: center;
  justify-content: center;
  transform: translateY(0) scale(var(--info-scale, 1));
  transform-origin: bottom center;
  opacity: 0;
  filter: drop-shadow(0 10px 22px rgba(0, 0, 0, 0.14));
  animation: info-bubble-pop 260ms cubic-bezier(0.2, 1.3, 0.32, 1) forwards;
}

.info-window__content::after {
  position: absolute;
  bottom: 0;
  left: 50%;
  width: 12px;
  height: 12px;
  background: rgba(255, 255, 255, 0.97);
  border-right: 1px solid rgba(60, 60, 67, 0.16);
  border-bottom: 1px solid rgba(60, 60, 67, 0.16);
  content: '';
  transform: translate(-50%, 5px) rotate(45deg) scale(0.92);
}

@keyframes info-bubble-pop {
  0% {
    opacity: 0;
    transform: translateY(12px) scale(calc(var(--info-scale, 1) * 0.86));
  }

  68% {
    opacity: 1;
    transform: translateY(-2px) scale(calc(var(--info-scale, 1) * 1.035));
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(var(--info-scale, 1));
  }
}
</style>
