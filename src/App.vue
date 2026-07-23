<script setup lang="ts">
/**
 * App.vue - 根组件
 * 管理桌宠布局、拖拽、子组件协调
 */
import { ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import Pet from '@/components/Pet.vue'
import InfoPanel from '@/components/InfoPanel.vue'
import ChatBubble from '@/components/ChatBubble.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import { useSystemInfo } from '@/composables/useSystemInfo'
import { usePetState } from '@/composables/usePetState'

const { info } = useSystemInfo()
const {
  mood,
  isInfoPanelVisible,
  isChatVisible,
  updateMood,
  toggleInfoPanel,
  toggleChat,
} = usePetState()

const showContextMenu = ref(false)

// 系统信息更新时同步心情
watch(info, (val) => updateMood(val), { immediate: true })

/** 拖拽窗口（Tauri 原生拖拽） */
async function handleMouseDown(e: MouseEvent) {
  // 只响应左键
  if (e.button !== 0) return
  // 如果在子面板区域内，不触发拖拽
  const target = e.target as HTMLElement
  if (target.closest('.info-panel, .chat-bubble, .context-menu')) return

  try {
    await getCurrentWindow().startDragging()
  } catch {
    // 拖拽被取消，忽略
  }
}

/** 左键点击：关闭右键菜单，切换聊天 */
function handleClick(e: MouseEvent) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('.info-panel, .chat-bubble, .context-menu')) return

  showContextMenu.value = false
  toggleChat()
}

/** 右键菜单 */
function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  showContextMenu.value = !showContextMenu.value
}

/** 双击：切换系统信息面板 */
function handleDblClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('.info-panel, .chat-bubble, .context-menu')) return
  showContextMenu.value = false
  toggleInfoPanel()
}

/** 右键菜单操作 */
function handleToggleInfo() {
  showContextMenu.value = false
  toggleInfoPanel()
}

function handleToggleChat() {
  showContextMenu.value = false
  toggleChat()
}

async function handleQuit() {
  showContextMenu.value = false
  const win = getCurrentWindow()
  await win.close()
}
</script>

<template>
  <div
    class="app"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @contextmenu="handleContextMenu"
    @dblclick="handleDblClick"
  >
    <!-- 系统信息面板 -->
    <InfoPanel v-if="isInfoPanelVisible" :info="info" />

    <!-- 聊天气泡 -->
    <ChatBubble
      v-if="isChatVisible"
      :info="info"
      :mood="mood"
      @close="isChatVisible = false"
    />

    <!-- 右键菜单 -->
    <ContextMenu
      v-if="showContextMenu"
      @toggle-info="handleToggleInfo"
      @toggle-chat="handleToggleChat"
      @quit="handleQuit"
    />

    <!-- 桌宠角色 -->
    <Pet :mood="mood" />
  </div>
</template>

<style scoped>
.app {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background: transparent;
}
</style>
