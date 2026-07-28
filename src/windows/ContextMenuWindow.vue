<script setup lang="ts">
/** ContextMenuWindow.vue - 桌宠主题化右键菜单。 */
import { computed, nextTick, onMounted, ref } from 'vue'
import {
  ChevronRightIcon,
  MessageCircleIcon,
  MonitorIcon,
  PauseIcon,
  PowerIcon,
  SettingsIcon,
} from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import ContextMenuExitDialog from '@/components/ContextMenuExitDialog.vue'
import ContextMenuReminderPicker from '@/components/ContextMenuReminderPicker.vue'
import ContextMenuRolePicker from '@/components/ContextMenuRolePicker.vue'
import { getPetRole } from '@/config/petRoles'
import { useAppSettings } from '@/composables/useAppSettings'
import { useContextMenuFocus } from '@/composables/useContextMenuFocus'
import type { PetRoleId } from '@/types/pet'

const { settings, ready, loadSettings } = useAppSettings()
const rootItemRefs = new Map<string, HTMLButtonElement>()
const view = ref<'menu' | 'reminders' | 'roles'>('menu')
const exitDialogOpen = ref(false)
const reminders = computed(() => settings.value.reminders)
const selectedRole = computed(() => settings.value.pet_role)
const selectedRoleName = computed(() => getPetRole(selectedRole.value).displayName)
const rootItemIds = ['chat', 'status', 'reminders', 'settings', 'roles', 'exit'] as const

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

async function pauseReminder(reminderId: string) {
  try {
    await invoke('pause_enabled_reminder_until_tomorrow', { reminderId })
  } catch (error) {
    console.error('暂停提醒失败:', error)
    return
  }

  try {
    await invoke('show_reminder_paused_confirmation', { reminderId })
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
    if (event.key === 'ArrowRight' && itemId === 'reminders') {
      showReminders()
      return
    }
    if (event.key === 'ArrowRight' && itemId === 'roles') {
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

function showReminders() {
  view.value = 'reminders'
}

function showRoles() {
  view.value = 'roles'
}

async function showMenu(focusItem = 'roles') {
  view.value = 'menu'
  await nextTick()
  focusRootItem(focusItem)
}

function handleEscape() {
  if (exitDialogOpen.value) return
  if (view.value === 'reminders') {
    void showMenu('reminders')
    return
  }
  if (view.value === 'roles') {
    void showMenu()
    return
  }
  void hideMenu()
}

async function focusMenu() {
  exitDialogOpen.value = false
  view.value = 'menu'
  await nextTick()
  focusRootItem(rootItemIds[0])
}

useContextMenuFocus(focusMenu, hideMenu)

onMounted(() => {
  void loadSettings()
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
          <button :ref="element => setRootItemRef('reminders', element)" class="context-menu__item context-menu__item--reminders" type="button" aria-haspopup="dialog" @keydown="handleRootKeydown($event, 'reminders')" @click="showReminders">
            <PauseIcon :size="15" aria-hidden="true" />
            <span>管理提醒</span>
            <ChevronRightIcon :size="15" aria-hidden="true" />
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

      <ContextMenuReminderPicker
        v-else-if="view === 'reminders'"
        :reminders="reminders"
        @back="showMenu('reminders')"
        @pause-all="pauseReminders"
        @pause-one="pauseReminder"
        @open-settings="runAction('show_main_reminder_settings')"
      />
      <ContextMenuRolePicker v-else :selected-role="selectedRole" @back="showMenu" @select="selectRole" />
    </section>
  </main>

  <ContextMenuExitDialog v-model:open="exitDialogOpen" />
</template>

<style scoped src="./ContextMenuWindow.css"></style>
