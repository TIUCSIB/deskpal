<script setup lang="ts">
/**
 * SettingsWindow.vue - 托盘设置窗口
 * 使用独立 composable 管理设置交互与窗口状态。
 */
import { useSettingsWindow } from '@/composables/useSettingsWindow'

const {
  settings,
  ready,
  scaleText,
  shortcutDraft,
  shortcutSummary,
  feedbackText,
  infoModeOptions,
  closeWindow,
  handleInfoModeChange,
  handleScaleChange,
  handleSizeLockedChange,
  handleShortcutEnabledChange,
  handleAlwaysOnTopChange,
  handleTaskbarChange,
  handleLaunchAtStartupChange,
  handleShortcutDraftInput,
  applyShortcut,
  restoreDefaultScale,
  resetPosition,
  resetSettingsWindowBounds,
  resetAllSettings,
} = useSettingsWindow()
</script>

<template>
  <main class="settings-window">
    <section v-if="ready" class="settings-window__panel" aria-label="设置">
      <header class="settings-window__header" data-tauri-drag-region>
        <div class="settings-window__heading">
          <h1 class="settings-window__title">设置</h1>
        </div>
        <button
          class="settings-window__close"
          type="button"
          title="关闭"
          data-tauri-drag-region="false"
          @click="closeWindow"
        >
          ×
        </button>
      </header>

      <div class="settings-window__content">
        <section class="settings-window__section">
          <div class="settings-window__section-title">显示</div>
          <label class="settings-window__field">
            <span class="settings-window__label">信息窗模式</span>
            <select class="settings-window__select" :value="settings.info_mode" @change="handleInfoModeChange">
              <option v-for="option in infoModeOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </label>
          <label class="settings-window__toggle">
            <input
              class="settings-window__checkbox"
              type="checkbox"
              :checked="settings.main_window_always_on_top"
              @change="handleAlwaysOnTopChange"
            />
            <span>桌宠窗口置顶</span>
          </label>
          <label class="settings-window__toggle">
            <input
              class="settings-window__checkbox"
              type="checkbox"
              :checked="settings.main_window_show_in_taskbar"
              @change="handleTaskbarChange"
            />
            <span>在任务栏显示</span>
          </label>
        </section>

        <section class="settings-window__section">
          <div class="settings-window__section-title">大小</div>
          <label class="settings-window__field">
            <div class="settings-window__row-header">
              <span class="settings-window__label">宠物缩放</span>
              <strong class="settings-window__scale-value">{{ scaleText }}</strong>
            </div>
            <input
              class="settings-window__range"
              type="range"
              min="0.45"
              max="1.2"
              step="0.05"
              :value="settings.pet_scale"
              @change="handleScaleChange"
            />
          </label>
          <label class="settings-window__toggle settings-window__toggle--top-gap">
            <input
              class="settings-window__checkbox"
              type="checkbox"
              :checked="settings.size_locked"
              @change="handleSizeLockedChange"
            />
            <span>锁定大小</span>
          </label>
          <div class="settings-window__actions settings-window__actions--inline">
            <button class="settings-window__button" type="button" @click="restoreDefaultScale">
              恢复默认大小
            </button>
          </div>
        </section>

        <section class="settings-window__section">
          <div class="settings-window__section-title">快捷键</div>
          <label class="settings-window__toggle">
            <input
              class="settings-window__checkbox"
              type="checkbox"
              :checked="settings.shortcut_enabled"
              @change="handleShortcutEnabledChange"
            />
            <span>启用聊天快捷键</span>
          </label>
          <label class="settings-window__field settings-window__field--top-gap">
            <span class="settings-window__label">快捷键组合</span>
            <div class="settings-window__shortcut-row">
              <input
                class="settings-window__input"
                type="text"
                :value="shortcutDraft"
                placeholder="例如 Ctrl+Alt+D"
                @input="handleShortcutDraftInput"
              />
              <button class="settings-window__button settings-window__button--primary" type="button" @click="applyShortcut">
                应用
              </button>
            </div>
          </label>
          <p class="settings-window__hint">
            支持写法示例：Ctrl+Alt+D、Shift+F1、Command+Option+D
          </p>
          <p class="settings-window__hint">{{ shortcutSummary }}</p>
        </section>

        <section class="settings-window__section">
          <div class="settings-window__section-title">系统</div>
          <label class="settings-window__toggle">
            <input
              class="settings-window__checkbox"
              type="checkbox"
              :checked="settings.launch_at_startup"
              @change="handleLaunchAtStartupChange"
            />
            <span>开机自动启动</span>
          </label>
          <div class="settings-window__actions settings-window__actions--wrap">
            <button class="settings-window__button" type="button" @click="resetPosition">
              重置桌宠位置
            </button>
            <button class="settings-window__button" type="button" @click="resetSettingsWindowBounds">
              重置设置窗口位置和大小
            </button>
            <button
              class="settings-window__button settings-window__button--danger"
              type="button"
              @click="resetAllSettings"
            >
              恢复全部默认设置
            </button>
          </div>
        </section>
      </div>

      <p v-if="feedbackText" class="settings-window__feedback">{{ feedbackText }}</p>
    </section>

    <div v-else class="settings-window__loading">正在载入设置…</div>
  </main>
