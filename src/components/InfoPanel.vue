<script setup lang="ts">
/** InfoPanel.vue - 白色实时系统信息面板 */
import { computed } from 'vue'
import type { SystemInfo } from '@/types/system'

const props = withDefaults(
  defineProps<{
    info: SystemInfo | null
    compact?: boolean
  }>(),
  { compact: false },
)

const uptimeText = computed(() => {
  if (!props.info) return '--'
  const hours = Math.floor(props.info.uptime_secs / 3600)
  const minutes = Math.floor((props.info.uptime_secs % 3600) / 60)
  const seconds = props.info.uptime_secs % 60
  return `${hours}h ${minutes}m ${seconds}s`
})

const networkText = computed(() => {
  if (!props.info) return '--'
  return `↓ ${props.info.network_down_kbps.toFixed(1)} / ↑ ${props.info.network_up_kbps.toFixed(1)} KB/s`
})

function barColor(usage: number): string {
  if (usage > 85) return '#ff3b30'
  if (usage > 60) return '#ff9500'
  return '#34c759'
}

function safeUsage(usage: number): number {
  return Math.max(0, Math.min(100, usage))
}
</script>

<template>
  <section class="info-panel" :class="{ 'info-panel--compact': compact }" aria-label="实时系统状态">
    <template v-if="info">
      <div class="info-panel__row">
        <span class="info-panel__label">CPU</span>
        <div class="info-panel__bar">
          <i
            class="info-panel__fill"
            :style="{
              width: safeUsage(info.cpu_usage) + '%',
              backgroundColor: barColor(info.cpu_usage),
            }"
          ></i>
        </div>
        <span class="info-panel__value">{{ info.cpu_usage.toFixed(1) }}%</span>
      </div>

      <div class="info-panel__row">
        <span class="info-panel__label">内存</span>
        <div class="info-panel__bar">
          <i
            class="info-panel__fill"
            :style="{
              width: safeUsage(info.memory_usage) + '%',
              backgroundColor: barColor(info.memory_usage),
            }"
          ></i>
        </div>
        <span class="info-panel__value">{{ info.memory_usage.toFixed(1) }}%</span>
      </div>

      <div class="info-panel__row">
        <span class="info-panel__label">存储</span>
        <div class="info-panel__bar">
          <i
            class="info-panel__fill"
            :style="{
              width: safeUsage(info.disk_usage) + '%',
              backgroundColor: barColor(info.disk_usage),
            }"
          ></i>
        </div>
        <span class="info-panel__value">{{ info.disk_usage.toFixed(1) }}%</span>
      </div>

      <div class="info-panel__meta">
        <div class="info-panel__meta-item">
          <span class="info-panel__meta-label">网络</span>
          <strong class="info-panel__meta-value">{{ networkText }}</strong>
        </div>
      </div>

      <div class="info-panel__footer">
        <span>运行时间</span>
        <strong>{{ uptimeText }}</strong>
      </div>
    </template>

    <div v-else class="info-panel__loading">正在读取系统状态…</div>
  </section>
</template>

<style scoped>
.info-panel {
  box-sizing: border-box;
  width: 228px;
  min-height: 128px;
  padding: 10px 12px;
  color: #1c1c1e;
  background: rgba(255, 255, 255, 0.97);
  border: 1px solid rgba(60, 60, 67, 0.16);
  border-radius: 14px;
  font-variant-numeric: tabular-nums;
}

.info-panel--compact {
  padding: 9px 11px;
}

.info-panel__row {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) max-content;
  align-items: center;
  gap: 7px;
  min-width: 0;
  min-height: 20px;
}

.info-panel__row + .info-panel__row {
  margin-top: 4px;
}

.info-panel__label,
.info-panel__value {
  font-size: 11px;
  line-height: 1;
  letter-spacing: 0;
}

.info-panel__label {
  color: #636366;
}

.info-panel__value {
  min-width: max-content;
  color: #1c1c1e;
  text-align: right;
  font-weight: 600;
  white-space: nowrap;
}

.info-panel__bar {
  height: 7px;
  overflow: hidden;
  background: #e5e5ea;
  border-radius: 4px;
}

.info-panel__fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width 240ms ease, background-color 240ms ease;
}

.info-panel__meta {
  display: grid;
  gap: 4px;
  margin-top: 7px;
}

.info-panel__meta-item {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.info-panel__meta-label,
.info-panel__meta-value {
  font-size: 10px;
  line-height: 1.2;
}

.info-panel__meta-label {
  color: #8e8e93;
}

.info-panel__meta-value {
  color: #3a3a3c;
  font-weight: 600;
  text-align: right;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.info-panel__footer {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 8px;
  margin-top: 7px;
  padding-top: 7px;
  color: #8e8e93;
  border-top: 1px solid #e5e5ea;
  font-size: 10px;
  letter-spacing: 0;
}

.info-panel__footer strong {
  min-width: 0;
  overflow: hidden;
  color: #3a3a3c;
  font-weight: 600;
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-panel__loading {
  display: grid;
  min-height: 90px;
  place-items: center;
  color: #8e8e93;
  font-size: 11px;
}
</style>
