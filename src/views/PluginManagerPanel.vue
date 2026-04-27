<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed, ref, watch } from "vue";
import PluginBackendSessionPanel from "../components/PluginBackendSessionPanel.vue";
import InstalledPluginWorkspaceDetail from "../components/InstalledPluginWorkspaceDetail.vue";
import PluginScaffoldWizard from "../components/PluginScaffoldWizard.vue";
import PmSlotRow from "../components/PmSlotRow.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import {
  SLOT_CHAT_HEADER,
  SLOT_CHAT_TOOLBAR,
  SLOT_DEBUG_DOCK,
  SLOT_LAUNCHER_PALETTE,
  SLOT_OVERLAY_FLOATING,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_ADVANCED,
  SLOT_SETTINGS_PANEL,
  SLOT_SETTINGS_PLUGINS,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import {
  applyAuthorSuggestedPluginBackends,
  packPlugin,
  previewPluginZipPermissions,
  type PluginMarketEntryDto,
} from "../utils/tauri-api";
import { getPluginMarketSourcesConfig } from "../utils/tauri-api";
import { OFFICIAL_UI_SLOTS } from "../lib/shellCapabilities";

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const { showToast } = useAppToast();

const marketSourceSelected = ref("official");
const marketSources = ref<string[]>([]);

const permConsentVisible = ref(false);
const permConsentTitle = ref("");
const permConsentPerms = ref<string[]>([]);
const permConsentSelected = ref<Record<string, boolean>>({});
const permConsentTrustSummary = ref<string>("");
let permConsentResolver: ((v: string[] | null) => void) | null = null;

function calcPermRisk(perms: string[]) {
  const hasNetwork = perms.some((p) => p === "network:*" || p.startsWith("network:"));
  const hasFs = perms.some((p) => p.startsWith("filesystem:"));
  const hasShell = perms.some((p) => p.startsWith("shell:") || p === "process:spawn");
  const hasRpcInvoke = perms.includes("rpc:invoke");
  return { hasNetwork, hasFs, hasShell, hasRpcInvoke };
}

async function requestPermissionConsent(
  title: string,
  declaredPerms: string[],
): Promise<string[] | null> {
  const perms = [...declaredPerms].map((s) => s.trim()).filter(Boolean);
  if (perms.length === 0) return [];
  permConsentTitle.value = title;
  permConsentPerms.value = perms;
  permConsentTrustSummary.value = "";
  const next: Record<string, boolean> = {};
  for (const p of perms) next[p] = true;
  permConsentSelected.value = next;
  permConsentVisible.value = true;
  return await new Promise<string[] | null>((resolve) => {
    permConsentResolver = resolve;
  });
}

async function requestPermissionConsentWithTrust(
  title: string,
  declaredPerms: string[],
  trustSummary: string,
): Promise<string[] | null> {
  const res = await requestPermissionConsent(title, declaredPerms);
  permConsentTrustSummary.value = trustSummary.trim();
  return res;
}

function onPermConsentCancel() {
  permConsentVisible.value = false;
  const r = permConsentResolver;
  permConsentResolver = null;
  permConsentTrustSummary.value = "";
  r?.(null);
}

function onPermConsentConfirm() {
  const selected = Object.entries(permConsentSelected.value)
    .filter(([, v]) => v)
    .map(([k]) => k);
  permConsentVisible.value = false;
  const r = permConsentResolver;
  permConsentResolver = null;
  permConsentTrustSummary.value = "";
  r?.(selected);
}

function setPermConsentAll(v: boolean) {
  const next: Record<string, boolean> = {};
  for (const p of permConsentPerms.value) next[p] = v;
  permConsentSelected.value = next;
}

async function loadMarketSourcesForPanel(): Promise<void> {
  try {
    const cfg = await getPluginMarketSourcesConfig();
    marketSources.value = (cfg.pluginIndexSources ?? []).filter((x) => !!x?.trim());
    // 非开发者模式时仅允许官方
    if (cfg.developerMode !== true) {
      marketSourceSelected.value = "official";
    } else if (
      marketSourceSelected.value !== "official" &&
      !marketSources.value.includes(marketSourceSelected.value)
    ) {
      marketSourceSelected.value = marketSources.value[0] ?? "official";
    }
  } catch {
    // ignore; fall back to official
    marketSources.value = [];
    marketSourceSelected.value = "official";
  }
}

watch(
  () => pluginStore.panelVisible,
  (vis) => {
    if (vis) {
      void pluginStore.loadCachedPluginMarket();
      void loadMarketSourcesForPanel();
    }
  },
);

const batchMode = ref(false);
const batchSelected = ref<Record<string, boolean>>({});
const scaffoldWizardVisible = ref(false);
const pluginPackStatus = ref("");
/** 已安装区：侧栏当前选中（右侧单一配置 + 调试台） */
const selectedWorkspacePluginId = ref("");

const selectedWorkspacePlugin = computed(() =>
  pluginStore.catalog.find((c) => c.id === selectedWorkspacePluginId.value) ?? null,
);

const supportedUiSlotsForShell = computed(() => pluginStore.supportedUiSlots ?? []);
const unsupportedOfficialUiSlots = computed(() => {
  const supported = new Set(supportedUiSlotsForShell.value);
  return OFFICIAL_UI_SLOTS.filter((s) => !supported.has(s));
});

function selectWorkspacePlugin(id: string): void {
  selectedWorkspacePluginId.value = id;
}

