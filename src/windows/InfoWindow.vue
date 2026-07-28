<script setup lang="ts">
/** InfoWindow.vue - 保持最小可读性的系统信息悬浮窗口 */
import { computed } from 'vue'
import InfoPanel from '@/components/InfoPanel.vue'
import { useOverlayTransition } from '@/composables/useOverlayTransition'
import { usePetContextReceiver } from '@/composables/useWindowBridge'

const { context } = usePetContextReceiver('info')
const { revision, transitionStyle } = useOverlayTransition()
const INFO_MIN_SCALE = 0.78

const INFO_CONTENT_HEIGHT = 158

const effectiveScale = computed(() => Math.max(context.value.scale, INFO_MIN_SCALE))
const compact = computed(() => context.value.scale < 0.72)
const contentStyle = computed(() => ({
  ...transitionStyle.value,
  '--info-scale': `${effectiveScale.value}`,
  '--info-content-width': `${232 * effectiveScale.value}px`,
  '--info-content-height': `${INFO_CONTENT_HEIGHT * effectiveScale.value}px`,
}))
</script>

<template>
  <main class="info-window">
    <div class="info-window__content" :style="contentStyle">
      <div
        class="info-window__panel"
        :class="revision % 2 === 0 ? 'info-window__panel--enter-a' : 'info-window__panel--enter-b'"
      >
        <InfoPanel
          :info="context.info"
          :interaction-text="context.interactionText"
          :compact="compact"
        />
      </div>
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
  background: transparent;
}

.info-window__content {
  width: var(--info-content-width, 232px);
  height: var(--info-content-height, 158px);
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
}

.info-window__panel {
  width: 232px;
  height: 158px;
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  transform: scale(var(--info-scale, 1));
  transform-origin: center;
}

.info-window__panel--enter-a {
  animation: overlay-enter-a 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both;
}

.info-window__panel--enter-b {
  animation: overlay-enter-b 200ms cubic-bezier(0.2, 1.15, 0.32, 1) both;
}

@keyframes overlay-enter-a {
  from {
    opacity: 0;
    transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(calc(var(--info-scale, 1) * 0.94));
  }

  to {
    opacity: 1;
    transform: translate(0) scale(var(--info-scale, 1));
  }
}

@keyframes overlay-enter-b {
  from {
    opacity: 0;
    transform: translate(var(--overlay-enter-x, 0), var(--overlay-enter-y, 8px)) scale(calc(var(--info-scale, 1) * 0.94));
  }

  to {
    opacity: 1;
    transform: translate(0) scale(var(--info-scale, 1));
  }
}

@media (prefers-reduced-motion: reduce) {
  .info-window__panel {
    animation-duration: 1ms;
  }
}
</style>
