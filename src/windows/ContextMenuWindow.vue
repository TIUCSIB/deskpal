<script setup lang="ts">
/** ContextMenuWindow.vue - 桌宠主题化右键菜单。 */
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { CheckIcon, MessageCircleIcon, MonitorIcon, PauseIcon, PowerIcon, SettingsIcon } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { petRoles } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import type { PetRoleId } from '@/types/pet'
import { WINDOW_EVENTS } from '@/types/window'

const { settings, ready, loadSettings } = useAppSettings()
const menuRef = ref<HTMLElement | null>(null)
const selectedRole = computed(() => settings.value.pet_role)
let unlistenFocus: UnlistenFn | null = null
let unlistenWindowFocus: UnlistenFn | null = null
let focusedAt = 0
let hasFocused = false
const BLUR_HIDE_GUARD_MS = 160

async function hideMenu() {
  await invoke('hide_main_context_menu')
}

async function runAction(command: string) {
  try {
    await invoke(command)
    await hideMenu()
  } catch (error) {
    console.error('执行桌宠菜单操作失败:', error)
  }
}

async function selectRole(role: PetRoleId) {
  if (role === selectedRole.value) {
    await hideMenu()
    return
  }
  try {
    await invoke('set_pet_role', { role })
    await hideMenu()
  } catch (error) {
    console.error('切换桌宠角色失败:', error)
  }
}

async function focusMenu() {
  hasFocused = true
  focusedAt = Date.now()
  await nextTick()
  menuRef.value?.focus()
}

onMounted(async () => {
  await loadSettings()
  unlistenFocus = await listen(WINDOW_EVENTS.focusContextMenu, () => {
    void focusMenu()
  })
  unlistenWindowFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload) {
      void focusMenu()
      return
    }
    if (!hasFocused || Date.now() - focusedAt < BLUR_HIDE_GUARD_MS) return
    hasFocused = false
    void hideMenu()
  })
})

onUnmounted(() => {
  unlistenFocus?.()
  unlistenWindowFocus?.()
})
</script>

<template>
  <main
    ref="menuRef"
    class="context-menu"
    tabindex="-1"
    aria-label="桌宠快捷菜单"
    @keydown.esc="hideMenu"
  >
    <section v-if="ready" class="context-menu__surface">
      <div class="context-menu__group" aria-label="快捷操作">
        <button class="context-menu__item" type="button" @click="runAction('show_chat_window')">
          <MessageCircleIcon :size="16" aria-hidden="true" />
          <span>打开聊天</span>
        </button>
        <button class="context-menu__item" type="button" @click="runAction('show_main_context_status')">
          <MonitorIcon :size="16" aria-hidden="true" />
          <span>查看状态</span>
        </button>
        <button class="context-menu__item" type="button" @click="runAction('pause_all_reminders_until_tomorrow')">
          <PauseIcon :size="16" aria-hidden="true" />
          <span>提醒暂停到明天</span>
        </button>
        <button class="context-menu__item" type="button" @click="runAction('show_main_settings_window')">
          <SettingsIcon :size="16" aria-hidden="true" />
          <span>打开设置</span>
        </button>
      </div>

      <div class="context-menu__divider" role="separator"></div>

      <section class="context-menu__roles" aria-label="切换角色">
        <h2 class="context-menu__title">切换角色</h2>
        <div class="context-menu__role-list">
          <button
            v-for="role in petRoles"
            :key="role.id"
            class="context-menu__role"
            :class="{ 'context-menu__role--selected': role.id === selectedRole }"
            type="button"
            :aria-pressed="role.id === selectedRole"
            @click="selectRole(role.id)"
          >
            <span class="context-menu__role-text">
              <span class="context-menu__role-name">{{ role.displayName }}</span>
              <span class="context-menu__role-description">{{ role.description }}</span>
            </span>
            <CheckIcon v-if="role.id === selectedRole" :size="16" aria-label="当前角色" />
          </button>
        </div>
      </section>

      <div class="context-menu__divider" role="separator"></div>

      <button class="context-menu__item context-menu__item--danger" type="button" @click="runAction('exit_application')">
        <PowerIcon :size="16" aria-hidden="true" />
        <span>退出</span>
      </button>
    </section>
  </main>
</template>

<style scoped>
.context-menu {
  width: 100%;
  height: 100%;
  padding: 8px;
  outline: none;
}

.context-menu__surface {
  display: grid;
  height: 100%;
  padding: 6px;
  color: var(--popover-foreground);
  background: color-mix(in srgb, var(--popover) 94%, transparent);
  border: 1px solid color-mix(in srgb, var(--border) 86%, transparent);
  border-radius: 14px;
  box-shadow: 0 16px 34px color-mix(in srgb, var(--foreground) 18%, transparent);
  backdrop-filter: blur(14px);
}

.context-menu__group {
  display: grid;
  gap: 2px;
}

.context-menu__item,
.context-menu__role {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 9px;
  padding: 8px 10px;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 9px;
  cursor: pointer;
}

.context-menu__item {
  font-size: 13px;
  line-height: 18px;
}

.context-menu__item:hover,
.context-menu__role:hover,
.context-menu__role--selected {
  background: color-mix(in srgb, var(--accent) 88%, transparent);
}

.context-menu__item:focus-visible,
.context-menu__role:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: -2px;
}

.context-menu__divider {
  height: 1px;
  margin: 5px 4px;
  background: var(--border);
}

.context-menu__roles {
  display: grid;
  min-height: 0;
  gap: 4px;
}

.context-menu__title {
  margin: 0;
  padding: 2px 10px;
  color: var(--muted-foreground);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
}

.context-menu__role-list {
  display: grid;
  min-height: 0;
  max-height: 146px;
  gap: 2px;
  overflow-y: auto;
}

.context-menu__role {
  justify-content: space-between;
  min-height: 44px;
}

.context-menu__role-text {
  display: grid;
  min-width: 0;
  gap: 1px;
}

.context-menu__role-name,
.context-menu__role-description {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-menu__role-name {
  font-size: 13px;
  font-weight: 500;
  line-height: 17px;
}

.context-menu__role-description {
  color: var(--muted-foreground);
  font-size: 11px;
  line-height: 15px;
}

.context-menu__item--danger:hover {
  color: var(--destructive);
  background: color-mix(in srgb, var(--destructive) 10%, transparent);
}
</style>
