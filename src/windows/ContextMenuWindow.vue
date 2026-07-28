<script setup lang="ts">
/** ContextMenuWindow.vue - 桌宠主题化右键菜单。 */
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import {
  CheckIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  MessageCircleIcon,
  MonitorIcon,
  PauseIcon,
  PowerIcon,
  SettingsIcon,
} from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getPetRole, petRoles } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import type { PetRoleId } from '@/types/pet'
import { WINDOW_EVENTS } from '@/types/window'

const { settings, ready, loadSettings } = useAppSettings()
const menuRef = ref<HTMLElement | null>(null)
const roleTriggerRef = ref<HTMLButtonElement | null>(null)
const view = ref<'menu' | 'roles'>('menu')
const selectedRole = computed(() => settings.value.pet_role)
const selectedRoleName = computed(() => getPetRole(selectedRole.value).displayName)
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

async function showRoles() {
  view.value = 'roles'
  await nextTick()
  menuRef.value?.focus()
}

async function showMenu() {
  view.value = 'menu'
  await nextTick()
  roleTriggerRef.value?.focus()
}

function handleEscape() {
  if (view.value === 'roles') {
    void showMenu()
    return
  }
  void hideMenu()
}

async function focusMenu() {
  hasFocused = true
  focusedAt = Date.now()
  view.value = 'menu'
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
    @keydown.esc="handleEscape"
  >
    <section v-if="ready" class="context-menu__surface">
      <template v-if="view === 'menu'">
        <div class="context-menu__group" aria-label="快捷操作">
          <button class="context-menu__item" type="button" @click="runAction('show_chat_window')">
            <MessageCircleIcon :size="15" aria-hidden="true" />
            <span>打开聊天</span>
          </button>
          <button class="context-menu__item" type="button" @click="runAction('show_main_context_status')">
            <MonitorIcon :size="15" aria-hidden="true" />
            <span>查看状态</span>
          </button>
          <button class="context-menu__item" type="button" @click="runAction('pause_all_reminders_until_tomorrow')">
            <PauseIcon :size="15" aria-hidden="true" />
            <span>提醒暂停到明天</span>
          </button>
          <button class="context-menu__item" type="button" @click="runAction('show_main_settings_window')">
            <SettingsIcon :size="15" aria-hidden="true" />
            <span>打开设置</span>
          </button>
        </div>

        <div class="context-menu__divider" role="separator"></div>

        <button
          ref="roleTriggerRef"
          class="context-menu__item context-menu__item--roles"
          type="button"
          aria-haspopup="true"
          :aria-expanded="false"
          @click="showRoles"
        >
          <span>切换角色</span>
          <span class="context-menu__role-summary">{{ selectedRoleName }}</span>
          <ChevronRightIcon :size="15" aria-hidden="true" />
        </button>

        <div class="context-menu__divider" role="separator"></div>

        <button class="context-menu__item context-menu__item--danger" type="button" @click="runAction('exit_application')">
          <PowerIcon :size="15" aria-hidden="true" />
          <span>退出</span>
        </button>
      </template>

      <template v-else>
        <header class="context-menu__roles-header">
          <button class="context-menu__back" type="button" aria-label="返回菜单" @click="showMenu">
            <ChevronLeftIcon :size="16" aria-hidden="true" />
          </button>
          <h2 class="context-menu__title">切换角色</h2>
        </header>
        <div class="context-menu__role-list" role="radiogroup" aria-label="桌宠角色">
          <button
            v-for="role in petRoles"
            :key="role.id"
            class="context-menu__role"
            :class="{ 'context-menu__role--selected': role.id === selectedRole }"
            type="button"
            role="radio"
            :aria-checked="role.id === selectedRole"
            @click="selectRole(role.id)"
          >
            <span class="context-menu__role-name">{{ role.displayName }}</span>
            <CheckIcon v-if="role.id === selectedRole" :size="15" aria-label="当前角色" />
          </button>
        </div>
      </template>
    </section>
  </main>
</template>

<style scoped>
.context-menu {
  width: 100%;
  height: 100%;
  padding: 4px;
  outline: none;
}

.context-menu__surface {
  display: grid;
  height: 100%;
  align-content: start;
  padding: 4px;
  color: var(--popover-foreground);
  background: color-mix(in srgb, var(--popover) 96%, transparent);
  border: 1px solid color-mix(in srgb, var(--border) 86%, transparent);
  border-radius: 11px;
}

.context-menu__group {
  display: grid;
  gap: 1px;
}

.context-menu__item,
.context-menu__role,
.context-menu__back {
  display: flex;
  align-items: center;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 7px;
  cursor: pointer;
}

.context-menu__item,
.context-menu__role {
  width: 100%;
  min-height: 28px;
  gap: 8px;
  padding: 5px 8px;
  font-size: 12px;
  line-height: 16px;
}

.context-menu__item:hover,
.context-menu__role:hover,
.context-menu__role--selected {
  background: color-mix(in srgb, var(--accent) 88%, transparent);
}

.context-menu__item:focus-visible,
.context-menu__role:focus-visible,
.context-menu__back:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: -2px;
}

.context-menu__divider {
  height: 1px;
  margin: 4px;
  background: var(--border);
}

.context-menu__item--roles {
  justify-content: flex-start;
}

.context-menu__role-summary {
  min-width: 0;
  margin-left: auto;
  overflow: hidden;
  color: var(--muted-foreground);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-menu__item--danger:hover {
  color: var(--destructive);
  background: color-mix(in srgb, var(--destructive) 10%, transparent);
}

.context-menu__roles-header {
  display: flex;
  align-items: center;
  min-height: 28px;
  padding: 0 4px 3px;
}

.context-menu__back {
  justify-content: center;
  width: 24px;
  height: 24px;
}

.context-menu__title {
  margin: 0 0 0 5px;
  font-size: 12px;
  font-weight: 600;
  line-height: 16px;
}

.context-menu__role-list {
  display: grid;
  min-height: 0;
  max-height: 172px;
  gap: 1px;
  overflow-y: auto;
}

.context-menu__role {
  justify-content: space-between;
}

.context-menu__role-name {
  overflow: hidden;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
