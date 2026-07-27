import { computed } from 'vue'
import { getPetRole, PET_ROLES } from '@/config/petRoles'
import type { AppSettings } from '@/types/settings'
import type { PetRoleId } from '@/types/pet'

/** useRoleSettings - 角色选择设置 */
export function useRoleSettings(
  settings: { value: AppSettings },
  invokeSetting: (command: string, payload?: Record<string, unknown>) => Promise<AppSettings>,
  setFeedback: (text: string) => void,
) {
  const selectedRole = computed(() => getPetRole(settings.value.pet_role))

  async function handlePetRoleChange(role: PetRoleId) {
    const updated = await invokeSetting('set_pet_role', { role })
    setFeedback(`已切换为${getPetRole(updated.pet_role).displayName}`)
  }

  return {
    petRoles: PET_ROLES,
    selectedRole,
    handlePetRoleChange,
  }
}
