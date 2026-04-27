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
  getPluginAuditLogs,
  setPluginPermissionGrant,
  type DirectoryPluginCatalogEntry,
  type PluginPermissionGrantDto,
  type PluginAuditLogRowDto,
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

const auditLoading = ref(false);
const auditError = ref<string | null>(null);
const auditLogs = ref<PluginAuditLogRowDto[]>([]);

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

const permEffective = computed(() => {
  const declared = declaredPermsSorted.value;
  const g = permSorted.value ?? [];
  const enabledMap = new Map<string, boolean>();
  for (const x of g) {
    enabledMap.set(x.permission, x.enabled === true);
  }
  const tokens = new Set<string>();
  for (const p of declared) tokens.add(p);
  for (const x of g) tokens.add(x.permission);
  const all = [...tokens].sort((a, b) => (a === b ? 0 : a < b ? -1 : 1));
  return all.map((permission) => ({
    permission,
    enabled: enabledMap.get(permission) === true,
    declared: declared.includes(permission),
  }));
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

async function refreshAudit(): Promise<void> {
  const pid = props.entry.id?.trim();
  if (!pid) return;
  auditLoading.value = true;
  auditError.value = null;
  try {
    const res = await getPluginAuditLogs(pid, 60);
    auditLogs.value = res.logs ?? [];
  } catch (e) {
    auditError.value = e instanceof Error ? e.message : String(e);
  } finally {
    auditLoading.value = false;
  }
}

onMounted(() => {
  void refreshPerms();
  void refreshAudit();
});

watch(
  () => props.entry.id,
  () => {
    void refreshPerms();
    void refreshAudit();
  },
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
      <p v-else-if="permEffective.length === 0" class="ipwd-perms-muted">
        暂无权限信息（可能为旧版本安装，或该插件未声明任何权限）。
      </p>
      <ul v-else class="ipwd-perms-list">
        <li v-for="p in permEffective" :key="p.permission" class="ipwd-perms-li">
          <label class="ipwd-perms-row">
            <input
              type="checkbox"
              :disabled="permLoading"
              :checked="p.enabled === true"
              @change="
                onTogglePermission(
                  { pluginId: entry.id, permission: p.permission, enabled: p.enabled },
                  ($event.target as HTMLInputElement).checked,
                )
              "
            />
            <span class="ipwd-perms-token">{{ p.permission }}</span>
            <span v-if="p.declared !== true" class="ipwd-perms-tag">额外</span>
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
    <div class="ipwd-audit">
      <div class="ipwd-audit-h">审计（最近）</div>
      <p v-if="auditError" class="ipwd-perms-err">{{ auditError }}</p>
      <p v-else-if="auditLoading" class="ipwd-perms-muted">加载中…</p>
      <p v-else-if="(auditLogs ?? []).length === 0" class="ipwd-perms-muted">
        暂无审计记录（只有在允许/拒绝调用时才会写入元数据）。
      </p>
      <ul v-else class="ipwd-audit-list">
        <li v-for="(x, idx) in auditLogs" :key="`${x.createdAt}-${idx}`" class="ipwd-audit-li">
          <span class="ipwd-audit-time">{{ x.createdAt }}</span>
          <span class="ipwd-audit-pill" :class="x.allowed ? 'ok' : 'deny'">
            {{ x.allowed ? "ALLOW" : "DENY" }}
          </span>
          <span class="ipwd-audit-action">{{ x.action }}</span>
          <span v-if="x.permission" class="ipwd-perms-token">{{ x.permission }}</span>
        </li>
      </ul>
      <p class="ipwd-perms-hint">仅记录元数据（不记录内容）。</p>
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
.ipwd-audit {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.ipwd-audit-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
.ipwd-audit-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ipwd-audit-li {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.ipwd-audit-time {
  color: var(--text-secondary);
}
.ipwd-audit-action {
  font-weight: 600;
}
.ipwd-audit-pill {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.ipwd-audit-pill.ok {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
}
.ipwd-audit-pill.deny {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
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
.ipwd-perms-tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
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
