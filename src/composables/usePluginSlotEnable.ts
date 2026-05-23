import type { DirectoryPluginCatalogEntry } from '../api'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginStore } from '../stores/pluginStore'

export interface UiSlotChoice {
  id: string
  label: string
}

export interface PluginSlotSelectorState {
  open: boolean
  pluginId: string
  pluginTitle: string
  slots: UiSlotChoice[]
  selected: string[]
}

function declaredUiSlotIds(entry: DirectoryPluginCatalogEntry): string[] {
  if (entry.isShell)
    return []
  const seen = new Set<string>()
  const out: string[] = []
  for (const name of entry.uiSlotNames ?? []) {
    const id = name.trim()
    if (id && !seen.has(id)) {
      seen.add(id)
      out.push(id)
    }
  }
  return out
}

/**
 * 启用目录插件：无 UI 插槽声明则直接启用；有声明则先弹出插槽位置选择。
 */
export function usePluginSlotEnable() {
  const { t } = useI18n()
  const pluginStore = usePluginStore()

  const selector = ref<PluginSlotSelectorState>({
    open: false,
    pluginId: '',
    pluginTitle: '',
    slots: [],
    selected: [],
  })

  function slotLabel(slotId: string): string {
    const key = `simplePluginManager.slots.${slotId}`
    const translated = t(key)
    return translated !== key ? translated : slotId
  }

  function openSelector(entry: DirectoryPluginCatalogEntry): void {
    const slotIds = declaredUiSlotIds(entry)
    const selected = slotIds.filter(
      s => !pluginStore.isSlotContributionDisabled(s, entry.id),
    )
    selector.value = {
      open: true,
      pluginId: entry.id,
      pluginTitle: entry.id,
      slots: slotIds.map(id => ({ id, label: slotLabel(id) })),
      selected: selected.length > 0 ? [...selected] : [...slotIds],
    }
  }

  function closeSelector(): void {
    selector.value = { ...selector.value, open: false }
  }

  function toggleSlotChoice(slotId: string): void {
    const set = new Set(selector.value.selected)
    if (set.has(slotId)) {
      set.delete(slotId)
    }
    else {
      set.add(slotId)
    }
    selector.value = { ...selector.value, selected: [...set] }
  }

  async function applySelectorAndEnable(): Promise<void> {
    const { pluginId, slots, selected } = selector.value
    if (!pluginId || selected.length === 0) {
      throw new Error(t('simplePluginManager.slotSelector.needOne'))
    }
    const entry = pluginStore.catalog.find(c => c.id === pluginId)
    if (entry && entry.dependencyStatus !== 'ok') {
      throw new Error(
        t('pluginWorkbench.errors.dependencyNotMet', {
          id: pluginId,
          issues: (entry.dependencyIssues ?? []).join('; '),
        }),
      )
    }
    pluginStore.setPluginDisabled(pluginId, false)
    for (const slotId of slots.map(s => s.id)) {
      const active = selected.includes(slotId)
      pluginStore.setSlotContributionDisabled(slotId, pluginId, !active)
    }
    for (const slotId of selected) {
      const rest = pluginStore
        .pluginsOrderedForSlot(slotId)
        .filter(id => id !== pluginId)
      pluginStore.setSlotPluginIds(slotId, [pluginId, ...rest])
    }
    await pluginStore.persist()
    closeSelector()
  }

  /** 设置启用/停用；启用且含 UI 插槽时返回 true 表示已打开选择器（尚未 persist）。 */
  async function setPluginEnabled(
    pluginId: string,
    enabled: boolean,
  ): Promise<boolean> {
    if (!enabled) {
      pluginStore.setPluginDisabled(pluginId, true)
      await pluginStore.persist()
      return false
    }
    const entry = pluginStore.catalog.find(c => c.id === pluginId)
    if (!entry) {
      pluginStore.setPluginDisabled(pluginId, false)
      await pluginStore.persist()
      return false
    }
    const slotIds = declaredUiSlotIds(entry)
    if (slotIds.length === 0) {
      pluginStore.setPluginDisabled(pluginId, false)
      await pluginStore.persist()
      return false
    }
    openSelector(entry)
    return true
  }

  return {
    selector,
    closeSelector,
    toggleSlotChoice,
    applySelectorAndEnable,
    setPluginEnabled,
  }
}
