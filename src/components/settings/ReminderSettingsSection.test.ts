import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ReminderSettingsSection from '@/components/settings/ReminderSettingsSection.vue'
import { REMINDER_PRESETS } from '@/config/reminderPresets'
import type { Reminder } from '@/types/settings'

const reminder: Reminder = {
  id: 'water-1',
  enabled: true,
  message: '喝水',
  schedule: { type: 'interval', interval_minutes: 30 },
  snooze_minutes: 5,
  paused_until: null,
}

function mountSection(overrides: Partial<InstanceType<typeof ReminderSettingsSection>['$props']> = {}) {
  return mount(ReminderSettingsSection, {
    props: {
      reminders: [reminder],
      draft: null,
      deleteTarget: null,
      intervalOptions: [30],
      snoozeOptions: [5],
      presets: REMINDER_PRESETS,
      formatSchedule: () => '每 30 分钟',
      formatPause: () => '',
      ...overrides,
    },
  })
}

describe('ReminderSettingsSection', () => {
  it('keeps one manual add control and emits create', async () => {
    const wrapper = mountSection()
    const addButtons = wrapper.findAll('button').filter((button) => button.text().includes('添加提醒'))

    expect(addButtons).toHaveLength(1)
    expect(wrapper.text()).not.toContain('添加另一条提醒')

    await addButtons[0]!.trigger('click')
    expect(wrapper.emitted('create')).toEqual([[]])
  })

  it('emits the correct reminder when deletion is requested', async () => {
    const wrapper = mountSection()
    const deleteButton = wrapper.findAll('button').find((button) => button.text() === '删除')

    await deleteButton!.trigger('click')

    expect(wrapper.emitted('requestDelete')).toEqual([[reminder]])
  })

  it('prevents another create action while an editor is open', () => {
    const wrapper = mountSection({
      draft: {
        id: null,
        message: '喝水',
        scheduleType: 'interval',
        intervalMinutes: 30,
        time: '09:00',
        repeatType: 'daily',
        weekdays: [1, 2, 3, 4, 5],
        snoozeMinutes: 5,
      },
    })

    const addButton = wrapper.findAll('button').find((button) => button.text().includes('添加提醒'))
    expect(addButton!.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).not.toContain('一键添加常用提醒')
  })
})