</template>

<style scoped>
.settings-window {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  padding: 12px;
  background: transparent;
}

.settings-window__panel {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 14px;
  color: #1c1c1e;
  background: rgba(247, 247, 250, 0.98);
  border: 1px solid rgba(60, 60, 67, 0.12);
  border-radius: 18px;
}

.settings-window__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 2px 2px 0;
}

.settings-window__title {
  margin: 0;
  font-size: 22px;
  line-height: 1.1;
  font-weight: 700;
}

.settings-window__close {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  padding: 0;
  color: #636366;
  background: #ededf3;
  border: 0;
  border-radius: 50%;
  cursor: pointer;
  font-size: 22px;
  line-height: 1;
}

.settings-window__content {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
  display: grid;
  gap: 12px;
  padding-right: 4px;
}

.settings-window__content::-webkit-scrollbar {
  width: 8px;
}

.settings-window__content::-webkit-scrollbar-thumb {
  background: rgba(60, 60, 67, 0.18);
  border-radius: 999px;
}

.settings-window__section {
  padding: 14px;
  background: #fff;
  border: 1px solid #e5e5ea;
  border-radius: 14px;
}

.settings-window__section-title {
  margin-bottom: 12px;
  color: #8e8e93;
  font-size: 12px;
  line-height: 1;
}

.settings-window__field {
  display: grid;
  gap: 8px;
}

.settings-window__field--top-gap {
  margin-top: 12px;
}

.settings-window__label {
  color: #48484a;
  font-size: 14px;
  line-height: 1.2;
}

.settings-window__row-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.settings-window__select,
.settings-window__input {
  width: 100%;
  height: 40px;
  box-sizing: border-box;
  padding: 0 12px;
  color: #1c1c1e;
  background: #f7f7fa;
  border: 1px solid #d9d9df;
  border-radius: 12px;
  font-size: 14px;
}

.settings-window__shortcut-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
}

.settings-window__range {
  width: 100%;
}

.settings-window__scale-value {
  min-width: 48px;
  color: #3a3a3c;
  font-size: 15px;
  line-height: 1;
  text-align: right;
}

.settings-window__toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #1c1c1e;
  font-size: 14px;
  line-height: 1.3;
}

.settings-window__toggle + .settings-window__toggle {
  margin-top: 10px;
}

.settings-window__toggle--top-gap {
  margin-top: 12px;
}

.settings-window__checkbox {
  width: 16px;
  height: 16px;
  margin: 0;
  flex: none;
}

.settings-window__hint,
.settings-window__feedback {
  margin: 10px 0 0;
  font-size: 12px;
  line-height: 1.5;
}

.settings-window__hint {
  color: #8e8e93;
}

.settings-window__feedback {
  margin: 0;
  padding: 10px 12px;
  color: #3a3a3c;
  background: #f1f1f5;
  border-radius: 12px;
}

.settings-window__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

.settings-window__actions--inline {
  justify-content: flex-start;
}

.settings-window__actions--wrap {
  flex-wrap: wrap;
  justify-content: flex-start;
}

.settings-window__button {
  min-width: 96px;
  height: 36px;
  padding: 0 14px;
  color: #1c1c1e;
  background: #f2f2f7;
  border: 1px solid transparent;
  border-radius: 10px;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.settings-window__button--primary {
  color: #fff;
  background: #007aff;
}

.settings-window__button--danger {
  color: #fff;
  background: #ff3b30;
}

.settings-window__button:hover,
.settings-window__close:hover {
  background: #e6e6ec;
}

.settings-window__button--primary:hover {
  background: #0a6fe8;
}

.settings-window__button--danger:hover {
  background: #e0352b;
}

.settings-window__loading {
  width: 100%;
  padding: 18px 20px;
  color: #8e8e93;
  background: #ffffff;
  border: 1px solid rgba(60, 60, 67, 0.12);
  border-radius: 14px;
  font-size: 14px;
}
</style>
