<script setup lang="ts">
/** ContextMenuWindow.vue - 桌宠主题化右键菜单。 */
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import {
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
import ContextMenuExitDialog from '@/components/ContextMenuExitDialog.vue'
import ContextMenuRolePicker from '@/components/ContextMenuRolePicker.vue'
import { getPetRole } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import type { PetRoleId } from '@/types/pet'
import { WINDOW_EVENTS } from '@/types/window'

const { settings, ready, loadSettings } = useAppSettings()
const rootItemRefs = new Map<string, HTMLButtonElement>()
const view = ref<'menu' | 'roles'>('menu')
const exitDialogOpen = ref(false)
const selectedRole = computed(() => settings.value.pet_role)
const selectedRoleName = computed(() => getPetRole(selectedRole.value).displayName)
const rootItemIds = ['chat', 'status', 'pause', 'settings', 'roles', 'exit'] as const
let unlistenFocus: UnlistenFn | null = null
let unlistenWindowFocus: UnlistenFn | null = null
let focusedAt = 0
let hasFocused = false
const BLUR_HIDE_GUARD_MS = 160

function setRootItemRef(itemId: string, element: unknown) {
  if (element instanceof HTMLButtonElement) {
    rootItemRefs.set(itemId, element)
    return
  }
  rootItemRefs.delete(itemId)
}

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

async function pauseReminders() {
  try {
    await invoke('pause_all_reminders_until_tomorrow')
  } catch (error) {
    console.error('暂停提醒失败:', error)
    return
  }

  try {
    await invoke('show_reminders_paused_confirmation')
  } catch (error) {
    console.error('显示提醒暂停提示失败:', error)
  }
  await hideMenu()
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

function focusRootItem(itemId: string) {
  rootItemRefs.get(itemId)?.focus()
}

function moveRootItem(currentId: string, direction: number) {
  const currentIndex = rootItemIds.indexOf(currentId as typeof rootItemIds[number])
  const nextIndex = (currentIndex + direction + rootItemIds.length) % rootItemIds.length
  focusRootItem(rootItemIds[nextIndex]!)
}

function handleRootKeydown(event: KeyboardEvent, itemId: string) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
    event.preventDefault()
    if (itemId === 'roles' && event.key === 'ArrowRight') {
      showRoles()
      return
    }
    moveRootItem(itemId, 1)
    return
  }
  if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
    event.preventDefault()
    moveRootItem(itemId, -1)
    return
  }
  if (event.key === 'Home') {
    event.preventDefault()
    focusRootItem(rootItemIds[0])
    return
  }
  if (event.key === 'End') {
    event.preventDefault()
    focusRootItem(rootItemIds[rootItemIds.length - 1])
  }
}

function showRoles() {
  view.value = 'roles'
}

async function showMenu() {
  view.value = 'menu'
  await nextTick()
  focusRootItem('roles')
}

function handleEscape() {
  if (exitDialogOpen.value) return
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
  focusRootItem(rootItemIds[0])
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
  <main class="context-menu" tabindex="-1" aria-label="桌宠快捷菜单" @keydown.esc="handleEscape">
    <section v-if="ready" class="context-menu__surface">
      <template v-if="view === 'menu'">
        <div class="context-menu__group" aria-label="快捷操作">
          <button :ref="element => setRootItemRef('chat', element)" class="context-menu__item" type="button" @keydown="handleRootKeydown($event, 'chat')" @click="runAction('show_chat_window')">
            <MessageCircleIcon :size="15" aria-hidden="true" />
            <span>打开聊天</span>
          </button>
          <button :ref="element => setRootItemRef('status', element)" class="context-menu__item" type="button" @keydown="handleRootKeydown($event, 'status')" @click="runAction('show_main_context_status')">
            <MonitorIcon :size="15" aria-hidden="true" />
            <span>查看状态</span>
          </button>
          <button :ref="element => setRootItemRef('pause', element)" class="context-menu__item" type="button" @keydown="handleRootKeydown($event, 'pause')" @click="pauseReminders">
            <PauseIcon :size="15" aria-hidden="true" />
            <span>提醒暂停到明天</span>
          </button>
          <button :ref="element => setRootItemRef('settings', element)" class="context-menu__item" type="button" @keydown="handleRootKeydown($event, 'settings')" @click="runAction('show_main_settings_window')">
            <SettingsIcon :size="15" aria-hidden="true" />
            <span>打开设置</span>
          </button>
        </div>

        <div class="context-menu__divider" role="separator"></div>

        <button :ref="element => setRootItemRef('roles', element)" class="context-menu__item context-menu__item--roles" type="button" aria-haspopup="dialog" @keydown="handleRootKeydown($event, 'roles')" @click="showRoles">
          <span>切换角色</span>
          <span class="context-menu__role-summary">{{ selectedRoleName }}</span>
          <ChevronRightIcon :size="15" aria-hidden="true" />
        </button>

        <div class="context-menu__divider" role="separator"></div>

        <button :ref="element => setRootItemRef('exit', element)" class="context-menu__item context-menu__item--danger" type="button" @keydown="handleRootKeydown($event, 'exit')" @click="exitDialogOpen = true">
          <PowerIcon :size="15" aria-hidden="true" />
          <span>退出</span>
        </button>
      </template>

      <ContextMenuRolePicker v-else :selected-role="selectedRole" @back="showMenu" @select="selectRole" />
    </section>
  </main>

  <ContextMenuExitDialog v-model:open="exitDialogOpen" />
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

.context-menu__item {
  display: flex;
  align-items: center;
  width: 100%;
  min-height: 28px;
  gap: 8px;
  padding: 5px 8px;
  color: inherit;
  font-size: 12px;
  line-height: 16px;
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 7px;
  cursor: pointer;
}

.context-menu__item:hover {
  background: color-mix(in srgb, var(--accent) 88%, transparent);
}

.context-menu__item:focus-visible {
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

</style>
