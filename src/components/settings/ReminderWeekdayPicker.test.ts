import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ReminderWeekdayPicker from '@/components/settings/ReminderWeekdayPicker.vue'

describe('ReminderWeekdayPicker', () => {
  it('renders selected weekdays with the selected modifier class', () => {
    const wrapper = mount(ReminderWeekdayPicker, {
      props: { modelValue: [1, 3] },
    })
    const buttons = wrapper.findAll('button')

    expect(buttons[0]?.classes()).toContain('weekday-picker__day--selected')
    expect(buttons[0]?.attributes('aria-pressed')).toBe('true')
    expect(buttons[1]?.attributes('aria-pressed')).toBe('false')
    expect(buttons[2]?.classes()).toContain('weekday-picker__day--selected')
  })

  it('emits the weekday value when a button is clicked', async () => {
    const wrapper = mount(ReminderWeekdayPicker, {
      props: { modelValue: [] },
    })

    await wrapper.findAll('button')[4]!.trigger('click')

    expect(wrapper.emitted('toggle')).toEqual([[5]])
  })
})
