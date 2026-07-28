import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  getPetRole,
  petRoles,
  replaceInstalledPetRoles,
  type InstalledPetRole,
} from '@/config/petRoles'
import type { AppSettings } from '@/types/settings'
import type { PetRoleId } from '@/types/pet'

/** useRoleSettings - 角色选择与受控资源包管理。 */
export function useRoleSettings(
  settings: { value: AppSettings },
  invokeSetting: (command: string, payload?: Record<string, unknown>) => Promise<AppSettings>,
  setFeedback: (text: string, isError?: boolean) => void,
) {
  const loading = ref(false)
  const selectedRole = computed(() => getPetRole(settings.value.pet_role))

  async function refreshRoles() {
    const roles = await invoke<InstalledPetRole[]>('list_installed_role_packs')
    replaceInstalledPetRoles(roles)
  }

  async function handlePetRoleChange(role: PetRoleId) {
    const updated = await invokeSetting('set_pet_role', { role })
    setFeedback(`已切换为${getPetRole(updated.pet_role).displayName}`)
  }

  async function installRolePack() {
    if (loading.value) return
    loading.value = true
    try {
      const role = await invoke<InstalledPetRole | null>('install_role_pack')
      if (!role) return
      await refreshRoles()
      setFeedback(`已导入角色包：${role.displayName}`)
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : '角色包导入失败'
      setFeedback(message, true)
    } finally {
      loading.value = false
    }
  }

  async function removeRolePack(role: PetRoleId) {
    if (settings.value.pet_role === role) {
      setFeedback('请先选择其他角色，再删除当前角色包')
      return
    }
    try {
      await invoke('remove_role_pack', { roleId: role })
      await refreshRoles()
      setFeedback('角色包已删除')
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : '角色包删除失败'
      setFeedback(message, true)
    }
  }

  onMounted(() => {
    void refreshRoles().catch((error: unknown) => {
      console.error('加载自定义角色失败:', error)
    })
  })

  return {
    petRoles,
    selectedRole,
    rolePackLoading: loading,
    handlePetRoleChange,
    installRolePack,
    removeRolePack,
    refreshRoles,
  }
}
