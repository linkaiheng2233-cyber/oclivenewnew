<script setup lang="ts">
import type { DirectoryPluginCatalogEntry } from '../utils/tauri-api'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '../composables/useAppToast'
import {
  SLOT_CHAT_HEADER,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_PANEL,
  SLOT_SIDEBAR,
  usePluginStore,
} from '../stores/pluginStore'
import PluginDebugPanel from './PluginDebugPanel.vue'
import PluginListItem from './PluginListItem.vue'
import PluginPrivateSettingsForm from './PluginPrivateSettingsForm.vue'

defineProps<{
  entry: DirectoryPluginCatalogEntry
  batchMode: boolean
  batchSelected: boolean
}>()

const emit = defineEmits<{
  'update:batchSelected': [value: boolean]
}>()

const pluginStore = usePluginStore()
const { showToast } = useAppToast()
const { t } = useI18n()

function onPluginDisabledRow(id: string, disabled: boolean): void {
  try {
    pluginStore.setPluginDisabled(id, disabled)
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}
</script>

<template>
  <div class="ipwd-root">
    <PluginListItem
      :entry="entry"
      :batch-select-mode="batchMode"
      :batch-selected="batchSelected"
      :plugin-disabled="pluginStore.isPluginDisabled(entry.id)"
      :toolbar-contribution-disabled="pluginStore.isToolbarContributionDisabled(entry.id)"
      :settings-panel-contribution-disabled="
        pluginStore.isSlotContributionDisabled(SLOT_SETTINGS_PANEL, entry.id)
      "
      :role-detail-contribution-disabled="
        pluginStore.isSlotContributionDisabled(SLOT_ROLE_DETAIL, entry.id)
      "
      :sidebar-contribution-disabled="
        pluginStore.isSlotContributionDisabled(SLOT_SIDEBAR, entry.id)
      "
      :chat-header-contribution-disabled="
        pluginStore.isSlotContributionDisabled(SLOT_CHAT_HEADER, entry.id)
      "
      @update:batch-selected="emit('update:batchSelected', $event)"
      @update:plugin-disabled="onPluginDisabledRow(entry.id, $event)"
      @update:toolbar-contribution-disabled="
        pluginStore.setToolbarContributionDisabled(entry.id, $event)
      "
      @update:settings-panel-contribution-disabled="
        pluginStore.setSlotContributionDisabled(SLOT_SETTINGS_PANEL, entry.id, $event)
      "
      @update:role-detail-contribution-disabled="
        pluginStore.setSlotContributionDisabled(SLOT_ROLE_DETAIL, entry.id, $event)
      "
      @update:sidebar-contribution-disabled="
        pluginStore.setSlotContributionDisabled(SLOT_SIDEBAR, entry.id, $event)
      "
      @update:chat-header-contribution-disabled="
        pluginStore.setSlotContributionDisabled(SLOT_CHAT_HEADER, entry.id, $event)
      "
    />
    <div v-if="entry.hasUiSettings" class="ipwd-settings">
      <div class="ipwd-settings-h">
        {{ t("pluginManager.installed.privateSettings") }}
      </div>
      <PluginPrivateSettingsForm :plugin-id="entry.id" />
    </div>
    <details class="ipwd-advanced">
      <summary class="ipwd-advanced-sum">
        {{ t("pluginManager.installed.advanced") }}
      </summary>
      <div class="ipwd-debug">
        <div class="ipwd-debug-h">
          {{ t("pluginManager.installed.debugWorkbench") }}
        </div>
        <PluginDebugPanel
          :key="entry.id"
          :plugin-id="entry.id"
          :expanded="true"
          :spawn-supported="entry.hasRpcProcess"
        />
      </div>
    </details>
  </div>
</template>

<style scoped>
.ipwd-root {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}
.ipwd-settings {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.ipwd-settings-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}
.ipwd-debug {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.ipwd-debug-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
.ipwd-advanced {
  border-top: 1px dashed var(--border-light);
  padding-top: 8px;
}
.ipwd-advanced-sum {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
}
</style>
