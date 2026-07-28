import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WINDOW_EVENTS, type SettingsSection } from '@/types/window'

/** 管理设置窗口的标签定位与键盘焦点。 */
export function useSettingsSectionFocus() {
  const activeSection = ref<SettingsSection>('display')
  const sectionRefs = new Map<SettingsSection, HTMLButtonElement>()
  let unlistenFocusSection: UnlistenFn | null = null

  function setSectionRef(section: SettingsSection, element: unknown) {
    const button = element instanceof HTMLButtonElement
      ? element
      : element && typeof element === 'object' && '$el' in element
        ? (element as { $el?: unknown }).$el
        : null
    if (button instanceof HTMLButtonElement) {
      sectionRefs.set(section, button)
      return
    }
    sectionRefs.delete(section)
  }

  async function focusSection(section: SettingsSection) {
    activeSection.value = section
    await nextTick()
    sectionRefs.get(section)?.focus()
  }

  onMounted(async () => {
    unlistenFocusSection = await listen<SettingsSection>(WINDOW_EVENTS.focusSettingsSection, (event) => {
      void focusSection(event.payload)
    })
  })

  onUnmounted(() => {
    unlistenFocusSection?.()
  })

  return { activeSection, setSectionRef }
}
