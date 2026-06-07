<script setup lang="ts">
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import UiSidePanel from '../../components/ui/UiSidePanel.vue'
import {
  ModelManagerPanel,
  SettingsView,
  SimplePluginManagerPanel,
} from '../../composables/useMainShell'

export type SidePanelTab = 'settings' | 'plugins' | 'models'
export type PluginsPanelSubview = 'list' | 'market'

const props = defineProps<{
  open: boolean
  activeTab: SidePanelTab
  pluginsSubview: PluginsPanelSubview
  settingsFocusTab?: 'general' | 'plugins' | 'storage' | null
}>()

const emit = defineEmits<{
  'close': []
  'update:activeTab': [tab: SidePanelTab]
  'update:pluginsSubview': [subview: PluginsPanelSubview]
  'openSettingsFromModels': []
}>()

const { t } = useI18n()

const panelTitle = computed(() => {
  if (props.activeTab === 'settings')
    return t('settings.title')
  if (props.activeTab === 'plugins') {
    return props.pluginsSubview === 'market'
      ? t('pluginWorkbench.market.pageTitle')
      : t('simplePluginManager.title')
  }
  return t('modelManager.title')
})

const outerTabs = computed<Array<{ id: SidePanelTab, label: string }>>(() => [
  { id: 'settings', label: t('settings.title') },
  { id: 'plugins', label: t('simplePluginManager.title') },
  { id: 'models', label: t('modelManager.title') },
])

function selectTab(tab: SidePanelTab): void {
  if (tab !== props.activeTab)
    emit('update:activeTab', tab)
}

watch(
  () => props.activeTab,
  (tab) => {
    if (tab !== 'plugins' && props.pluginsSubview !== 'list')
      emit('update:pluginsSubview', 'list')
  },
)
</script>

<template>
  <UiSidePanel
    :visible="open"
    :title="panelTitle"
    flush-body
    @close="emit('close')"
  >
    <template #tabs>
      <div
        class="tool-mgmt-subtabs"
        role="tablist"
        :aria-label="t('toolShell.sidePanelTabs')"
      >
        <button
          v-for="tab in outerTabs"
          :key="tab.id"
          type="button"
          role="tab"
          class="tool-mgmt-subtabs__btn"
          :class="{ 'tool-mgmt-subtabs__btn--active': activeTab === tab.id }"
          :aria-selected="activeTab === tab.id"
          @click="selectTab(tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>
    </template>

    <SettingsView
      v-if="activeTab === 'settings'"
      embedded
      :visible="open"
      :focus-tab="settingsFocusTab"
      @close="emit('close')"
    />
    <SimplePluginManagerPanel
      v-else-if="activeTab === 'plugins'"
      embedded
      :visible="open"
      :subview="pluginsSubview"
      @close="emit('close')"
      @update:subview="emit('update:pluginsSubview', $event)"
      @open-market="emit('update:pluginsSubview', 'market')"
    />
    <ModelManagerPanel
      v-else-if="activeTab === 'models'"
      embedded
      :visible="open"
      @close="emit('close')"
      @open-settings="emit('openSettingsFromModels')"
    />
  </UiSidePanel>
</template>
