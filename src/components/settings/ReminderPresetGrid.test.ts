import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ReminderPresetGrid from '@/components/settings/ReminderPresetGrid.vue'
import { REMINDER_PRESETS } from '@/config/reminderPresets'

describe('ReminderPresetGrid', () => {
  it('renders an accessible icon card for every preset', () => {
    const wrapper = mount(ReminderPresetGrid, { props: { presets: REMINDER_PRESETS } })
    const buttons = wrapper.findAll('button')

    expect(buttons).toHaveLength(REMINDER_PRESETS.length)
    for (const preset of REMINDER_PRESETS) {
      const button = buttons.find((candidate) => candidate.attributes('aria-label') === `添加${preset.label}提醒`)
      expect(button).toBeDefined()
      expect(button!.find('svg').exists()).toBe(true)
    }
  })

  it('emits the selected preset', async () => {
    const wrapper = mount(ReminderPresetGrid, { props: { presets: REMINDER_PRESETS } })

    await wrapper.findAll('button')[0]!.trigger('click')

    expect(wrapper.emitted('select')).toEqual([[REMINDER_PRESETS[0]]])
  })
})
