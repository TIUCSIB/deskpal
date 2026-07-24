<script setup lang="ts">
/** App.vue - 根据原生窗口角色分发根组件 */
import { computed } from 'vue'
import ChatWindow from '@/windows/ChatWindow.vue'
import InfoWindow from '@/windows/InfoWindow.vue'
import PetWindow from '@/windows/PetWindow.vue'
import type { WindowRole } from '@/types/window'

const windowRole = computed<WindowRole>(() => {
  const role = new URLSearchParams(window.location.search).get('window')
  if (role === 'chat' || role === 'info') return role
  return 'pet'
})
</script>

<template>
  <ChatWindow v-if="windowRole === 'chat'" />
  <InfoWindow v-else-if="windowRole === 'info'" />
  <PetWindow v-else />
</template>