function clearBatchSelection(): void {
  batchSelected.value = {};
}

watch(batchMode, (v) => {
  if (!v) {
    clearBatchSelection();
  }
});

watch(
  () => pluginStore.catalog.map((c) => c.id).join("\n"),
  () => {
    const next: Record<string, boolean> = {};
    for (const p of pluginStore.catalog) {
      if (batchSelected.value[p.id]) {
        next[p.id] = true;
      }
    }
    batchSelected.value = next;

    const ids = pluginStore.catalog.map((c) => c.id);
    if (ids.length === 0) {
      selectedWorkspacePluginId.value = "";
      return;
    }
    if (
      !selectedWorkspacePluginId.value ||
      !ids.includes(selectedWorkspacePluginId.value)
    ) {
      selectedWorkspacePluginId.value = ids[0] ?? "";
    }
  },
  { immediate: true },
);

const batchSelectedCount = computed(
  () => Object.values(batchSelected.value).filter(Boolean).length,
);

const batchSelectedIds = computed(() =>
  Object.entries(batchSelected.value)
    .filter(([, v]) => v)
    .map(([k]) => k),
);

function setBatchSelected(id: string, v: boolean): void {
  batchSelected.value = { ...batchSelected.value, [id]: v };
}

async function onBatchEnable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    pluginStore.batchEnablePluginIds(ids);
    showToast("success", `已启用 ${ids.length} 个插件；保存后生效，建议重启应用。`);
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onBatchDisable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  pluginStore.batchDisablePluginIds(ids);
  showToast("success", `已停用 ${ids.length} 个插件；保存后生效，建议重启应用。`);
  clearBatchSelection();
}

async function onBatchUpdate() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    await pluginStore.batchUpdatePluginsFromGitIndex(ids);
    showToast("success", "已从索引 Git 源拉取更新（ff-only）；若失败请查看错误提示。");
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onSyncMarketIndex() {
  try {
    await pluginStore.syncPluginMarket(
      marketSourceSelected.value === "official" ? null : marketSourceSelected.value,
    );
    if (pluginStore.pluginMarketSnapshot?.warning) {
      showToast("info", pluginStore.pluginMarketSnapshot.warning);
    } else {
      showToast("success", "索引已同步。");
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallMarketEntry(row: PluginMarketEntryDto) {
  if ((row.missingDependencies ?? []).length > 0) {
    showToast(
      "error",
      `依赖未满足，无法安装：${row.missingDependencies.join("、")}`,
    );
    return;
  }
  const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
  const trust = [
    row.source ? `来源：${row.source}` : "",
    row.publisher ? `发布者：${row.publisher}` : "",
    (row.publicKeys ?? []).length
      ? `公钥：${(row.publicKeys ?? [])
          .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
          .join("，")}`
      : "",
  ]
    .filter(Boolean)
    .join("\n");
  const accepted = await requestPermissionConsentWithTrust(
    `安装 ${row.id}`,
    declaredPerms,
    trust,
  );
  if (accepted == null) return;
  const { hasNetwork, hasFs, hasShell, hasRpcInvoke } = calcPermRisk(accepted);
  if ((hasNetwork && hasFs && hasShell) || hasRpcInvoke) {
    const ok2 = window.confirm(
      `你已勾选高风险权限（${[
        hasRpcInvoke ? "rpc:invoke" : "",
        hasNetwork && hasFs && hasShell ? "network+filesystem+shell" : "",
      ]
        .filter(Boolean)
        .join("，")}）。\n\n建议仅安装你信任的来源。\n\n请再次确认：是否继续安装？`,
    );
    if (!ok2) return;
  }
  try {
    // 默认安装走索引内版本解析（git tag clone）；仅开发者模式才应允许自定义 gitUrl 覆盖
    await pluginStore.installFromPluginMarket(row.id, null, accepted);
    showToast("success", `已安装 ${row.id}，建议保存配置并视需要重启应用。`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const marketPickedVersion = ref<Record<string, string>>({});

function marketVersionsForRow(row: PluginMarketEntryDto): string[] {
  const vs = (row.versions ?? []).map((x) => x.version).filter((x) => !!x?.trim());
  // 降序展示：优先 semver 解析失败时按字符串倒序
  return [...vs].sort((a, b) => (a === b ? 0 : a < b ? 1 : -1));
}

function marketPickedVersionForRow(row: PluginMarketEntryDto): string {
  const pid = row.id;
  const picked = marketPickedVersion.value[pid]?.trim();
  if (picked) return picked;
  const vs = marketVersionsForRow(row);
  return vs[0] ?? row.version;
}

async function onInstallMarketVersion(row: PluginMarketEntryDto) {
  const v = marketPickedVersionForRow(row);
  if (!v?.trim()) return;
  const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
  const trust = [
    row.source ? `来源：${row.source}` : "",
    row.publisher ? `发布者：${row.publisher}` : "",
    (row.publicKeys ?? []).length
      ? `公钥：${(row.publicKeys ?? [])
          .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
          .join("，")}`
      : "",
  ]
    .filter(Boolean)
    .join("\n");
  const accepted = await requestPermissionConsentWithTrust(
    `安装 ${row.id} v${v}`,
    declaredPerms,
    trust,
  );
  if (accepted == null) return;
  const { hasNetwork, hasFs, hasShell, hasRpcInvoke } = calcPermRisk(accepted);
  if ((hasNetwork && hasFs && hasShell) || hasRpcInvoke) {
    const ok2 = window.confirm(
      `你已勾选高风险权限（${[
        hasRpcInvoke ? "rpc:invoke" : "",
        hasNetwork && hasFs && hasShell ? "network+filesystem+shell" : "",
      ]
        .filter(Boolean)
        .join("，")}）。\n\n建议仅安装你信任的来源。\n\n请再次确认：是否继续安装 v${v}？`,
    );
    if (!ok2) return;
  }
  try {
    await pluginStore.installVersionFromPluginMarket(row.id, v, accepted);
    showToast(
      "success",
      row.installed ? `已回滚/切换 ${row.id} → v${v}` : `已安装 ${row.id} v${v}`,
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateMarketEntry(row: PluginMarketEntryDto) {
  try {
    await pluginStore.updateInstalledPluginFromGit(row.id);
    showToast("success", `已更新 ${row.id}（git pull --ff-only）。`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onGitPullWorkspacePlugin() {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) return;
  try {
    await pluginStore.updateInstalledPluginFromGit(pid);
    showToast("success", "已从远程 Git 拉取更新。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const toolbarOrder = computed(() => pluginStore.toolbarPluginsOrdered());
const settingsPanelOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PANEL),
);
const roleDetailOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_ROLE_DETAIL));
const sidebarOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_SIDEBAR));
const chatHeaderOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_CHAT_HEADER));
const settingsPluginsOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PLUGINS),
);
const settingsAdvancedOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_ADVANCED),
);
const overlayFloatingOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_OVERLAY_FLOATING),
);
const launcherPaletteOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_LAUNCHER_PALETTE),
);
const debugDockOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_DEBUG_DOCK));

