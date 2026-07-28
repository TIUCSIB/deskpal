<script setup lang="ts">
/** ContextMenuRolePicker.vue - 右键菜单中的角色搜索与键盘选择列表。 */
import { computed, nextTick, ref, watch } from 'vue'
import { CheckIcon, ChevronLeftIcon, SearchIcon } from '@lucide/vue'
import Input from '@/components/ui/input/Input.vue'
import { petRoles } from '@/config/petRoles'
import type { PetRoleId } from '@/types/pet'

const props = defineProps<{
  selectedRole: PetRoleId
}>()

const emit = defineEmits<{
  back: []
  select: [roleId: PetRoleId]
}>()

const filterQuery = ref('')
const activeRoleId = ref<PetRoleId>(props.selectedRole)
const roleButtonRefs = new Map<PetRoleId, HTMLButtonElement>()
const filteredRoles = computed(() => {
  const query = filterQuery.value.trim().toLocaleLowerCase()
  if (!query) return petRoles.value
  return petRoles.value.filter(role => role.displayName.toLocaleLowerCase().includes(query))
})

function setRoleButtonRef(roleId: PetRoleId, element: unknown) {
  if (element instanceof HTMLButtonElement) {
    roleButtonRefs.set(roleId, element)
    return
  }
  roleButtonRefs.delete(roleId)
}

function focusRole(roleId: PetRoleId) {
  activeRoleId.value = roleId
  roleButtonRefs.get(roleId)?.focus()
}

function moveRole(direction: number) {
  const roles = filteredRoles.value
  if (!roles.length) return
  const currentIndex = Math.max(roles.findIndex(role => role.id === activeRoleId.value), 0)
  const nextIndex = (currentIndex + direction + roles.length) % roles.length
  focusRole(roles[nextIndex]!.id)
}

function handleRoleKeydown(event: KeyboardEvent, roleId: PetRoleId) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
    event.preventDefault()
    moveRole(1)
    return
  }
  if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
    event.preventDefault()
    moveRole(-1)
    return
  }
  if (event.key === 'Home') {
    event.preventDefault()
    const firstRole = filteredRoles.value[0]
    if (firstRole) focusRole(firstRole.id)
    return
  }
  if (event.key === 'End') {
    event.preventDefault()
    const lastRole = filteredRoles.value[filteredRoles.value.length - 1]
    if (lastRole) focusRole(lastRole.id)
    return
  }
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    emit('select', roleId)
  }
}

function handleSearchKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    const role = filteredRoles.value.find(role => role.id === activeRoleId.value) ?? filteredRoles.value[0]
    if (role) focusRole(role.id)
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    const role = filteredRoles.value[filteredRoles.value.length - 1]
    if (role) focusRole(role.id)
  }
}

function handleEscape() {
  if (filterQuery.value) {
    filterQuery.value = ''
    return
  }
  emit('back')
}

function handleAuxClick(event: MouseEvent) {
  if (event.button !== 3) return
  event.preventDefault()
  emit('back')
}

watch(filteredRoles, (roles) => {
  if (roles.some(role => role.id === activeRoleId.value)) return
  activeRoleId.value = roles[0]?.id ?? props.selectedRole
})

nextTick(() => {
  const selectedButton = roleButtonRefs.get(props.selectedRole)
  selectedButton?.scrollIntoView({ block: 'nearest' })
  selectedButton?.focus()
})
</script>

<template>
  <section class="role-picker" @keydown.esc.stop="handleEscape" @mouseup="handleAuxClick">
    <header class="role-picker__header">
      <button class="role-picker__back" type="button" aria-label="返回菜单" @click="emit('back')">
        <ChevronLeftIcon :size="16" aria-hidden="true" />
      </button>
      <h2 class="role-picker__title">切换角色</h2>
    </header>
    <label class="role-picker__search">
      <SearchIcon :size="14" aria-hidden="true" />
      <Input
        v-model="filterQuery"
        aria-label="搜索角色"
        placeholder="搜索角色"
        class="role-picker__search-input"
        @keydown="handleSearchKeydown"
      />
    </label>
    <div class="role-picker__list" role="radiogroup" aria-label="桌宠角色">
      <button
        v-for="role in filteredRoles"
        :key="role.id"
        :ref="element => setRoleButtonRef(role.id, element)"
        class="role-picker__role"
        :class="{ 'role-picker__role--selected': role.id === selectedRole }"
        type="button"
        role="radio"
        :tabindex="role.id === activeRoleId ? 0 : -1"
        :aria-checked="role.id === selectedRole"
        @focus="activeRoleId = role.id"
        @keydown="handleRoleKeydown($event, role.id)"
        @click="emit('select', role.id)"
      >
        <span class="role-picker__role-name">{{ role.displayName }}</span>
        <CheckIcon v-if="role.id === selectedRole" :size="15" aria-label="当前角色" />
      </button>
      <p v-if="!filteredRoles.length" class="role-picker__empty">未找到匹配的角色</p>
    </div>
  </section>
</template>

<style scoped>
.role-picker__header {
  display: flex;
  align-items: center;
  min-height: 28px;
  padding: 0 4px 3px;
}

.role-picker__back,
.role-picker__role {
  display: flex;
  align-items: center;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 7px;
  cursor: pointer;
}

.role-picker__back {
  justify-content: center;
  width: 24px;
  height: 24px;
}

.role-picker__back:focus-visible,
.role-picker__role:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: -2px;
}

.role-picker__title {
  margin: 0 0 0 5px;
  font-size: 12px;
  font-weight: 600;
  line-height: 16px;
}

.role-picker__search {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 4px 4px;
  color: var(--muted-foreground);
}

.role-picker__search-input {
  height: 26px;
  font-size: 12px;
}

.role-picker__list {
  display: grid;
  min-height: 0;
  max-height: 132px;
  gap: 1px;
  overflow-y: auto;
}

.role-picker__role {
  justify-content: space-between;
  width: 100%;
  min-height: 28px;
  gap: 8px;
  padding: 5px 8px;
  font-size: 12px;
  line-height: 16px;
}

.role-picker__role:hover,
.role-picker__role--selected {
  background: color-mix(in srgb, var(--accent) 88%, transparent);
}

.role-picker__role-name {
  overflow: hidden;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.role-picker__empty {
  margin: 0;
  padding: 16px 8px;
  color: var(--muted-foreground);
  font-size: 12px;
  text-align: center;
}
</style>
