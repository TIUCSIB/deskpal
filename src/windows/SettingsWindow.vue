<script setup lang="ts">
/**
 * SettingsWindow.vue - 托盘设置窗口
 * 使用 shadcn-vue 基础组件重构设置面板。
 */
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Slider } from '@/components/ui/slider'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ReminderSettingsSection from '@/components/settings/ReminderSettingsSection.vue'
import RoleSettingsSection from '@/components/settings/RoleSettingsSection.vue'
import SettingsActionRow from '@/components/settings/SettingsActionRow.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import SettingsShell from '@/components/settings/SettingsShell.vue'
import SettingsToggleRow from '@/components/settings/SettingsToggleRow.vue'
import { useSettingsWindow } from '@/composables/useSettingsWindow'
import type { InfoMode } from '@/types/settings'

const {
  settings,
  ready,
  scaleText,
  shortcutDraft,
  shortcutSummary,
  infoModeOptions,
  closeWindow,
  handleInfoModeChange,
  handleScaleChange,
  handleSizeLockedChange,
  handleShortcutEnabledChange,
  handleAlwaysOnTopChange,
  handleTaskbarChange,
  handleLaunchAtStartupChange,
  handleShortcutDraftInput,
  applyShortcut,
  restoreDefaultScale,
  resetPosition,
  resetSettingsWindowBounds,
  resetAllSettings,
  intervalOptions,
  snoozeOptions,
  reminderMessageDraft,
  handleReminderEnabledChange,
  handleReminderIntervalChange,
  handleReminderSnoozeChange,
  handleReminderDraftInput,
  applyReminderMessage,
  previewReminder,
  petRoles,
  selectedRole,
  handlePetRoleChange,
} = useSettingsWindow()

const INFO_MODE_ID = 'settings-info-mode'
const SCALE_ID = 'settings-pet-scale'
const SHORTCUT_ID = 'settings-shortcut'

function handleInfoModeValue(value: string) {
  handleInfoModeChange(value as InfoMode)
}

function handleScaleValue(value: number[] | undefined) {
  handleScaleChange(value?.[0] ?? settings.value.pet_scale)
}
</script>

