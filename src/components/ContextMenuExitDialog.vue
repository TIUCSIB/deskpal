<script setup lang="ts">
/** ContextMenuExitDialog.vue - 右键菜单的受控退出确认对话框。 */
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

const open = defineModel<boolean>('open', { required: true })
const isExiting = ref(false)

async function confirmExit() {
  if (isExiting.value) return
  isExiting.value = true
  try {
    await invoke('exit_application')
  } catch (error) {
    console.error('退出桌宠失败:', error)
    isExiting.value = false
  }
}
</script>

<template>
  <AlertDialog v-model:open="open">
    <AlertDialogContent size="sm" class="context-menu-exit-dialog">
      <AlertDialogHeader>
        <AlertDialogTitle>退出桌宠？</AlertDialogTitle>
        <AlertDialogDescription>桌宠、提醒和聊天窗口将关闭。</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="isExiting">取消</AlertDialogCancel>
        <AlertDialogAction variant="destructive" :disabled="isExiting" @click.capture="confirmExit">退出</AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>

<style scoped>
.context-menu-exit-dialog {
  max-width: calc(100vw - 16px);
}
</style>
