import { computed, ref } from 'vue'
import { getPetPersonality } from '@/config/petPersonalities'
import { DEFAULT_PET_ROLE } from '@/config/petRoles'
import type { DragDirection } from '@/composables/usePetInteraction'
import type { PetRoleId } from '@/types/pet'
import type { PetMood } from '@/types/system'
import { chooseWeighted } from '@/utils/weightedChoice'

const INTERACTION_RETURN_DELAY = 900
const PETTING_DELAY_MS = 900
const IDLE_ANIMATION_DELAY = 15000
const AMBIENT_ANIMATION_DELAY = 60000
const REQUIRED_ANIMATIONS = ['Idle', 'RunLeft', 'RunRight', 'Waving', 'Jumping', 'Failed']

/** usePetBehavior - 统一宠物动画状态与优先级 */
export function usePetBehavior(random: () => number = Math.random) {
  const mood = ref<PetMood>('normal')
  const roleId = ref<PetRoleId>(DEFAULT_PET_ROLE)
  const availableAnimations = ref<string[]>([...REQUIRED_ANIMATIONS, 'Waiting', 'Review', 'Running'])
  const hovering = ref(false)
  const petting = ref(false)
  const dragDirection = ref<DragDirection | null>(null)
  const activated = ref(false)
  const ambientAnimation = ref<string | null>(null)
  const animationRevision = ref(0)
  let idleTimer: ReturnType<typeof setTimeout> | null = null
  let ambientTimer: ReturnType<typeof setTimeout> | null = null
  let activationTimer: ReturnType<typeof setTimeout> | null = null
  let pettingTimer: ReturnType<typeof setTimeout> | null = null
  let scheduleGeneration = 0

  const personality = computed(() => getPetPersonality(roleId.value))

  /** 按交互优先级推导当前唯一动画 */
  const animationName = computed(() => {
    if (dragDirection.value === 'left') return resolveAnimation('RunLeft')
    if (dragDirection.value === 'right') return resolveAnimation('RunRight')
    if (activated.value) return resolveAnimation('Jumping')
    if (petting.value) return resolveAnimation('Waving')
    if (mood.value === 'warning') return resolveAnimation(personality.value.moodAnimations.warning)
    return resolveAnimation(ambientAnimation.value ?? personality.value.moodAnimations[mood.value])
  })

  function resolveAnimation(animationName: string) {
    if (availableAnimations.value.includes(animationName)) return animationName
    if (availableAnimations.value.includes('Idle')) return 'Idle'
    return availableAnimations.value[0] ?? 'Idle'
  }

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

  function clearPettingTimer() {
    if (!pettingTimer) return
    clearTimeout(pettingTimer)
    pettingTimer = null
  }

  function clearAmbientSchedule() {
    scheduleGeneration += 1
    clearIdleTimer()
    clearAmbientTimer()
    ambientAnimation.value = null
  }

  function canScheduleAmbient() {
    return !dragDirection.value && !activated.value && !petting.value && mood.value !== 'warning'
  }

  function scheduleAmbient() {
    clearAmbientSchedule()
    if (!canScheduleAmbient()) return
    const generation = scheduleGeneration
    idleTimer = setTimeout(() => {
      if (generation !== scheduleGeneration || !canScheduleAmbient()) return
      ambientAnimation.value = resolveAnimation('Waiting')
      scheduleAmbientLoop(generation)
    }, IDLE_ANIMATION_DELAY)
  }

  function scheduleAmbientLoop(generation: number) {
    clearAmbientTimer()
    ambientTimer = setTimeout(() => {
      if (generation !== scheduleGeneration || !canScheduleAmbient()) return
      const ambientMood = mood.value === 'warning' ? 'normal' : mood.value
      const choices = personality.value.ambientAnimations[ambientMood]
      const next = chooseWeighted(choices, random) ?? personality.value.moodAnimations[ambientMood]
      ambientAnimation.value = resolveAnimation(next)
      scheduleAmbientLoop(generation)
    }, AMBIENT_ANIMATION_DELAY)
  }

  /** 根据系统状态更新心情，交互期间不会抢占显示动画 */
  function setMood(nextMood: PetMood) {
    if (mood.value === nextMood) return
    mood.value = nextMood
    scheduleAmbient()
  }

  /** 同步角色与可用动作；缺失的角色动作会自动回退到 Idle */
  function setRole(nextRoleId: PetRoleId, animationNames: string[]) {
    roleId.value = nextRoleId
    availableAnimations.value = animationNames.length > 0 ? animationNames : ['Idle']
    resetForRoleChange()
  }

  /** 同步像素级悬停状态，持续悬停后才进入抚摸反馈 */
  function setHovering(nextHovering: boolean) {
    if (hovering.value === nextHovering) return
    hovering.value = nextHovering
    clearPettingTimer()

    if (!nextHovering) {
      petting.value = false
      scheduleAmbient()
      return
    }

    schedulePetting()
  }

  function schedulePetting() {
    clearPettingTimer()
    if (!hovering.value || dragDirection.value) return
    pettingTimer = setTimeout(() => {
      pettingTimer = null
      if (!hovering.value || dragDirection.value) return
      petting.value = true
      clearAmbientSchedule()
    }, PETTING_DELAY_MS)
  }

  /** 开始或更新拖拽状态 */
  function setDragging(direction: DragDirection | null) {
    if (dragDirection.value === direction) return
    dragDirection.value = direction
    if (direction) {
      clearPettingTimer()
      petting.value = false
      clearAmbientSchedule()
    } else if (hovering.value) {
      schedulePetting()
    } else {
      scheduleAmbient()
    }
  }

  /** 播放一次点击反馈动画，结束后按当前事实状态恢复 */
  function triggerClickFeedback() {
    activated.value = true
    clearActivationTimer()
    clearAmbientSchedule()
    activationTimer = setTimeout(() => {
      activated.value = false
      activationTimer = null
      scheduleAmbient()
    }, INTERACTION_RETURN_DELAY)
  }

  /** 兼容现有非点击交互调用 */
  function activate() {
    triggerClickFeedback()
  }

  /** 角色变更后清理旧的日常动画与计时器 */
  function resetForRoleChange() {
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
    clearPettingTimer()
  }

  return {
    mood,
    roleId,
    hovering,
    petting,
    animationName,
    animationRevision,
    setMood,
    setRole,
    setHovering,
    setDragging,
    triggerClickFeedback,
    activate,
    resetForRoleChange,
    start,
    dispose,
  }
}
