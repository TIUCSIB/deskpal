<script setup lang="ts">
/** App.vue - 根据原生窗口角色分发根组件 */
import { computed } from 'vue'
import ChatWindow from '@/windows/ChatWindow.vue'
import ContextMenuWindow from '@/windows/ContextMenuWindow.vue'
import InfoWindow from '@/windows/InfoWindow.vue'
import PetWindow from '@/windows/PetWindow.vue'
import ReminderWindow from '@/windows/ReminderWindow.vue'
import SettingsWindow from '@/windows/SettingsWindow.vue'
import SystemFeedbackWindow from '@/windows/SystemFeedbackWindow.vue'
import type { WindowRole } from '@/types/window'

const windowRole = computed<WindowRole>(() => {
  const role = new URLSearchParams(window.location.search).get('window')
  if (role === 'context-menu' || role === 'chat' || role === 'info' || role === 'settings' || role === 'reminder' || role === 'feedback') {
    return role
  }
  return 'pet'
})
</script>

<template>
  <ContextMenuWindow v-if="windowRole === 'context-menu'" />
  <ChatWindow v-else-if="windowRole === 'chat'" />
  <InfoWindow v-else-if="windowRole === 'info'" />
  <SettingsWindow v-else-if="windowRole === 'settings'" />
  <ReminderWindow v-else-if="windowRole === 'reminder'" />
  <SystemFeedbackWindow v-else-if="windowRole === 'feedback'" />
  <PetWindow v-else />
</template>
