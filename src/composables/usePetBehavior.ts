import { computed, ref } from 'vue'
import type { DragDirection } from '@/composables/usePetInteraction'
import type { PetMood } from '@/types/system'

const INTERACTION_RETURN_DELAY = 900
const IDLE_ANIMATION_DELAY = 15000
const AMBIENT_ANIMATION_DELAY = 60000

const MOOD_ANIMATIONS: Record<PetMood, string> = {
  happy: 'Review',
  normal: 'Idle',
  sleepy: 'Waiting',
  warning: 'Failed',
}

const AMBIENT_ANIMATIONS: Record<PetMood, string[]> = {
  happy: ['Idle', 'Review', 'Waiting'],
  normal: ['Idle', 'Review', 'Waiting'],
  sleepy: ['Waiting', 'Idle', 'Review'],
  warning: ['Failed'],
}

/** usePetBehavior - 统一宠物动画状态与优先级 */
export function usePetBehavior() {
  const mood = ref<PetMood>('normal')
  const hovering = ref(false)
  const dragDirection = ref<DragDirection | null>(null)
  const activated = ref(false)
  const ambientAnimation = ref<string | null>(null)
  const ambientCursor = ref(0)
  const animationRevision = ref(0)
  let idleTimer: ReturnType<typeof setTimeout> | null = null
  let ambientTimer: ReturnType<typeof setTimeout> | null = null
  let activationTimer: ReturnType<typeof setTimeout> | null = null
  let scheduleGeneration = 0

  /** 按交互优先级推导当前唯一动画 */
  const animationName = computed(() => {
    if (dragDirection.value === 'left') return 'RunLeft'
    if (dragDirection.value === 'right') return 'RunRight'
    if (activated.value) return 'Jumping'
    if (hovering.value) return 'Waving'
    if (mood.value === 'warning') return MOOD_ANIMATIONS.warning
    return ambientAnimation.value ?? MOOD_ANIMATIONS[mood.value]
  })

  function clearIdleTimer() {
    if (!idleTimer) return
    clearTimeout(idleTimer)
    idleTimer = null
  }

  function clearAmbientTimer() {
    if (!ambientTimer) return
    clearTimeout(ambientTimer)
    ambientTimer = null
  }

  function clearActivationTimer() {
    if (!activationTimer) return
    clearTimeout(activationTimer)
    activationTimer = null
  }

  function clearAmbientSchedule() {
    scheduleGeneration += 1
    clearIdleTimer()
    clearAmbientTimer()
    ambientAnimation.value = null
  }

  function canScheduleAmbient() {
    return !dragDirection.value && !activated.value && !hovering.value && mood.value !== 'warning'
  }

  function scheduleAmbient() {
    clearAmbientSchedule()
    if (!canScheduleAmbient()) return
    const generation = scheduleGeneration
    idleTimer = setTimeout(() => {
      if (generation !== scheduleGeneration || !canScheduleAmbient()) return
      ambientAnimation.value = 'Waiting'
      scheduleAmbientLoop(generation)
    }, IDLE_ANIMATION_DELAY)
  }

  function scheduleAmbientLoop(generation: number) {
    clearAmbientTimer()
    ambientTimer = setTimeout(() => {
      if (generation !== scheduleGeneration || !canScheduleAmbient()) return
      const pool = AMBIENT_ANIMATIONS[mood.value]
      const next = pool[ambientCursor.value % pool.length] ?? MOOD_ANIMATIONS[mood.value]
      ambientCursor.value = (ambientCursor.value + 1) % pool.length
      ambientAnimation.value = next
      scheduleAmbientLoop(generation)
    }, AMBIENT_ANIMATION_DELAY)
  }

  /** 根据系统状态更新心情，交互期间不会抢占显示动画 */
  function setMood(nextMood: PetMood) {
    if (mood.value === nextMood) return
    mood.value = nextMood
    ambientCursor.value = 0
    scheduleAmbient()
  }

  /** 同步像素级悬停状态 */
  function setHovering(nextHovering: boolean) {
    if (hovering.value === nextHovering) return
    hovering.value = nextHovering
    if (nextHovering) clearAmbientSchedule()
    else scheduleAmbient()
  }

  /** 开始或更新拖拽状态 */
  function setDragging(direction: DragDirection | null) {
    if (dragDirection.value === direction) return
    dragDirection.value = direction
    if (direction) clearAmbientSchedule()
    else scheduleAmbient()
  }

  /** 播放一次点击激活动画，结束后按当前事实状态恢复 */
  function activate() {
    activated.value = true
    clearActivationTimer()
    clearAmbientSchedule()
    activationTimer = setTimeout(() => {
      activated.value = false
      activationTimer = null
      scheduleAmbient()
    }, INTERACTION_RETURN_DELAY)
  }

  /** 角色变更后清理旧的日常动画与计时器 */
  function resetForRoleChange() {
    ambientCursor.value = 0
    animationRevision.value += 1
    scheduleAmbient()
  }

  /** 初始化日常动画计时 */
  function start() {
    scheduleAmbient()
  }

  /** 清理行为计时器 */
  function dispose() {
    clearIdleTimer()
    clearAmbientTimer()
    clearActivationTimer()
  }

  return {
    mood,
    hovering,
    animationName,
    animationRevision,
    setMood,
    setHovering,
    setDragging,
    activate,
    resetForRoleChange,
    start,
    dispose,
  }
}
