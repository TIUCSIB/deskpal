import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import RoleSettingsSection from '@/components/settings/RoleSettingsSection.vue'
import { PET_ROLES } from '@/config/petRoles'

describe('RoleSettingsSection', () => {
  it('renders a selectable card for every role', () => {
    const wrapper = mount(RoleSettingsSection, {
      props: {
        selectedRoleId: 'guga',
        selectedRole: PET_ROLES[0]!,
        roles: PET_ROLES,
      },
    })

    const cards = wrapper.findAll('button[aria-pressed]')
    expect(cards).toHaveLength(PET_ROLES.length)
    expect(cards[0]!.attributes('aria-pressed')).toBe('true')
    expect(cards[1]!.attributes('aria-pressed')).toBe('false')
  })

  it('emits the selected role ID after a card is clicked', async () => {
    const wrapper = mount(RoleSettingsSection, {
      props: {
        selectedRoleId: 'guga',
        selectedRole: PET_ROLES[0]!,
        roles: PET_ROLES,
      },
    })

    await wrapper.findAll('button[aria-pressed]')[2]!.trigger('click')

    expect(wrapper.emitted('update:selected-role')).toEqual([['broom-witch']])
  })
})
