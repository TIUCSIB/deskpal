<script setup lang="ts">
/**
 * ChatBubble.vue - 胶囊输入框
 * 点击桌宠后弹出胶囊输入框，发送后显示回复，无对话历史
 */
import { nextTick, ref } from 'vue'
import type { SystemInfo, PetMood } from '@/types/system'
import { generateReply } from '@/composables/useChat'

const props = defineProps<{
  info: SystemInfo | null
  mood: PetMood
}>()

const emit = defineEmits<{ close: [] }>()

const inputText = ref('')
const replyText = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

/** 将焦点交给输入框，供窗口显示后调用 */
function focusInput() {
  inputRef.value?.focus()
}

/** 发送消息，回复显示在原位 */
function handleSend() {
  const text = inputText.value.trim()
  if (!text) return

  replyText.value = generateReply(text, props.info, props.mood)
  inputText.value = ''
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

/** 关闭时清空状态 */
function handleClose() {
  replyText.value = ''
  inputText.value = ''
  emit('close')
}

/** 自动聚焦输入框 */
nextTick(focusInput)

defineExpose({ focusInput })
</script>

<template>
  <div class="chat-bubble">
    <!-- 回复内容（有回复时显示） -->
    <div v-if="replyText" class="chat-bubble__reply">
      {{ replyText }}
    </div>

    <!-- 胶囊输入框 -->
    <div class="chat-bubble__input-area">
      <input
        ref="inputRef"
        v-model="inputText"
        class="chat-bubble__input"
        placeholder="说点什么..."
        @keydown="handleKeydown"
      />
      <button class="chat-bubble__send" @click="handleSend">➤</button>
      <button class="chat-bubble__close" @click="handleClose">✕</button>
    </div>
  </div>
</template>

<style scoped>
.chat-bubble {
  position: relative;
  width: 100%;
  min-width: 200px;
  max-width: 360px;
  animation: slide-up 0.2s ease-out;
  z-index: 9999;
  pointer-events: auto;
  isolation: isolate;
}

/* 回复气泡 */
.chat-bubble__reply {
  background: rgba(30, 30, 30, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px 14px 14px 4px;
  padding: 10px 14px;
  font-size: 13px;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 6px;
  word-break: break-word;
}

/* 胶囊输入区域 */
.chat-bubble__input-area {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(30, 30, 30, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 4px 4px 4px 14px;
  backdrop-filter: blur(10px);
}

.chat-bubble__input {
  flex: 1;
  background: none;
  border: none;
  font-size: 13px;
  color: #fff;
  outline: none;
  min-width: 0;
}

.chat-bubble__input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.chat-bubble__send {
  background: #1976d2;
  border: none;
  border-radius: 50%;
  color: #fff;
  width: 28px;
  height: 28px;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.chat-bubble__send:hover {
  background: #1565c0;
}

.chat-bubble__close {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  font-size: 12px;
  padding: 4px;
  flex-shrink: 0;
}

.chat-bubble__close:hover {
  color: #fff;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
