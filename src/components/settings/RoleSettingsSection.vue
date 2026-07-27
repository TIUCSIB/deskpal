<script setup lang="ts">
/** RoleSettingsSection.vue - 桌宠角色卡片选择 */
import PetRoleThumbnail from '@/components/settings/PetRoleThumbnail.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import type { PetRole, PetRoleId } from '@/types/pet'

const props = defineProps<{
  selectedRoleId: PetRoleId
  selectedRole: PetRole
  roles: PetRole[]
}>()

const emit = defineEmits<{
  'update:selected-role': [PetRoleId]
}>()

/** 返回角色类型的用户可见名称 */
function formatRoleKind(role: PetRole) {
  return role.kind === 'person' ? '人物' : '动物'
}
</script>

<template>
  <SettingsSection title="角色">
    <p class="m-0 text-sm leading-5 text-muted-foreground">
      选择喜欢的桌宠角色，切换后会立即同步到桌面。
    </p>

    <div class="role-settings__list" aria-label="桌宠角色列表">
      <button
        v-for="role in props.roles"
        :key="role.id"
        class="role-settings__card"
        :class="{ 'role-settings__card--selected': role.id === props.selectedRoleId }"
        type="button"
        :aria-pressed="role.id === props.selectedRoleId"
        @click="emit('update:selected-role', role.id)"
      >
        <PetRoleThumbnail :role="role" />
        <span class="role-settings__content">
          <span class="role-settings__header">
            <strong class="role-settings__name">{{ role.displayName }}</strong>
            <span class="role-settings__kind">{{ formatRoleKind(role) }}</span>
          </span>
          <span class="role-settings__description">{{ role.description }}</span>
        </span>
      </button>
    </div>
  </SettingsSection>
</template>

<style scoped>
.role-settings__list {
  display: grid;
  gap: 8px;
}

.role-settings__card {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  color: inherit;
  text-align: left;
  background: color-mix(in srgb, var(--background) 55%, transparent);
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 12px;
  cursor: pointer;
  transition:
    background-color 160ms ease,
    border-color 160ms ease,
    box-shadow 160ms ease,
    transform 160ms ease;
}

.role-settings__card:hover {
  background: color-mix(in srgb, var(--primary) 7%, var(--background));
  border-color: color-mix(in srgb, var(--primary) 38%, var(--border));
  transform: translateY(-1px);
}

.role-settings__card:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}

.role-settings__card--selected {
  background: color-mix(in srgb, var(--primary) 11%, var(--background));
  border-color: color-mix(in srgb, var(--primary) 60%, var(--border));
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 22%, transparent);
}

.role-settings__content {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.role-settings__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.role-settings__name {
  overflow: hidden;
  font-size: 14px;
  font-weight: 500;
  line-height: 20px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.role-settings__kind {
  flex: 0 0 auto;
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 16px;
}

.role-settings__description {
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 18px;
}
</style>