<template>
  <SettingsShell :ready="ready" title="设置" @close="closeWindow">
    <template #loading>
      正在载入设置…
    </template>

    <Tabs default-value="display" orientation="horizontal" class="flex min-h-0 flex-1 flex-col gap-3">
      <TabsList variant="line" class="w-full justify-start border-b border-border/70 bg-transparent p-0">
        <TabsTrigger value="display" class="rounded-none px-4 py-2 text-[13px]">显示</TabsTrigger>
        <TabsTrigger value="size" class="rounded-none px-4 py-2 text-[13px]">大小</TabsTrigger>
        <TabsTrigger value="shortcut" class="rounded-none px-4 py-2 text-[13px]">快捷键</TabsTrigger>
        <TabsTrigger value="reminder" class="rounded-none px-4 py-2 text-[13px]">提醒</TabsTrigger>
        <TabsTrigger value="role" class="rounded-none px-4 py-2 text-[13px]">角色</TabsTrigger>
        <TabsTrigger value="system" class="rounded-none px-4 py-2 text-[13px]">系统</TabsTrigger>
      </TabsList>

      <TabsContent value="display" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <SettingsSection title="显示">
              <div class="grid gap-2">
                <Label :for="INFO_MODE_ID" class="text-sm leading-5 text-foreground">信息窗模式</Label>
                <Select :model-value="settings.info_mode" @update:model-value="(value) => value && handleInfoModeValue(String(value))">
                  <SelectTrigger :id="INFO_MODE_ID" class="h-10 w-full rounded-xl bg-background/70">
                    <SelectValue placeholder="请选择信息窗模式" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="option in infoModeOptions" :key="option.value" :value="option.value">
                      {{ option.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <SettingsToggleRow
                id="settings-always-on-top"
                label="桌宠窗口置顶"
                :checked="settings.main_window_always_on_top"
                @update:checked="handleAlwaysOnTopChange"
              />
              <SettingsToggleRow
                id="settings-show-taskbar"
                label="在任务栏显示"
                :checked="settings.main_window_show_in_taskbar"
                @update:checked="handleTaskbarChange"
              />
            </SettingsSection>
          </div>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="size" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <SettingsSection title="大小">
              <div class="grid gap-3">
                <div class="flex items-center justify-between gap-3">
                  <Label :for="SCALE_ID" class="text-sm leading-5 text-foreground">宠物缩放</Label>
                  <strong class="text-sm font-semibold text-foreground">{{ scaleText }}</strong>
                </div>
                <Slider
                  :id="SCALE_ID"
                  :model-value="[settings.pet_scale]"
                  :min="0.45"
                  :max="1.2"
                  :step="0.05"
                  @update:model-value="handleScaleValue"
                />
              </div>

              <SettingsToggleRow
                id="settings-size-locked"
                label="锁定大小"
                description="开启后将不再响应滚轮缩放。"
                :checked="settings.size_locked"
                @update:checked="handleSizeLockedChange"
              />

              <SettingsActionRow align="start">
                <Button variant="outline" class="rounded-xl" @click="restoreDefaultScale">
                  恢复默认大小
                </Button>
              </SettingsActionRow>
            </SettingsSection>
          </div>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="shortcut" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <SettingsSection title="快捷键">
              <SettingsToggleRow
                id="settings-shortcut-enabled"
                label="启用聊天快捷键"
                :checked="settings.shortcut_enabled"
                @update:checked="handleShortcutEnabledChange"
              />

              <div class="grid gap-2">
                <Label :for="SHORTCUT_ID" class="text-sm leading-5 text-foreground">快捷键组合</Label>
                <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                  <Input
                    :id="SHORTCUT_ID"
                    :model-value="shortcutDraft"
                    class="h-10 rounded-xl bg-background/70"
                    placeholder="例如 Ctrl+Alt+D"
                    @update:model-value="(value) => handleShortcutDraftInput(String(value))"
                  />
                  <Button class="h-10 rounded-xl px-4" @click="applyShortcut">应用</Button>
                </div>
              </div>

              <div class="grid gap-1 text-xs leading-5 text-muted-foreground">
                <p>支持写法示例：Ctrl+Alt+D、Shift+F1、Command+Option+D</p>
                <p>{{ shortcutSummary }}</p>
              </div>
            </SettingsSection>
          </div>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="reminder" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <ReminderSettingsSection
              :enabled="settings.reminder.enabled"
              :message-draft="reminderMessageDraft"
              :interval-minutes="settings.reminder.interval_minutes"
              :snooze-minutes="settings.reminder.snooze_minutes"
              :interval-options="intervalOptions"
              :snooze-options="snoozeOptions"
              @update:enabled="handleReminderEnabledChange"
              @update:message-draft="handleReminderDraftInput"
              @apply-message="applyReminderMessage"
              @update:interval="handleReminderIntervalChange"
              @update:snooze="handleReminderSnoozeChange"
              @preview="previewReminder"
            />
          </div>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="role" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <RoleSettingsSection
              :selected-role-id="settings.pet_role"
              :selected-role="selectedRole"
              :roles="petRoles"
              @update:selected-role="handlePetRoleChange"
            />
          </div>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="system" class="min-h-0 data-[state=inactive]:hidden">
        <ScrollArea class="h-full">
          <div class="pr-2">
            <SettingsSection title="系统">
              <SettingsToggleRow
                id="settings-launch-at-startup"
                label="开机自动启动"
                :checked="settings.launch_at_startup"
                @update:checked="handleLaunchAtStartupChange"
              />

              <SettingsActionRow wrap align="start">
                <Button variant="outline" class="rounded-xl" @click="resetPosition">重置桌宠位置</Button>
                <Button variant="outline" class="rounded-xl" @click="resetSettingsWindowBounds">
                  重置设置窗口位置和大小
                </Button>

                <AlertDialog>
                  <AlertDialogTrigger as-child>
                    <Button
                      variant="destructive"
                      class="rounded-xl border-destructive/30 bg-destructive text-white hover:bg-destructive/90 hover:text-white"
                    >
                      恢复全部默认设置
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>确定要恢复全部默认设置吗？</AlertDialogTitle>
                      <AlertDialogDescription>
                        这会重置桌宠位置、大小、快捷键、提醒和设置窗口布局。
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>取消</AlertDialogCancel>
                      <AlertDialogAction
                        variant="destructive"
                        class="border-destructive/30 bg-destructive text-white hover:bg-destructive/90 hover:text-white"
                        @click="resetAllSettings"
                      >
                        确认恢复
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </SettingsActionRow>
            </SettingsSection>
          </div>
        </ScrollArea>
      </TabsContent>
    </Tabs>
  </SettingsShell>
</template>