let dragSlot: { slot: string; index: number } | null = null;

function onDragSlotStart(slot: string, index: number) {
  dragSlot = { slot, index };
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDropSlot(slot: string, index: number) {
  if (!dragSlot || dragSlot.slot !== slot) {
    dragSlot = null;
    return;
  }
  if (dragSlot.index === index) {
    dragSlot = null;
    return;
  }
  pluginStore.movePluginInSlotOrder(slot, dragSlot.index, index);
  dragSlot = null;
}

async function onSave() {
  try {
    await pluginStore.persist();
    showToast("success", "已保存插件配置；停用插件建议重启应用后完全生效。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onResetToPackDefault() {
  try {
    if (pluginStore.persistScope === "global") {
      pluginStore.setPersistScope("role");
    }
    await pluginStore.resetToRolePackDefault();
    showToast(
      "success",
      "已重置为当前角色包推荐布局（author.suggested_ui 优先，否则 ui.json）。",
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onApplyAuthorSuggestedBackends() {
  try {
    const info = await applyAuthorSuggestedPluginBackends(roleStore.currentRoleId);
    roleStore.applyRoleInfo(info);
    showToast("success", "已应用 author.json 中的 suggested_plugin_backends（会话级，未改 settings.json）。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onCheckUpdates() {
  try {
    await pluginStore.checkPluginUpdatesFromRegistry();
    if (pluginStore.error) {
      showToast("error", pluginStore.error);
    } else {
      showToast("success", "检查完成（在线版本接口预留中）。");
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateFromZip(pluginId: string) {
  const path = await open({
    multiple: false,
    filters: [{ name: "Zip", extensions: ["zip"] }],
  });
  if (path === null || Array.isArray(path)) {
    return;
  }
  try {
    const preview = await previewPluginZipPermissions(path);
    if (preview.pluginId.trim() !== pluginId.trim()) {
      showToast(
        "error",
        `zip 内 manifest.id=${preview.pluginId} 与目标插件 ${pluginId} 不一致`,
      );
      return;
    }
    const accepted = await requestPermissionConsent(
      `侧载更新 ${pluginId}`,
      preview.permissions ?? [],
    );
    if (accepted == null) return;
    await pluginStore.installPluginFromLocalZip(pluginId, path, accepted);
    showToast("success", "更新完成，请重启应用生效。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onPackSelectedPlugin(): Promise<void> {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) {
    pluginPackStatus.value = "请先在目录中选择一个插件。";
    return;
  }
  try {
    const r = await packPlugin(pid);
    pluginPackStatus.value = `打包完成：${r.archive_path}`;
  } catch (e) {
    pluginPackStatus.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="pluginStore.panelVisible"
      class="pm-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件工作台（专业模式）"
      @click.self="pluginStore.closePanel()"
    >
      <div
        v-if="permConsentVisible"
        class="pm-modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-label="插件安装权限确认"
        @click.self="onPermConsentCancel"
      >
        <div class="pm-modal" @click.stop>
          <div class="pm-modal-h">{{ permConsentTitle }}</div>
          <p v-if="permConsentTrustSummary" class="pm-trust-summary">
            <span class="pm-trust-h">信任摘要</span>
            <br />
            <span class="pm-trust-mono" style="white-space: pre-wrap">{{
              permConsentTrustSummary
            }}</span>
          </p>
          <p class="pm-hint">
            请选择你愿意授予的权限（安装后仍可在“已安装插件 → 权限”中随时调整）。
          </p>
          <div class="pm-modal-actions">
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="setPermConsentAll(true)"
            >
              全选
            </button>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="setPermConsentAll(false)"
            >
              全不选
            </button>
          </div>
          <ul class="pm-perm-list">
            <li v-for="p in permConsentPerms" :key="p" class="pm-perm-li">
              <label class="pm-perm-row">
                <input
                  type="checkbox"
                  :checked="permConsentSelected[p] === true"
                  @change="
                    permConsentSelected = {
                      ...permConsentSelected,
                      [p]: ($event.target as HTMLInputElement).checked,
                    }
                  "
                />
                <span class="pm-perm-token">{{ p }}</span>
              </label>
            </li>
          </ul>
          <div class="pm-modal-actions pm-modal-actions--foot">
            <button type="button" class="pm-btn secondary" @click="onPermConsentCancel">
              取消
            </button>
            <button type="button" class="pm-btn" @click="onPermConsentConfirm">继续安装</button>
          </div>
        </div>
      </div>
      <div class="pm-dialog pm-dialog--studio" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">插件工作台</h2>
            <span
              class="pm-studio-badge"
              title="面向创作者与排错：目录插件、后端与会话覆盖"
            >专业模式</span>
          </div>
          <p class="pm-sub">
            <kbd class="pm-kbd">Ctrl</kbd>+<kbd class="pm-kbd">Shift</kbd>+<kbd class="pm-kbd">F</kbd>
            开关本窗口 · 保存后插槽/启用状态建议重启应用生效
          </p>
          <button type="button" class="pm-close" aria-label="关闭" @click="pluginStore.closePanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted pm-dialog-pad">加载中…</div>
        <p v-else-if="pluginStore.error" class="pm-err pm-dialog-pad">{{ pluginStore.error }}</p>

        <template v-else>
          <div class="pm-tabs" role="tablist" aria-label="插件工作台分区">
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'plugins' }"
              :aria-selected="pluginStore.panelMainTab === 'plugins'"
              @click="pluginStore.panelMainTab = 'plugins'"
            >
              界面插件
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'backends' }"
              :aria-selected="pluginStore.panelMainTab === 'backends'"
              @click="pluginStore.panelMainTab = 'backends'"
            >
              后端模块
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'slots' }"
              :aria-selected="pluginStore.panelMainTab === 'slots'"
              @click="pluginStore.panelMainTab = 'slots'"
            >
              插槽顺序
            </button>
          </div>

          <div class="pm-scroll">
          <div
            v-show="pluginStore.panelMainTab === 'plugins'"
            class="pm-tab-panel"
            role="tabpanel"
          >
          <section class="pm-section">
            <h3 class="pm-h3">保存目标</h3>
            <p class="pm-hint">
              「全局默认」对所有角色生效并与各角色设置合并（整壳与插槽以当前角色为准；全局禁用插件为并集）。
            </p>
            <div class="pm-scope-row" role="group" aria-label="插件配置保存范围">
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'role'"
                  @change="pluginStore.setPersistScope('role')"
                />
                仅当前角色
              </label>
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'global'"
                  @change="pluginStore.setPersistScope('global')"
                />
                全局默认
              </label>
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">Shell 能力（Module 8）</h3>
            <p class="pm-hint">
              插槽属于前端壳能力集；插件可按能力渲染/降级。后端 bootstrap 会返回本发行版支持的插槽名。
            </p>
            <p class="pm-muted" v-if="supportedUiSlotsForShell.length === 0">
              未提供 supportedUiSlots（可能是旧版内核/后端），将按“全支持”兼容处理。
            </p>
            <div v-else class="pm-shell-slots">
              <div class="pm-shell-slots-row">
                <span class="pm-muted">支持：</span>
                <span class="pm-shell-chip" v-for="s in supportedUiSlotsForShell" :key="`sup-${s}`">
                  {{ s }}
                </span>
              </div>
              <div v-if="unsupportedOfficialUiSlots.length > 0" class="pm-shell-slots-row">
                <span class="pm-muted">不支持（官方插槽）：</span>
                <span
                  class="pm-shell-chip pm-shell-chip--warn"
                  v-for="s in unsupportedOfficialUiSlots"
                  :key="`unsup-${s}`"
                >
                  {{ s }}
                </span>
              </div>
            </div>
          </section>

          <section
            v-if="roleStore.roleInfo.authorPack?.suggested_plugin_backends"
            class="pm-section"
          >
            <h3 class="pm-h3">作者建议 · 后端</h3>
            <p class="pm-hint">
              将 author.json 中的 suggested_plugin_backends 写入本会话的后端覆盖（与「后端模块」Tab 一致）。
            </p>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="onApplyAuthorSuggestedBackends"
            >
              应用作者建议的后端
            </button>
          </section>

          <section v-if="roleStore.roleInfo.authorPack" class="pm-section">
            <h3 class="pm-h3">作者与推荐</h3>
            <p v-if="roleStore.roleInfo.authorPack.summary" class="pm-author-summary">
              {{ roleStore.roleInfo.authorPack.summary }}
            </p>
            <ul
              v-if="(roleStore.roleInfo.authorPack.recommended_plugins ?? []).length"
              class="pm-rec-list"
            >
              <li
                v-for="(rp, idx) in roleStore.roleInfo.authorPack.recommended_plugins"
                :key="`${rp.id}-${idx}`"
              >
                <strong>{{ rp.id }}</strong>
                <span v-if="rp.version_range" class="pm-muted"> · {{ rp.version_range }}</span>
                <span v-if="rp.optional" class="pm-muted">（可选）</span>
              </li>
            </ul>
            <p v-else class="pm-muted">未列出 recommended_plugins。</p>
          </section>

          <section class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">社区索引</h3>
              <div class="pm-section-actions">
                <select
                  class="pm-select pm-select--sm"
                  :value="marketSourceSelected"
                  @change="marketSourceSelected = ($event.target as HTMLSelectElement).value"
                >
                  <option value="official">官方默认索引</option>
                  <option
                    v-for="s in marketSources"
                    :key="s"
                    :value="s"
                  >
                    第三方源 · {{ s }}
                  </option>
                </select>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginMarketSyncing"
                  @click="onSyncMarketIndex"
                >
                  {{ pluginStore.pluginMarketSyncing ? "同步中…" : "同步在线索引" }}
                </button>
              </div>
            </div>
            <p v-if="pluginStore.pluginMarketError" class="pm-err">{{ pluginStore.pluginMarketError }}</p>
            <p
              v-else-if="pluginStore.pluginMarketSnapshot?.warning"
              class="pm-hint"
            >
              {{ pluginStore.pluginMarketSnapshot.warning }}
            </p>
            <p v-if="pluginStore.pluginMarketSnapshot?.offlineMode" class="pm-hint">
              当前为离线模式（使用本地缓存索引）。
            </p>
            <p
              v-if="marketSourceSelected !== 'official'"
              class="pm-err"
            >
              当前为第三方索引源。请仅安装你信任的来源，并谨慎授予权限（开发者模式功能）。
            </p>
            <p
              v-if="
                !pluginStore.pluginMarketSnapshot?.plugins?.length &&
                !pluginStore.pluginMarketError
              "
              class="pm-muted"
            >
              尚无索引数据，请点击「同步在线索引」。
            </p>
            <ul
              v-else-if="(pluginStore.pluginMarketSnapshot?.plugins?.length ?? 0) > 0"
              class="pm-market-list"
            >
              <li
                v-for="row in pluginStore.pluginMarketSnapshot!.plugins"
                :key="row.id"
                class="pm-market-li"
              >
                <div class="pm-market-main">
                  <strong>{{ row.id }}</strong>
                  <span
                    class="pm-source-badge"
                    :class="(row.source ?? '') === 'official' ? 'official' : 'third'"
                    :title="(row.source ?? '') === 'official' ? '官方默认索引' : '第三方索引源'"
                  >
                    {{ (row.source ?? "") === "official" ? "官方" : "第三方" }}
                  </span>
                  <span class="pm-muted"> · {{ row.name }} · v{{ row.version }}</span>
                  <p v-if="row.source || row.publisher" class="pm-market-trust">
                    <span v-if="row.source" class="pm-muted">来源：{{ row.source }}</span>
                    <span v-if="row.publisher" class="pm-muted"> · 发布者：{{ row.publisher }}</span>
                    <span
                      v-if="(row.publicKeys ?? []).length"
                      class="pm-muted"
                      title="索引登记的公钥状态"
                    >
                      · 公钥：{{
                        (row.publicKeys ?? [])
                          .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
                          .join("，")
                      }}
                    </span>
                  </p>
                  <p v-if="row.description" class="pm-market-desc">{{ row.description }}</p>
                  <p
                    v-if="(row.missingDependencies ?? []).length"
                    class="pm-err pm-market-deps"
                  >
                    依赖缺失：{{ row.missingDependencies.join("、") }}
                  </p>
                </div>
                <div class="pm-market-actions">
                  <div
                    v-if="(row.versions ?? []).length > 0"
                    class="pm-market-versions"
                  >
                    <select
                      class="pm-select pm-select--sm"
                      :value="marketPickedVersionForRow(row)"
                      @change="
                        (e) =>
                          (marketPickedVersion = {
                            ...marketPickedVersion,
                            [row.id]: (e.target as HTMLSelectElement).value,
                          })
                      "
                    >
                      <option
                        v-for="v in marketVersionsForRow(row)"
                        :key="`${row.id}-${v}`"
                        :value="v"
                      >
                        v{{ v }}
                      </option>
                    </select>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onInstallMarketVersion(row)"
                    >
                      {{ row.installed ? "回滚/切换" : "安装此版本" }}
                    </button>
                  </div>
                  <button
                    v-if="!row.installed"
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="onInstallMarketEntry(row)"
                  >
                    安装
                  </button>
                  <template v-else>
                    <span v-if="row.hasUpdate" class="pm-badge">可更新</span>
                    <span v-else class="pm-muted">已安装</span>
                    <button
                      v-if="row.hasUpdate"
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onUpdateMarketEntry(row)"
                    >
                      更新
                    </button>
                  </template>
                </div>
              </li>
            </ul>
          </section>

          <section class="pm-section pm-section--catalog">
            <div class="pm-section-head">
              <h3 class="pm-h3">已安装插件</h3>
              <div class="pm-section-actions">
                <label class="pm-batch-toggle chk">
                  <input v-model="batchMode" type="checkbox" />
                  批量选择
                </label>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  @click="scaffoldWizardVisible = true"
                >
                  新建插件
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="!selectedWorkspacePlugin"
                  @click="onPackSelectedPlugin"
                >
                  打包当前插件
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginUpdatesCheckLoading"
                  @click="onCheckUpdates"
                >
                  检查更新
                </button>
              </div>
            </div>
            <p v-if="pluginPackStatus" class="pm-hint">{{ pluginPackStatus }}</p>
            <div
              v-if="batchMode && batchSelectedCount > 0"
              class="pm-batch-bar"
              role="toolbar"
              aria-label="批量操作"
            >
              <span class="pm-batch-count">已选 {{ batchSelectedCount }} 个</span>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                批量启用
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                批量停用
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchUpdate">
                批量从 Git 更新
              </button>
            </div>
            <p v-if="!pluginStore.catalog.length" class="pm-muted">
              未扫描到目录插件（请将插件放入 roles 同级的 plugins/ 等目录）。
            </p>

            <div v-else class="pm-wb" aria-label="插件工作区">
              <aside class="pm-wb-sidebar">
                <div class="pm-wb-sidebar-head">
                  <span class="pm-wb-sidebar-title">目录</span>
                  <span class="pm-wb-sidebar-count">{{ pluginStore.catalog.length }}</span>
                </div>
                <ul class="pm-wb-list" role="listbox" aria-label="已安装目录插件">
                  <li v-for="p in pluginStore.catalog" :key="p.id" class="pm-wb-li">
                    <label v-if="batchMode" class="pm-wb-batch chk" @click.stop>
                      <input
                        type="checkbox"
                        :checked="!!batchSelected[p.id]"
                        @change="
                          setBatchSelected(
                            p.id,
                            ($event.target as HTMLInputElement).checked,
                          )
                        "
                      />
                    </label>
                    <button
                      type="button"
                      class="pm-wb-item"
                      :class="{ 'pm-wb-item--active': p.id === selectedWorkspacePluginId }"
                      role="option"
                      :aria-selected="p.id === selectedWorkspacePluginId"
                      @click="selectWorkspacePlugin(p.id)"
                    >
                      <span class="pm-wb-item-id">{{ p.id }}</span>
                      <span class="pm-wb-item-row2">
                        <span class="pm-wb-item-ver">v{{ p.version }}</span>
                        <span class="pm-wb-chip">{{ p.isShell ? "整壳" : "目录" }}</span>
                        <span
                          v-if="pluginStore.pluginUpdateById[p.id]?.hasUpdate"
                          class="pm-wb-pill"
                        >更新</span>
                      </span>
                    </button>
                  </li>
                </ul>
              </aside>

              <main v-if="selectedWorkspacePlugin" class="pm-wb-main">
                <div class="pm-wb-main-head">
                  <div class="pm-wb-main-titles">
                    <h4 class="pm-wb-main-h">{{ selectedWorkspacePlugin.id }}</h4>
                    <span class="pm-wb-main-sub">
                      配置与调试 · 左侧切换插件即可保留本区布局
                    </span>
                  </div>
                  <div class="pm-wb-main-actions">
                    <span
                      v-if="
                        pluginStore.pluginUpdateById[selectedWorkspacePlugin.id]?.hasUpdate
                      "
                      class="pm-badge"
                    >有新版本</span>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onGitPullWorkspacePlugin"
                    >
                      从 Git 拉取更新
                    </button>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      :disabled="
                        pluginStore.extractingPluginId === selectedWorkspacePlugin.id
                      "
                      @click="onUpdateFromZip(selectedWorkspacePlugin.id)"
                    >
                      从本地 zip 更新
                    </button>
                  </div>
                </div>
                <div class="pm-wb-main-body">
                  <InstalledPluginWorkspaceDetail
                    :entry="selectedWorkspacePlugin"
                    :batch-mode="batchMode"
                    :batch-selected="!!batchSelected[selectedWorkspacePlugin.id]"
                    @update:batch-selected="
                      setBatchSelected(selectedWorkspacePlugin.id, $event)
                    "
                  />
                </div>
              </main>
            </div>
          </section>
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'backends'"
            class="pm-tab-panel pm-tab-panel--backends"
            role="tabpanel"
          >
            <PluginBackendSessionPanel />
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'slots'"
            class="pm-tab-panel pm-tab-panel--slots"
            role="tabpanel"
          >
          <section class="pm-section pm-embed-slot">
            <h3 class="pm-h3">插件管理页预览（只读）</h3>
            <p class="pm-hint">
              与下方「settings.plugins」为同一插槽；预览不可操作，请在列表中拖拽排序。
            </p>
            <div class="pm-embed-preview" aria-hidden="true">
              <PluginSlotEmbed
                slot-name="settings.plugins"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.plugins 顺序</h3>
            <p class="pm-hint">本页内嵌区；拖拽排序，可选外观。</p>
            <ol class="pm-order" aria-label="插件管理页槽顺序">
              <li
                v-for="(id, i) in settingsPluginsOrder"
                :key="`spl-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PLUGINS, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PLUGINS, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PLUGINS" />
              </li>
            </ol>
            <p v-if="!settingsPluginsOrder.length" class="pm-muted">当前无 settings.plugins 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">chat_toolbar 顺序</h3>
            <p class="pm-hint">拖拽排序；仅含声明了该插槽的非整壳插件。</p>
            <ol class="pm-order" aria-label="工具栏插件顺序">
              <li
                v-for="(id, i) in toolbarOrder"
                :key="id"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_TOOLBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_TOOLBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_TOOLBAR" />
              </li>
            </ol>
            <p v-if="!toolbarOrder.length" class="pm-muted">当前无 chat_toolbar 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.panel 顺序</h3>
            <p class="pm-hint">设置页「插件扩展」中的嵌入顺序；拖拽排序。</p>
            <ol class="pm-order" aria-label="设置页插件顺序">
              <li
                v-for="(id, i) in settingsPanelOrder"
                :key="`sp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PANEL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PANEL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PANEL" />
              </li>
            </ol>
            <p v-if="!settingsPanelOrder.length" class="pm-muted">当前无 settings.panel 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">role.detail 顺序</h3>
            <p class="pm-hint">左侧角色详情区（立绘下方）嵌入顺序。</p>
            <ol class="pm-order" aria-label="角色详情插件顺序">
              <li
                v-for="(id, i) in roleDetailOrder"
                :key="`rd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_ROLE_DETAIL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_ROLE_DETAIL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_ROLE_DETAIL" />
              </li>
            </ol>
            <p v-if="!roleDetailOrder.length" class="pm-muted">当前无 role.detail 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">sidebar 顺序</h3>
            <p class="pm-hint">左侧栏角色块下方扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="侧边栏插件顺序">
              <li
                v-for="(id, i) in sidebarOrder"
                :key="`sb-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SIDEBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SIDEBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SIDEBAR" />
              </li>
            </ol>
            <p v-if="!sidebarOrder.length" class="pm-muted">当前无 sidebar 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">chat.header 顺序</h3>
            <p class="pm-hint">聊天列顶部（消息列表上方）；拖拽排序。</p>
            <ol class="pm-order" aria-label="聊天头部插件顺序">
              <li
                v-for="(id, i) in chatHeaderOrder"
                :key="`ch-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_HEADER, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_HEADER, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_HEADER" />
              </li>
            </ol>
            <p v-if="!chatHeaderOrder.length" class="pm-muted">当前无 chat.header 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.advanced 顺序</h3>
            <p class="pm-hint">设置对话框「常规」扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="settings.advanced 顺序">
              <li
                v-for="(id, i) in settingsAdvancedOrder"
                :key="`sa-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_ADVANCED, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_ADVANCED, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_ADVANCED" />
              </li>
            </ol>
            <p v-if="!settingsAdvancedOrder.length" class="pm-muted">当前无 settings.advanced 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">overlay.floating 顺序</h3>
            <p class="pm-hint">主界面右下角浮层模板区；拖拽排序。</p>
            <ol class="pm-order" aria-label="overlay.floating 顺序">
              <li
                v-for="(id, i) in overlayFloatingOrder"
                :key="`of-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_OVERLAY_FLOATING, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_OVERLAY_FLOATING, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_OVERLAY_FLOATING" />
              </li>
            </ol>
            <p v-if="!overlayFloatingOrder.length" class="pm-muted">当前无 overlay.floating 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">launcher.palette 顺序</h3>
            <p class="pm-hint">快捷键说明浮层内聚合区；拖拽排序。</p>
            <ol class="pm-order" aria-label="launcher.palette 顺序">
              <li
                v-for="(id, i) in launcherPaletteOrder"
                :key="`lp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_LAUNCHER_PALETTE, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_LAUNCHER_PALETTE, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_LAUNCHER_PALETTE" />
              </li>
            </ol>
            <p v-if="!launcherPaletteOrder.length" class="pm-muted">当前无 launcher.palette 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">debug.dock 顺序</h3>
            <p class="pm-hint">调试面板内扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="debug.dock 顺序">
              <li
                v-for="(id, i) in debugDockOrder"
                :key="`dd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_DEBUG_DOCK, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_DEBUG_DOCK, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_DEBUG_DOCK" />
              </li>
            </ol>
            <p v-if="!debugDockOrder.length" class="pm-muted">当前无 debug.dock 插槽插件。</p>
          </section>
          </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="pluginStore.closePanel()">关闭</button>
            <button type="button" class="pm-btn secondary" @click="onResetToPackDefault">
              重置为角色包推荐
            </button>
            <button type="button" class="pm-btn primary" @click="onSave">保存</button>
          </footer>
        </template>
      </div>
    </div>
    <PluginScaffoldWizard
      :visible="scaffoldWizardVisible"
      @close="scaffoldWizardVisible = false"
      @created="
        scaffoldWizardVisible = false;
        void pluginStore.refresh();
      "
    />
  </Teleport>
</template>

<style scoped>
.pm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10050;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 45%, transparent));
}
.pm-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 55%, transparent));
}
.pm-modal {
  width: min(520px, 100%);
  max-height: min(86vh, 720px);
  overflow: auto;
  padding: 14px 16px 12px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-modal-h {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 8px;
}
.pm-trust-summary {
  margin: 0 0 10px;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-trust-h {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-trust-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-modal-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin: 8px 0;
}
.pm-modal-actions--foot {
  justify-content: flex-end;
  margin-top: 12px;
}
.pm-perm-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm-perm-li {
  margin: 0;
}
.pm-perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.pm-perm-token {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-dialog {
  position: relative;
  width: min(680px, 100%);
  max-height: min(88vh, 760px);
  overflow: auto;
  padding: 16px 18px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog--studio {
  width: min(1080px, 100%);
  max-height: min(92vh, 900px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog-pad {
  padding: 12px 18px;
}
.pm-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px 18px 8px;
}
.pm-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
  padding: 0 18px 10px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-tab {
  flex: 1 1 auto;
  min-width: 0;
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
  transition: var(--control-transition, border-color 0.18s ease, background 0.18s ease);
}
.pm-tab:hover {
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
.pm-tab-panel {
  min-height: 0;
}
.pm-embed-preview {
  pointer-events: none;
  user-select: none;
  opacity: 0.97;
  border-radius: var(--radius-card);
  overflow: hidden;
  border: 1px dashed color-mix(in srgb, var(--border-light) 85%, transparent);
}
.pm-head {
  flex-shrink: 0;
  padding: 16px 40px 12px 18px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-head-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.pm-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}
.pm-studio-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  color: var(--text-accent);
  background: color-mix(in srgb, var(--accent) 12%, var(--bg-elevated));
}
.pm-sub {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-kbd {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 2px;
  font-size: 11px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.pm-close {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.pm-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.pm-section {
  margin-bottom: 18px;
}
.pm-section--catalog {
  padding: 12px 14px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  box-shadow: var(--shadow-sm);
}
.pm-market-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 240px;
  overflow: auto;
}
.pm-market-li {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 13px;
}
.pm-market-main {
  flex: 1 1 200px;
  min-width: 0;
}
.pm-market-desc {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-market-trust {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-source-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  font-size: 11px;
  font-weight: 600;
  vertical-align: middle;
}
.pm-source-badge.official {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
  background: color-mix(in srgb, var(--success-700, #1e7e34) 8%, var(--bg-primary));
}
.pm-source-badge.third {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 8%, var(--bg-primary));
}
.pm-market-deps {
  margin: 6px 0 0;
  font-size: 12px;
}
.pm-market-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}
.pm-section-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}
.pm-batch-toggle {
  font-size: 12px;
  user-select: none;
}
.pm-batch-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px dashed var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
}
.pm-batch-count {
  margin-right: 4px;
  color: var(--text-secondary);
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.pm-h3 {
  margin: 0;
  font-size: 14px;
}
.pm-shell-slots {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}
.pm-shell-slots-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  font-size: 12px;
}
.pm-shell-chip {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-shell-chip--warn {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
}

/* 已安装区：侧栏目录 + 右侧单一配置与调试台 */
.pm-wb {
  display: grid;
  grid-template-columns: minmax(200px, 260px) minmax(0, 1fr);
  gap: 0;
  min-height: min(520px, 58vh);
  max-height: min(62vh, 640px);
  margin-top: 4px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  overflow: hidden;
  background: var(--bg-primary);
}
.pm-wb-sidebar {
  border-right: 1px solid var(--border-light);
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.pm-wb-sidebar-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-wb-sidebar-title {
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.pm-wb-sidebar-count {
  font-variant-numeric: tabular-nums;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-size: 11px;
}
.pm-wb-list {
  list-style: none;
  margin: 0;
  padding: 4px 0;
  overflow: auto;
  flex: 1;
  min-height: 0;
}
.pm-wb-li {
  display: flex;
  align-items: stretch;
  border-bottom: 1px solid
    color-mix(in srgb, var(--border-light) 70%, transparent);
}
.pm-wb-batch {
  display: flex;
  align-items: center;
  padding: 0 8px;
  flex-shrink: 0;
}
.pm-wb-item {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 2px;
  padding: 8px 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font: inherit;
  color: var(--text-primary);
  transition: background 0.15s ease;
}
.pm-wb-item:hover {
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-wb-item--active {
  background: var(--bg-elevated);
  box-shadow: inset 3px 0 0 0 var(--accent);
}
.pm-wb-item-id {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  font-weight: 600;
  word-break: break-all;
}
.pm-wb-item-row2 {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}
.pm-wb-item-ver {
  font-size: 11px;
  color: var(--text-secondary);
}
.pm-wb-chip {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
}
.pm-wb-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-primary));
  color: var(--text-accent);
}
.pm-wb-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.pm-wb-main-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-wb-main-titles {
  min-width: 0;
}
.pm-wb-main-h {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  font-family: ui-monospace, Menlo, Consolas, monospace;
}
.pm-wb-main-sub {
  display: block;
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-secondary);
}
.pm-wb-main-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm-wb-main-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.pm-wb-debug {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.pm-wb-debug-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
@media (max-width: 720px) {
  .pm-wb {
    grid-template-columns: 1fr;
    max-height: none;
  }
  .pm-wb-sidebar {
    border-right: none;
    border-bottom: 1px solid var(--border-light);
    max-height: 200px;
  }
}

.pm-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated));
  color: var(--text-primary);
}
.pm-btn--sm {
  padding: 5px 10px;
  font-size: 12px;
}
.pm-hint {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-order {
  margin: 0;
  padding-left: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-order-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  font-size: 13px;
  border: 1px dashed var(--border-light);
  border-radius: var(--radius-btn);
  cursor: grab;
  background: var(--bg-elevated);
}
.pm-order-item--row {
  flex-wrap: wrap;
}
.pm-order-id {
  flex: 1;
  min-width: 0;
  word-break: break-all;
}
.pm-embed-slot code {
  font-size: 11px;
}
.pm-grip {
  color: var(--text-secondary);
  font-size: 12px;
  user-select: none;
}
.pm-scope-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 6px;
}
.pm-scope-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}
.pm-author-summary {
  margin: 0 0 8px;
  font-size: 13px;
  line-height: 1.45;
}
.pm-rec-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.5;
}
.pm-muted {
  font-size: 13px;
  color: var(--text-secondary);
}
.pm-err {
  color: var(--error);
  font-size: 13px;
}
.pm-foot {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  margin: 0;
  padding: 12px 18px;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-btn {
  padding: 8px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  font-size: 13px;
  cursor: pointer;
}
.pm-btn.secondary {
  background: transparent;
}
.pm-btn.primary {
  background: var(--accent);
  color: var(--bg-elevated);
  border-color: color-mix(in srgb, var(--accent) 85%, var(--text-primary) 15%);
}
.pm-btn.primary:hover {
  filter: brightness(1.05);
}
</style>
