<script setup lang="ts">
/**
 * Pet.vue - 桌宠角色（CSS 占位符）
 * 根据心情切换表情和动画
 */
import { computed } from 'vue'
import type { PetMood } from '../types/system'

const props = defineProps<{ mood: PetMood }>()

/** 心情 → 表情映射 */
const face = computed(() => {
  switch (props.mood) {
    case 'happy': return { eyes: '^ ^', mouth: 'ω' }
    case 'sleepy': return { eyes: '- -', mouth: '~' }
    case 'warning': return { eyes: '> <', mouth: '△' }
    default: return { eyes: '● ●', mouth: '▽' }
  }
})
</script>

<template>
  <div class="pet" :class="`pet--${mood}`">
    <!-- 耳朵 -->
    <div class="pet__ear pet__ear--left"></div>
    <div class="pet__ear pet__ear--right"></div>
    <!-- 身体 -->
    <div class="pet__body">
      <!-- 表情 -->
      <div class="pet__face">
        <div class="pet__eyes">{{ face.eyes }}</div>
        <div class="pet__mouth">{{ face.mouth }}</div>
      </div>
    </div>
    <!-- 手 -->
    <div class="pet__hand pet__hand--left"></div>
    <div class="pet__hand pet__hand--right"></div>
    <!-- 脚 -->
    <div class="pet__feet">
      <div class="pet__foot"></div>
      <div class="pet__foot"></div>
    </div>
  </div>
</template>

<style scoped>
.pet {
  position: relative;
  width: 100px;
  height: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  animation: breathe 3s ease-in-out infinite;
}

/* —— 耳朵 —— */
.pet__ear {
  position: absolute;
  top: 0;
  width: 22px;
  height: 22px;
  background: #ffb74d;
  border-radius: 50% 50% 0 0;
  border: 2px solid #f57c00;
  border-bottom: none;
}
.pet__ear--left { left: 16px; transform: rotate(-15deg); }
.pet__ear--right { right: 16px; transform: rotate(15deg); }

/* —— 身体 —— */
.pet__body {
  width: 80px;
  height: 80px;
  background: #ffcc80;
  border-radius: 50%;
  border: 3px solid #f57c00;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 10px;
  z-index: 1;
}

/* —— 表情 —— */
.pet__face {
  text-align: center;
  line-height: 1;
}
.pet__eyes {
  font-size: 16px;
  font-weight: bold;
  color: #4e342e;
  letter-spacing: 6px;
  margin-bottom: 4px;
}
.pet__mouth {
  font-size: 18px;
  color: #e65100;
}

/* —— 手 —— */
.pet__hand {
  position: absolute;
  bottom: 30px;
  width: 16px;
  height: 10px;
  background: #ffb74d;
  border-radius: 0 0 50% 50%;
  border: 2px solid #f57c00;
  border-top: none;
  z-index: 0;
}
.pet__hand--left {
  left: 4px;
  animation: wave-left 2s ease-in-out infinite;
}
.pet__hand--right {
  right: 4px;
  animation: wave-right 2s ease-in-out infinite;
}

/* —— 脚 —— */
.pet__feet {
  display: flex;
  gap: 12px;
  margin-top: 2px;
}
.pet__foot {
  width: 20px;
  height: 10px;
  background: #ffb74d;
  border-radius: 0 0 50% 50%;
  border: 2px solid #f57c00;
  border-top: none;
}

/* —— 心情样式 —— */
.pet--happy { animation: breathe 3s ease-in-out infinite, bounce 0.6s ease-in-out infinite; }
.pet--sleepy { animation: sway 4s ease-in-out infinite; opacity: 0.85; }
.pet--warning { animation: shake 0.3s ease-in-out infinite; }

/* —— 动画 —— */
@keyframes breathe {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

@keyframes sway {
  0%, 100% { transform: rotate(0deg); }
  25% { transform: rotate(-3deg); }
  75% { transform: rotate(3deg); }
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-2px); }
  75% { transform: translateX(2px); }
}

@keyframes wave-left {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-15deg); }
}

@keyframes wave-right {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(15deg); }
}
</style>
