<script setup lang="ts">
/**
 * ChatBubble.vue - iOS 风格单容器聊天
 * 输入、加载和回复在同一表面内切换。
 */
import { nextTick, onUnmounted, ref } from 'vue'
import type { PetMood, SystemInfo } from '@/types/system'
import { generateReply } from '@/composables/useChat'

type ChatView = 'input' | 'loading' | 'reply'

const props = defineProps<{
  info: SystemInfo | null
  mood: PetMood
}>()

const LOADING_DURATION = 420
const inputText = ref('')
const replyText = ref('')
const view = ref<ChatView>('input')
const inputRef = ref<HTMLInputElement | null>(null)
let replyTimer: ReturnType<typeof setTimeout> | null = null

/** 将焦点交给输入框 */
function focusInput() {
  nextTick(() => inputRef.value?.focus())
}

/** 清理当前会话并恢复输入态 */
function resetSession() {
  if (replyTimer) clearTimeout(replyTimer)
  replyTimer = null
  inputText.value = ''
  replyText.value = ''
  view.value = 'input'
}

/** 发送消息，在原容器中先加载再显示回复 */
function handleSend() {
  const text = inputText.value.trim()
  if (!text || view.value !== 'input') return

  const reply = generateReply(text, props.info, props.mood)
  inputText.value = ''
  view.value = 'loading'
  replyTimer = setTimeout(() => {
    replyText.value = reply
    view.value = 'reply'
    replyTimer = null
  }, LOADING_DURATION)
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSend()
  }
}

/** 从回复态回到输入态 */
function startNewMessage() {
  replyText.value = ''
  view.value = 'input'
  focusInput()
}

onUnmounted(resetSession)

defineExpose({ focusInput, resetSession })
</script>

<template>
  <section class="chat-bubble" aria-label="桌宠聊天">
    <div class="chat-bubble__surface" :class="`chat-bubble__surface--${view}`">
      <template v-if="view === 'input'">
        <input
          ref="inputRef"
          v-model="inputText"
          class="chat-bubble__input"
          placeholder="说点什么…"
          maxlength="160"
          @keydown="handleKeydown"
        />
        <button
          class="chat-bubble__button chat-bubble__button--send"
          type="button"
          title="发送"
          :disabled="!inputText.trim()"
          @click="handleSend"
        >
          ↑
        </button>
      </template>

      <div
        v-else-if="view === 'loading'"
        class="chat-bubble__loading"
        role="status"
        aria-live="polite"
      >
        <span class="chat-bubble__dots" aria-hidden="true">
          <i></i><i></i><i></i>
        </span>
        <span class="chat-bubble__loading-text">正在想…</span>
      </div>

      <template v-else>
        <p class="chat-bubble__reply" aria-live="polite">{{ replyText }}</p>
        <button
          class="chat-bubble__button chat-bubble__button--secondary"
          type="button"
          title="继续聊天"
          @click="startNewMessage"
        >
          ↩
        </button>
      </template>
    </div>
  </section>
</template>

<style scoped>
.chat-bubble {
  width: 232px;
  max-width: 100%;
  color: #1c1c1e;
  pointer-events: auto;
}

.chat-bubble__surface {
  width: 100%;
  min-height: 40px;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 6px 5px 12px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.97);
  border: 1px solid rgba(60, 60, 67, 0.16);
  border-radius: 20px;
  transition:
    min-height 180ms ease,
    padding 180ms ease,
    border-radius 180ms ease,
    opacity 160ms ease;
}

.chat-bubble__surface--loading {
  justify-content: center;
  color: #8e8e93;
  font-size: 12px;
}

.chat-bubble__surface--reply {
  min-height: 54px;
  max-height: 54px;
  padding: 8px 6px 8px 12px;
  border-radius: 18px;
}

.chat-bubble__input {
  min-width: 0;
  flex: 1;
  padding: 0;
  color: #1c1c1e;
  background: transparent;
  border: 0;
  outline: 0;
  font-size: 13px;
  letter-spacing: 0;
}

.chat-bubble__input::placeholder {
  color: #8e8e93;
}

.chat-bubble__button {
  width: 30px;
  height: 30px;
  flex: 0 0 30px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 50%;
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}

.chat-bubble__button--send {
  color: #fff;
  background: #007aff;
  font-weight: 700;
}

.chat-bubble__button--send:disabled {
  background: #c7c7cc;
  cursor: default;
}

.chat-bubble__button--secondary {
  color: #007aff;
  background: #f2f2f7;
}

.chat-bubble__loading {
  display: flex;
  align-items: center;
  gap: 10px;
}

.chat-bubble__loading-text {
  opacity: 0.85;
}

.chat-bubble__dots {
  display: flex;
  gap: 4px;
}

.chat-bubble__dots i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #8e8e93;
  animation: thinking 900ms ease-in-out infinite;
}

.chat-bubble__dots i:nth-child(2) {
  animation-delay: 120ms;
}

.chat-bubble__dots i:nth-child(3) {
  animation-delay: 240ms;
}

.chat-bubble__reply {
  display: -webkit-box;
  min-width: 0;
  flex: 1;
  margin: 0;
  overflow: hidden;
  color: #1c1c1e;
  font-size: 12px;
  line-height: 1.35;
  letter-spacing: 0;
  word-break: break-word;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  animation: reply-in 180ms ease-out;
}

@keyframes reply-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes thinking {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.45; }
  30% { transform: translateY(-3px); opacity: 1; }
}
</style>
