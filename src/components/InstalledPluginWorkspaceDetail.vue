<script setup lang="ts">
import PluginPrivateSettingsForm from "./PluginPrivateSettingsForm.vue";
import PluginDebugPanel from "./PluginDebugPanel.vue";
import PluginListItem from "./PluginListItem.vue";
import { useAppToast } from "../composables/useAppToast";
import { computed, onMounted, ref, watch } from "vue";
import {
  SLOT_CHAT_HEADER,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_PANEL,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";
import {
  getPluginPermissionGrants,
  setPluginPermissionGrant,
  type DirectoryPluginCatalogEntry,
  type PluginPermissionGrantDto,
} from "../utils/tauri-api";

const props = defineProps<{
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

const permLoading = ref(false);
const permError = ref<string | null>(null);
const permGrants = ref<PluginPermissionGrantDto[]>([]);

const permSorted = computed(() =>
  [...(permGrants.value ?? [])].sort((a, b) =>
    a.permission === b.permission ? 0 : a.permission < b.permission ? -1 : 1,
  ),
);

const declaredPermsSorted = computed(() => {
  const raw = props.entry.installMeta?.declaredPermissions ?? [];
  return [...raw]
    .map((s) => (s ?? "").trim())
    .filter(Boolean)
    .sort((a, b) => (a === b ? 0 : a < b ? -1 : 1));
});

async function refreshPerms(): Promise<void> {
  const pid = props.entry.id?.trim();
  if (!pid) return;
  permLoading.value = true;
  permError.value = null;
  try {
    const res = await getPluginPermissionGrants(pid);
    permGrants.value = res.grants ?? [];
  } catch (e) {
    permError.value = e instanceof Error ? e.message : String(e);
  } finally {
    permLoading.value = false;
  }
}

onMounted(() => {
  void refreshPerms();
});

watch(
  () => props.entry.id,
  () => void refreshPerms(),
);

async function onTogglePermission(p: PluginPermissionGrantDto, enabled: boolean) {
  const pid = props.entry.id?.trim();
  if (!pid) return;
  permLoading.value = true;
  try {
    await setPluginPermissionGrant(pid, p.permission, enabled);
    await refreshPerms();
    showToast("success", "权限已更新。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    permLoading.value = false;
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
    <div class="ipwd-perms">
      <div class="ipwd-perms-h">权限</div>
      <div v-if="declaredPermsSorted.length > 0" class="ipwd-perms-declared">
        <div class="ipwd-perms-subh">声明（来自市场索引）</div>
        <ul class="ipwd-perms-list">
          <li v-for="p in declaredPermsSorted" :key="p" class="ipwd-perms-li">
            <span class="ipwd-perms-token">{{ p }}</span>
          </li>
        </ul>
        <p class="ipwd-perms-muted">
          这是插件作者在索引中声明的权限范围；真正是否可用以“已授予”为准。
        </p>
      </div>
      <p v-if="permError" class="ipwd-perms-err">{{ permError }}</p>
      <p v-else-if="permLoading" class="ipwd-perms-muted">加载中…</p>
      <p v-else-if="permSorted.length === 0" class="ipwd-perms-muted">
        暂无权限记录（可能尚未安装/未触发授权种子）。
      </p>
      <ul v-else class="ipwd-perms-list">
        <li v-for="p in permSorted" :key="p.permission" class="ipwd-perms-li">
          <label class="ipwd-perms-row">
            <input
              type="checkbox"
              :disabled="permLoading"
              :checked="p.enabled === true"
              @change="onTogglePermission(p, ($event.target as HTMLInputElement).checked)"
            />
            <span class="ipwd-perms-token">{{ p.permission }}</span>
          </label>
        </li>
      </ul>
      <p class="ipwd-perms-hint">
        关闭权限后，对应能力会被宿主拒绝（并记录审计元数据）。部分变更可能需要重启插件进程生效。
      </p>
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
.ipwd-perms {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.ipwd-perms-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
.ipwd-perms-subh {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 8px 0 6px;
}
.ipwd-perms-declared {
  margin-bottom: 10px;
}
.ipwd-perms-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ipwd-perms-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.ipwd-perms-token {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.ipwd-perms-muted {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 6px 0 0;
}
.ipwd-perms-err {
  font-size: 12px;
  color: var(--danger-600, #c0392b);
  margin: 6px 0 0;
}
.ipwd-perms-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 8px 0 0;
}
</style>
