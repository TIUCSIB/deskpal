<script setup lang="ts">
/** InfoWindow.vue - 独立系统信息悬浮窗口 */
import InfoPanel from '@/components/InfoPanel.vue'
import { usePetContextReceiver } from '@/composables/useWindowBridge'

const { context } = usePetContextReceiver()
</script>

<template>
  <main class="info-window">
    <InfoPanel :info="context.info" />
  </main>
</template>

<style scoped>
.info-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  padding: 6px;
  background: transparent;
}
</style>
