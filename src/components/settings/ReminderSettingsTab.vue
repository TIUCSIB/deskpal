<script setup lang="ts">
/** ReminderSettingsTab.vue - 提醒设置、免打扰和活动记录标签页 */
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'
import QuietHoursSettings from '@/components/settings/QuietHoursSettings.vue'
import ReminderActivityList from '@/components/settings/ReminderActivityList.vue'
import ReminderSettingsSection from '@/components/settings/ReminderSettingsSection.vue'
import ReminderStatsSection from '@/components/settings/ReminderStatsSection.vue'
import SettingsActionRow from '@/components/settings/SettingsActionRow.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import { useReminderActivity } from '@/composables/useReminderActivity'
import type { ReminderSettings } from '@/composables/useSettingsWindow'
import type { AppSettings } from '@/types/settings'

const props = defineProps<{ settings: AppSettings; reminderSettings: ReminderSettings }>()
const { activity, clearActivity, loadActivity } = useReminderActivity()
const {
  intervalOptions, snoozeOptions, presets, draft, deleteTarget, quietHoursDraft,
  openCreateEditor, openEditEditor, cancelEditor, updateDraft, toggleWeekday, saveReminder,
  createPreset, saveQuietHours, setReminderEnabled, resumeReminder, previewReminder, requestDelete,
  cancelDelete, confirmDelete, formatSchedule, formatPause,
} = props.reminderSettings
</script>

<template>
  <ScrollArea class="h-full">
    <div class="grid gap-5 pr-2">
      <ReminderSettingsSection
        :reminders="props.settings.reminders"
        :draft="draft"
        :delete-target="deleteTarget"
        :interval-options="intervalOptions"
        :snooze-options="snoozeOptions"
        :presets="presets"
        :format-schedule="formatSchedule"
        :format-pause="formatPause"
        @create="openCreateEditor"
        @create-preset="createPreset"
        @edit="openEditEditor"
        @cancel="cancelEditor"
        @save="saveReminder"
        @update:draft-message="updateDraft('message', $event)"
        @update:draft-schedule-type="updateDraft('scheduleType', $event)"
        @update:draft-interval-minutes="updateDraft('intervalMinutes', $event)"
        @update:draft-time="updateDraft('time', $event)"
        @update:draft-repeat-type="updateDraft('repeatType', $event)"
        @toggle:weekday="toggleWeekday"
        @update:draft-snooze-minutes="updateDraft('snoozeMinutes', $event)"
        @update:enabled="setReminderEnabled"
        @preview="previewReminder"
        @resume="resumeReminder"
        @request-delete="requestDelete"
        @cancel-delete="cancelDelete"
        @confirm-delete="confirmDelete"
      />

      <SettingsSection title="免打扰">
        <QuietHoursSettings
          :draft="quietHoursDraft"
          @update:enabled="quietHoursDraft.enabled = $event"
          @update:start="quietHoursDraft.start = $event"
          @update:end="quietHoursDraft.end = $event"
          @save="saveQuietHours"
        />
      </SettingsSection>

      <SettingsSection title="提醒活动">
        <ReminderStatsSection :stats="activity.stats" />
        <ReminderActivityList :events="activity.events" :has-more-events="activity.hasMoreEvents" @show-all="loadActivity(true)" />
        <SettingsActionRow v-if="activity.events.length" align="start">
          <Button variant="outline" size="sm" class="rounded-lg" @click="clearActivity">清除活动记录</Button>
        </SettingsActionRow>
      </SettingsSection>
    </div>
  </ScrollArea>
</template>
