<script setup lang="ts">
import PluginPrivateSettingsForm from "./PluginPrivateSettingsForm.vue";
import PluginDebugPanel from "./PluginDebugPanel.vue";
import PluginListItem from "./PluginListItem.vue";
import { useAppToast } from "../composables/useAppToast";
import {
  SLOT_CHAT_HEADER,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_PANEL,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";
import type { DirectoryPluginCatalogEntry } from "../utils/tauri-api";

defineProps<{
  entry: DirectoryPluginCatalogEntry;
  batchMode: boolean;
  batchSelected: boolean;
}>();

const emit = defineEmits<{
  "update:batchSelected": [value: boolean];
}>();

const pluginStore = usePluginStore();
const { showToast } = useAppToast();

function onPluginDisabledRow(id: string, disabled: boolean): void {
  try {
    pluginStore.setPluginDisabled(id, disabled);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}
</script>

<template>
  <div class="ipwd-root">
    <PluginListItem
      :entry="entry"
      :batch-select-mode="batchMode"
      :batch-selected="batchSelected"
      @update:batch-selected="emit('update:batchSelected', $event)"
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
      <div class="ipwd-settings-h">插件私有设置</div>
      <PluginPrivateSettingsForm :plugin-id="entry.id" />
    </div>
    <div class="ipwd-debug">
      <div class="ipwd-debug-h">调试台</div>
      <PluginDebugPanel
        :key="entry.id"
        :plugin-id="entry.id"
        :expanded="true"
        :spawn-supported="entry.hasRpcProcess"
      />
    </div>
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
</style>
