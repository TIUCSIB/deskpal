<script setup lang="ts">
/** RoleSettingsSection.vue - 桌宠角色与受控资源包管理。 */
import { Button } from '@/components/ui/button'
import PetRoleThumbnail from '@/components/settings/PetRoleThumbnail.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import { isBuiltInPetRole } from '@/config/petRoles'
import type { PetRole, PetRoleId } from '@/types/pet'

const props = defineProps<{
  selectedRoleId: PetRoleId
  roles: PetRole[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'update:selected-role': [PetRoleId]
  import: []
  remove: [PetRoleId]
}>()
</script>

<template>
  <SettingsSection title="角色">
    <div class="role-settings__intro">
      <p class="m-0 text-sm leading-5 text-muted-foreground">
        选择喜欢的桌宠角色，切换后会立即同步到桌面。
      </p>
      <Button class="rounded-xl" :disabled="props.loading" @click="emit('import')">
        {{ props.loading ? '正在导入…' : '导入角色资源包' }}
      </Button>
    </div>
    <p class="role-settings__hint">
      仅支持包含 manifest.json 和 PNG/WebP 精灵图的 .deskpal-role.zip 文件。
    </p>

    <div class="role-settings__list" aria-label="桌宠角色列表">
      <article
        v-for="role in props.roles"
        :key="role.id"
        class="role-settings__card"
        :class="{ 'role-settings__card--selected': role.id === props.selectedRoleId }"
      >
        <button
          class="role-settings__select"
          type="button"
          :aria-pressed="role.id === props.selectedRoleId"
          @click="emit('update:selected-role', role.id)"
        >
          <PetRoleThumbnail :role="role" />
          <span class="role-settings__content">
            <span class="role-settings__header">
              <strong class="role-settings__name">{{ role.displayName }}</strong>
              <!-- <span class="role-settings__kind">{{ formatRoleKind(role) }}</span> -->
              <span v-if="!isBuiltInPetRole(role.id)" class="role-settings__badge">已导入</span>
            </span>
            <span class="role-settings__description">{{ role.description }}</span>
          </span>
        </button>
        <Button
          v-if="!isBuiltInPetRole(role.id)"
          variant="ghost"
          size="sm"
          class="role-settings__remove"
          :disabled="props.loading"
          @click="emit('remove', role.id)"
        >
          删除
        </Button>
      </article>
    </div>
  </SettingsSection>
</template>

<style scoped>
.role-settings__intro {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.role-settings__hint {
  margin: -4px 0 0;
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 18px;
}

.role-settings__list {
  display: grid;
  gap: 8px;
}

.role-settings__card {
  display: flex;
  align-items: center;
  min-width: 0;
  color: inherit;
  background: color-mix(in srgb, var(--background) 55%, transparent);
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 12px;
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

.role-settings__card--selected {
  background: color-mix(in srgb, var(--primary) 11%, var(--background));
  border-color: color-mix(in srgb, var(--primary) 60%, var(--border));
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 22%, transparent);
}

.role-settings__select {
  display: flex;
  flex: 1;
  align-items: center;
  min-width: 0;
  gap: 12px;
  padding: 10px 8px 10px 12px;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  cursor: pointer;
}

.role-settings__select:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: -2px;
}

.role-settings__content {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.role-settings__header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.role-settings__name {
  overflow: hidden;
  font-size: 14px;
  font-weight: 500;
  line-height: 20px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.role-settings__badge {
  flex: 0 0 auto;
  padding: 1px 6px;
  color: var(--primary);
  font-size: 11px;
  line-height: 16px;
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  border-radius: 999px;
}

.role-settings__description {
  color: var(--muted-foreground);
  font-size: 12px;
  line-height: 18px;
}

.role-settings__remove {
  flex: 0 0 auto;
  margin-right: 8px;
  color: var(--destructive);
}
</style>
