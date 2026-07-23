<script setup lang="ts">
/**
 * ContextMenu.vue - 右键菜单
 * 提供显示信息、聊天、退出等操作
 */
const emit = defineEmits<{
  toggleInfo: []
  toggleChat: []
  quit: []
}>()
</script>

<template>
  <div class="context-menu">
    <button class="context-menu__item" @click="emit('toggleInfo')">
      📊 系统信息
    </button>
    <button class="context-menu__item" @click="emit('toggleChat')">
      💬 聊天
    </button>
    <div class="context-menu__divider"></div>
    <button class="context-menu__item context-menu__item--danger" @click="emit('quit')">
      🚪 退出
    </button>
  </div>
</template>

<style scoped>
.context-menu {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 10px;
  padding: 4px;
  min-width: 140px;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  animation: fade-in 0.15s ease-out;
}

.context-menu__item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background 0.15s;
}
.context-menu__item:hover {
  background: rgba(255, 255, 255, 0.1);
}
.context-menu__item--danger:hover {
  background: rgba(229, 57, 53, 0.2);
  color: #ef5350;
}

.context-menu__divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  margin: 4px 8px;
}

@keyframes fade-in {
  from { opacity: 0; transform: translateX(-50%) scale(0.95); }
  to { opacity: 1; transform: translateX(-50%) scale(1); }
}
</style>
