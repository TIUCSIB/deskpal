<script setup lang="ts">
/** ContextMenuReminderPicker.vue - 右键菜单中的提醒暂停操作。 */
import { computed, nextTick, ref } from 'vue'
import { ChevronLeftIcon, PauseIcon, SettingsIcon } from '@lucide/vue'
import type { Reminder } from '@/types/settings'

const props = defineProps<{
  reminders: Reminder[]
}>()

const emit = defineEmits<{
  back: []
  pauseAll: []
  pauseOne: [reminderId: string]
  openSettings: []
}>()

const itemRefs = new Map<string, HTMLButtonElement>()
const activeItemId = ref('pause-all')
const enabledReminders = computed(() => props.reminders.filter(reminder => reminder.enabled))
const availableReminders = computed(() => enabledReminders.value.filter(reminder => !reminder.paused_until))
const itemIds = computed(() => [
  'pause-all',
  ...availableReminders.value.map(reminder => reminder.id),
  'settings',
])

function setItemRef(itemId: string, element: unknown) {
  if (element instanceof HTMLButtonElement) {
    itemRefs.set(itemId, element)
    return
  }
  itemRefs.delete(itemId)
}

function focusItem(itemId: string) {
  activeItemId.value = itemId
  itemRefs.get(itemId)?.focus()
}

function moveItem(direction: number) {
  const ids = itemIds.value
  const currentIndex = Math.max(ids.indexOf(activeItemId.value), 0)
  const nextIndex = (currentIndex + direction + ids.length) % ids.length
  focusItem(ids[nextIndex]!)
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
    event.preventDefault()
    moveItem(1)
    return
  }
  if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
    event.preventDefault()
    moveItem(-1)
    return
  }
  if (event.key === 'Home') {
    event.preventDefault()
    focusItem(itemIds.value[0]!)
    return
  }
  if (event.key === 'End') {
    event.preventDefault()
    focusItem(itemIds.value[itemIds.value.length - 1]!)
  }
}

function handleEscape() {
  emit('back')
}

function handleAuxClick(event: MouseEvent) {
  if (event.button !== 3) return
  event.preventDefault()
  emit('back')
}

nextTick(() => focusItem('pause-all'))
</script>

<template>
  <section class="reminder-picker" @keydown.esc.stop="handleEscape" @mouseup="handleAuxClick">
    <header class="reminder-picker__header">
      <button class="reminder-picker__back" type="button" aria-label="返回菜单" @click="emit('back')">
        <ChevronLeftIcon :size="16" aria-hidden="true" />
      </button>
      <h2 class="reminder-picker__title">提醒</h2>
    </header>

    <button
      :ref="element => setItemRef('pause-all', element)"
      class="reminder-picker__action"
      type="button"
      :tabindex="activeItemId === 'pause-all' ? 0 : -1"
      @focus="activeItemId = 'pause-all'"
      @keydown="handleKeydown"
      @click="emit('pauseAll')"
    >
      <PauseIcon :size="15" aria-hidden="true" />
      <span>全部暂停到明天</span>
    </button>

    <div class="reminder-picker__divider" role="separator"></div>

    <div class="reminder-picker__list" aria-label="已启用提醒">
      <p v-if="!enabledReminders.length" class="reminder-picker__empty">暂无已启用提醒</p>
      <template v-for="reminder in enabledReminders" :key="reminder.id">
        <div v-if="reminder.paused_until" class="reminder-picker__paused">
          <span class="reminder-picker__message">{{ reminder.message }}</span>
          <span>已暂停</span>
        </div>
        <button
          v-else
          :ref="element => setItemRef(reminder.id, element)"
          class="reminder-picker__reminder"
          type="button"
          :tabindex="activeItemId === reminder.id ? 0 : -1"
          @focus="activeItemId = reminder.id"
          @keydown="handleKeydown"
          @click="emit('pauseOne', reminder.id)"
        >
          <span class="reminder-picker__message">{{ reminder.message }}</span>
          <span class="reminder-picker__pause-label">暂停到明天</span>
        </button>
      </template>
    </div>

    <div class="reminder-picker__divider" role="separator"></div>

    <button
      :ref="element => setItemRef('settings', element)"
      class="reminder-picker__action"
      type="button"
      :tabindex="activeItemId === 'settings' ? 0 : -1"
      @focus="activeItemId = 'settings'"
      @keydown="handleKeydown"
      @click="emit('openSettings')"
    >
      <SettingsIcon :size="15" aria-hidden="true" />
      <span>打开提醒设置</span>
    </button>
  </section>
</template>

<style scoped>
.reminder-picker__header {
  display: flex;
  align-items: center;
  min-height: 28px;
  padding: 0 4px 3px;
}

.reminder-picker__back,
.reminder-picker__action,
.reminder-picker__reminder {
  display: flex;
  align-items: center;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 7px;
  cursor: pointer;
}

.reminder-picker__back {
  justify-content: center;
  width: 24px;
  height: 24px;
}

.reminder-picker__title {
  margin: 0 0 0 5px;
  font-size: 12px;
  font-weight: 600;
  line-height: 16px;
}

.reminder-picker__action,
.reminder-picker__reminder {
  width: 100%;
  min-height: 28px;
  gap: 8px;
  padding: 5px 8px;
  font-size: 12px;
  line-height: 16px;
}

.reminder-picker__action:hover,
.reminder-picker__reminder:hover {
  background: color-mix(in srgb, var(--accent) 88%, transparent);
}

.reminder-picker__back:focus-visible,
.reminder-picker__action:focus-visible,
.reminder-picker__reminder:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: -2px;
}

.reminder-picker__divider {
  height: 1px;
  margin: 4px;
  background: var(--border);
}

.reminder-picker__list {
  display: grid;
  max-height: 108px;
  gap: 1px;
  overflow-y: auto;
}

.reminder-picker__reminder,
.reminder-picker__paused {
  justify-content: space-between;
  gap: 8px;
}

.reminder-picker__paused {
  display: flex;
  min-height: 28px;
  padding: 5px 8px;
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 16px;
}

.reminder-picker__message {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.reminder-picker__pause-label {
  flex: none;
  color: var(--muted-foreground);
  font-size: 11px;
}

.reminder-picker__empty {
  margin: 0;
  padding: 12px 8px;
  color: var(--muted-foreground);
  font-size: 12px;
  text-align: center;
}
</style>
