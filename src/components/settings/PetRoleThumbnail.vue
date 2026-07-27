<script setup lang="ts">
/** PetRoleThumbnail.vue - 角色精灵图静态缩略图 */
import { computed } from 'vue'
import type { PetAnimation, PetRole } from '@/types/pet'

const props = withDefaults(defineProps<{
  role: PetRole
  height?: number
}>(), {
  height: 72,
})

/** 获取用于缩略图的静态动画帧 */
const animation = computed<PetAnimation>(() => (
  props.role.spritesheet.animations.find(({ name }) => name === 'Idle')
  ?? props.role.spritesheet.animations[0]!
))

/** 计算缩略图缩放比例 */
const scale = computed(() => props.height / props.role.spritesheet.frameHeight)

/** 计算精灵图背景样式 */
const thumbnailStyle = computed(() => {
  const sheet = props.role.spritesheet
  const cropLeft = sheet.crop?.left ?? 0
  const cropTop = sheet.crop?.top ?? 0
  const cropRight = sheet.crop?.right ?? 0
  const cropBottom = sheet.crop?.bottom ?? 0
  const rowStride = sheet.frameHeight + sheet.rowGap
  const frameScale = scale.value

  return {
    width: `${(sheet.frameWidth - cropLeft - cropRight) * frameScale}px`,
    height: `${(sheet.frameHeight - cropTop - cropBottom) * frameScale}px`,
    backgroundImage: `url(${props.role.spritesheetUrl})`,
    backgroundPosition: `${-cropLeft * frameScale}px ${-(animation.value.row * rowStride + cropTop) * frameScale}px`,
    backgroundSize: `${sheet.imageWidth * frameScale}px ${sheet.imageHeight * frameScale}px`,
  }
})
</script>

<template>
  <span class="pet-role-thumbnail" :style="thumbnailStyle" aria-hidden="true" />
</template>

<style scoped>
.pet-role-thumbnail {
  display: block;
  flex: 0 0 auto;
  background-repeat: no-repeat;
}
</style>
