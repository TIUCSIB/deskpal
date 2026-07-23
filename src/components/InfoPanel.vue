<script setup lang="ts">
/**
 * InfoPanel.vue - 系统信息面板
 * 显示 CPU / 内存 / 磁盘使用率和运行时间
 */
import { computed } from 'vue'
import type { SystemInfo } from '../types/system'

const props = defineProps<{ info: SystemInfo | null }>()

/** 格式化运行时间 */
const uptimeText = computed(() => {
  if (!props.info) return '--'
  const h = Math.floor(props.info.uptime_secs / 3600)
  const m = Math.floor((props.info.uptime_secs % 3600) / 60)
  return `${h}h ${m}m`
})

/** 进度条颜色 */
function barColor(usage: number): string {
  if (usage > 85) return '#e53935'
  if (usage > 60) return '#fb8c00'
  return '#43a047'
}
</script>

<template>
  <div class="info-panel" v-if="info">
    <div class="info-panel__row">
      <span class="info-panel__icon">🖥️</span>
      <span class="info-panel__label">CPU</span>
      <div class="info-panel__bar">
        <div
          class="info-panel__fill"
          :style="{ width: info.cpu_usage + '%', background: barColor(info.cpu_usage) }"
        ></div>
      </div>
      <span class="info-panel__value">{{ info.cpu_usage.toFixed(1) }}%</span>
    </div>

    <div class="info-panel__row">
      <span class="info-panel__icon">🧠</span>
      <span class="info-panel__label">MEM</span>
      <div class="info-panel__bar">
        <div
          class="info-panel__fill"
          :style="{ width: info.memory_usage + '%', background: barColor(info.memory_usage) }"
        ></div>
      </div>
      <span class="info-panel__value">{{ info.memory_usage.toFixed(1) }}%</span>
    </div>

    <div class="info-panel__row">
      <span class="info-panel__icon">💾</span>
      <span class="info-panel__label">DISK</span>
      <div class="info-panel__bar">
        <div
          class="info-panel__fill"
          :style="{ width: info.disk_usage + '%', background: barColor(info.disk_usage) }"
        ></div>
      </div>
      <span class="info-panel__value">{{ info.disk_usage.toFixed(1) }}%</span>
    </div>

    <div class="info-panel__row info-panel__row--uptime">
      <span class="info-panel__icon">⏱️</span>
      <span class="info-panel__label">运行</span>
      <span class="info-panel__value info-panel__value--wide">{{ uptimeText }}</span>
    </div>
  </div>
</template>

<style scoped>
.info-panel {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
  background: rgba(30, 30, 30, 0.92);
  border-radius: 12px;
  padding: 10px 14px;
  min-width: 200px;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  animation: slide-up 0.2s ease-out;
}

.info-panel__row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}
.info-panel__row:last-child {
  margin-bottom: 0;
}
.info-panel__row--uptime {
  margin-top: 4px;
  padding-top: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.info-panel__icon {
  font-size: 12px;
  width: 18px;
  text-align: center;
}
.info-panel__label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  width: 32px;
  font-family: 'Consolas', 'Courier New', monospace;
}
.info-panel__bar {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}
.info-panel__fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.5s ease, background 0.5s ease;
}
.info-panel__value {
  font-size: 11px;
  color: #fff;
  width: 42px;
  text-align: right;
  font-family: 'Consolas', 'Courier New', monospace;
}
.info-panel__value--wide {
  flex: 1;
  text-align: right;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateX(-50%) translateY(6px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
