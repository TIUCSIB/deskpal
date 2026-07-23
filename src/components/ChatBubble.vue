<script setup lang="ts">
/**
 * ChatBubble.vue - 对话气泡
 * 点击桌宠后弹出，支持简单聊天
 */
import { nextTick, ref, watch } from 'vue'
import type { SystemInfo, PetMood } from '@/types/system'
import { useChat, getGreeting } from '@/composables/useChat'

const props = defineProps<{
  info: SystemInfo | null
  mood: PetMood
}>()

const emit = defineEmits<{ close: [] }>()

const { messages, inputText, sendMessage, addSystemMessage } = useChat()
const messagesRef = ref<HTMLDivElement | null>(null)

// 首次打开时打招呼
let hasGreeted = false
watch(
  () => props.mood,
  () => {
    if (!hasGreeted) {
      addSystemMessage(getGreeting())
      hasGreeted = true
    }
  },
  { immediate: true },
)

// 自动滚动到底部
watch(
  () => messages.value.length,
  async () => {
    await nextTick()
    if (messagesRef.value) {
      messagesRef.value.scrollTop = messagesRef.value.scrollHeight
    }
  },
)

function handleSend() {
  sendMessage(props.info, props.mood)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}
</script>

<template>
  <div class="chat-bubble">
    <div class="chat-bubble__header">
      <span>💬 聊天</span>
      <button class="chat-bubble__close" @click="emit('close')">✕</button>
    </div>

    <div class="chat-bubble__messages" ref="messagesRef">
      <div
        v-for="(msg, i) in messages"
        :key="i"
        class="chat-bubble__msg"
        :class="msg.isUser ? 'chat-bubble__msg--user' : 'chat-bubble__msg--pet'"
      >
        {{ msg.text }}
      </div>
    </div>

    <div class="chat-bubble__input-area">
      <input
        v-model="inputText"
        class="chat-bubble__input"
        placeholder="说点什么..."
        @keydown="handleKeydown"
      />
      <button class="chat-bubble__send" @click="handleSend">➤</button>
    </div>
  </div>
</template>

<style scoped>
.chat-bubble {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
  width: 260px;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(10px);
  display: flex;
  flex-direction: column;
  animation: slide-up 0.2s ease-out;
  overflow: hidden;
}

.chat-bubble__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}
.chat-bubble__close {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  font-size: 14px;
  padding: 0 4px;
}
.chat-bubble__close:hover {
  color: #fff;
}

.chat-bubble__messages {
  height: 180px;
  overflow-y: auto;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.chat-bubble__messages::-webkit-scrollbar {
  width: 4px;
}
.chat-bubble__messages::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.chat-bubble__msg {
  max-width: 80%;
  padding: 6px 10px;
  border-radius: 10px;
  font-size: 12px;
  line-height: 1.4;
  word-break: break-word;
}
.chat-bubble__msg--user {
  align-self: flex-end;
  background: #1976d2;
  color: #fff;
  border-bottom-right-radius: 4px;
}
.chat-bubble__msg--pet {
  align-self: flex-start;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
  border-bottom-left-radius: 4px;
}

.chat-bubble__input-area {
  display: flex;
  padding: 8px;
  gap: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}
.chat-bubble__input {
  flex: 1;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  color: #fff;
  outline: none;
}
.chat-bubble__input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}
.chat-bubble__input:focus {
  border-color: rgba(25, 118, 210, 0.5);
}
.chat-bubble__send {
  background: #1976d2;
  border: none;
  border-radius: 8px;
  color: #fff;
  width: 32px;
  cursor: pointer;
  font-size: 14px;
}
.chat-bubble__send:hover {
  background: #1565c0;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateX(-50%) translateY(6px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
