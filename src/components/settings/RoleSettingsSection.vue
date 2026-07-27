<script setup lang="ts">
/** RoleSettingsSection.vue - 桌宠角色选择 */
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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

const ROLE_ID = 'settings-pet-role'
</script>

<template>
  <SettingsSection title="角色">
    <div class="grid gap-2">
      <Label :for="ROLE_ID" class="text-sm leading-5 text-foreground">
        当前桌宠
      </Label>
      <Select
        :model-value="props.selectedRoleId"
        @update:model-value="(value) => value && emit('update:selected-role', value as PetRoleId)"
      >
        <SelectTrigger :id="ROLE_ID" class="h-10 w-full rounded-xl bg-background/70">
          <SelectValue placeholder="请选择桌宠角色" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="role in props.roles" :key="role.id" :value="role.id">
            {{ role.displayName }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-1 rounded-xl border border-border/70 bg-background/55 px-3 py-3">
      <div class="flex items-center justify-between gap-3">
        <strong class="text-sm font-medium text-foreground">{{ props.selectedRole.displayName }}</strong>
        <span class="text-xs text-muted-foreground">
          {{ props.selectedRole.kind === 'person' ? '人物' : '动物' }}
        </span>
      </div>
      <p class="m-0 text-xs leading-5 text-muted-foreground">
        {{ props.selectedRole.description }}
      </p>
    </div>
  </SettingsSection>
</template>
