<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
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
  type PluginMarketEntryDto,
} from "../utils/tauri-api";

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const { showToast } = useAppToast();
const { t } = useI18n();

watch(
  () => pluginStore.panelVisible,
  (vis) => {
    if (vis) {
      void pluginStore.loadCachedPluginMarket();
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
    showToast(
      "success",
      t("pluginWorkbench.toast.batchEnable", { count: ids.length }),
    );
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
  showToast(
    "success",
    t("pluginWorkbench.toast.batchDisable", { count: ids.length }),
  );
  clearBatchSelection();
}

async function onBatchUpdate() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    await pluginStore.batchUpdatePluginsFromGitIndex(ids);
    showToast("success", t("pluginWorkbench.toast.batchGitPull"));
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onSyncMarketIndex() {
  try {
    await pluginStore.syncPluginMarket();
    if (pluginStore.pluginMarketSnapshot?.warning) {
      showToast("info", t("pluginWorkbench.market.toastOfflineCache"));
    } else {
      showToast("success", t("pluginWorkbench.toast.indexSynced"));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallMarketEntry(row: PluginMarketEntryDto) {
  if ((row.missingDependencies ?? []).length > 0) {
    showToast(
      "error",
      t("pluginWorkbench.toast.installDeps", {
        list: row.missingDependencies.join(", "),
      }),
    );
    return;
  }
  try {
    await pluginStore.installFromPluginMarket(row.id, row.git);
    showToast("success", t("pluginWorkbench.toast.installed", { id: row.id }));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateMarketEntry(row: PluginMarketEntryDto) {
  try {
    await pluginStore.updateInstalledPluginFromGit(row.id);
    showToast("success", t("pluginWorkbench.toast.updatedGit", { id: row.id }));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onGitPullWorkspacePlugin() {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) return;
  try {
    await pluginStore.updateInstalledPluginFromGit(pid);
    showToast("success", t("pluginWorkbench.toast.gitPulled"));
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
    showToast("success", t("pluginWorkbench.toast.saved"));
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
    showToast("success", t("pluginWorkbench.toast.resetLayout"));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onApplyAuthorSuggestedBackends() {
  try {
    const info = await applyAuthorSuggestedPluginBackends(roleStore.currentRoleId);
    roleStore.applyRoleInfo(info);
    showToast("success", t("pluginWorkbench.toast.authorBackends"));
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
      showToast("success", t("pluginWorkbench.toast.checkDone"));
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
    await pluginStore.installPluginFromLocalZip(pluginId, path);
    showToast("success", t("pluginWorkbench.toast.zipUpdated"));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onPackSelectedPlugin(): Promise<void> {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) {
    pluginPackStatus.value = t("pluginWorkbench.pack.pickFirst");
    return;
  }
  try {
    const r = await packPlugin(pid);
    pluginPackStatus.value = t("pluginWorkbench.pack.done", { path: r.archive_path });
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
      :aria-label="t('pluginWorkbench.aria.dialogStudio')"
      @click.self="pluginStore.closePanel()"
    >
      <div class="pm-dialog pm-dialog--studio" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">{{ t("pluginWorkbench.header.title") }}</h2>
            <span
              class="pm-studio-badge"
              :title="t('pluginWorkbench.header.badgeTitle')"
            >{{ t("pluginWorkbench.header.badge") }}</span>
          </div>
          <p class="pm-sub">
            <kbd class="pm-kbd">Ctrl</kbd>+<kbd class="pm-kbd">Shift</kbd>+<kbd class="pm-kbd">F</kbd>
            {{ t("pluginWorkbench.header.sub") }}
          </p>
          <button type="button" class="pm-close" :aria-label="t('common.close')" @click="pluginStore.closePanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted pm-dialog-pad">{{ t("pluginWorkbench.loading") }}</div>
        <p v-else-if="pluginStore.error" class="pm-err pm-dialog-pad">{{ pluginStore.error }}</p>

        <template v-else>
          <div class="pm-tabs" role="tablist" :aria-label="t('pluginWorkbench.aria.tablist')">
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'plugins' }"
              :aria-selected="pluginStore.panelMainTab === 'plugins'"
              @click="pluginStore.panelMainTab = 'plugins'"
            >
              {{ t("pluginWorkbench.tabs.uiPlugins") }}
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'backends' }"
              :aria-selected="pluginStore.panelMainTab === 'backends'"
              @click="pluginStore.panelMainTab = 'backends'"
            >
              {{ t("pluginWorkbench.tabs.backends") }}
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'slots' }"
              :aria-selected="pluginStore.panelMainTab === 'slots'"
              @click="pluginStore.panelMainTab = 'slots'"
            >
              {{ t("pluginWorkbench.tabs.slotOrder") }}
            </button>
          </div>

          <div class="pm-scroll">
          <div
            v-show="pluginStore.panelMainTab === 'plugins'"
            class="pm-tab-panel"
            role="tabpanel"
          >
          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.persist.title") }}</h3>
            <p class="pm-hint">
              {{ t("pluginWorkbench.persist.hint") }}
            </p>
            <div class="pm-scope-row" role="group" :aria-label="t('pluginWorkbench.persist.scopeAria')">
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'role'"
                  @change="pluginStore.setPersistScope('role')"
                />
                {{ t("pluginWorkbench.persist.scopeRole") }}
              </label>
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'global'"
                  @change="pluginStore.setPersistScope('global')"
                />
                {{ t("pluginWorkbench.persist.scopeGlobal") }}
              </label>
            </div>
          </section>

          <section
            v-if="roleStore.roleInfo.authorPack?.suggested_plugin_backends"
            class="pm-section"
          >
            <h3 class="pm-h3">{{ t("pluginWorkbench.authorBackends.title") }}</h3>
            <p class="pm-hint">
              {{ t("pluginWorkbench.authorBackends.hint") }}
            </p>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="onApplyAuthorSuggestedBackends"
            >
              {{ t("pluginWorkbench.authorBackends.apply") }}
            </button>
          </section>

          <section v-if="roleStore.roleInfo.authorPack" class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.authorRec.title") }}</h3>
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
                <span v-if="rp.optional" class="pm-muted">{{ t("pluginWorkbench.authorRec.optional") }}</span>
              </li>
            </ul>
            <p v-else class="pm-muted">{{ t("pluginWorkbench.authorRec.noList") }}</p>
          </section>

          <section class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">{{ t("pluginWorkbench.market.title") }}</h3>
              <div class="pm-section-actions">
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginMarketSyncing"
                  @click="onSyncMarketIndex"
                >
                  {{ pluginStore.pluginMarketSyncing ? t("pluginWorkbench.market.syncing") : t("pluginWorkbench.market.sync") }}
                </button>
              </div>
            </div>
            <p v-if="pluginStore.pluginMarketError" class="pm-err">{{ pluginStore.pluginMarketError }}</p>
            <div
              v-else-if="pluginStore.pluginMarketSnapshot?.warning"
              class="pm-market-callout"
              role="status"
            >
              <p class="pm-hint pm-hint--strong">{{ t("pluginWorkbench.market.syncFailedTitle") }}</p>
              <p class="pm-muted pm-hint-detail">
                {{ t("pluginWorkbench.market.syncFailedDetail", { detail: pluginStore.pluginMarketSnapshot.warning }) }}
              </p>
            </div>
            <p
              v-else-if="pluginStore.pluginMarketSnapshot?.offlineMode"
              class="pm-hint"
            >
              {{ t("pluginWorkbench.market.offline") }}
            </p>
            <p
              v-if="
                !pluginStore.pluginMarketSnapshot?.plugins?.length &&
                !pluginStore.pluginMarketError
              "
              class="pm-muted"
            >
              {{ t("pluginWorkbench.market.emptyIndex") }}
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
                  <span class="pm-muted"> · {{ row.name }} · v{{ row.version }}</span>
                  <p v-if="row.description" class="pm-market-desc">{{ row.description }}</p>
                  <p
                    v-if="(row.missingDependencies ?? []).length"
                    class="pm-err pm-market-deps"
                  >
                    {{ t("pluginWorkbench.market.deps", { list: row.missingDependencies.join(", ") }) }}
                  </p>
                </div>
                <div class="pm-market-actions">
                  <button
                    v-if="!row.installed"
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="onInstallMarketEntry(row)"
                  >
                    {{ t("pluginWorkbench.market.install") }}
                  </button>
                  <template v-else>
                    <span v-if="row.hasUpdate" class="pm-badge">{{ t("pluginWorkbench.market.badgeUpdate") }}</span>
                    <span v-else class="pm-muted">{{ t("pluginWorkbench.market.installed") }}</span>
                    <button
                      v-if="row.hasUpdate"
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onUpdateMarketEntry(row)"
                    >
                      {{ t("pluginWorkbench.market.update") }}
                    </button>
                  </template>
                </div>
              </li>
            </ul>
          </section>

          <section class="pm-section pm-section--catalog">
            <div class="pm-section-head">
              <h3 class="pm-h3">{{ t("pluginWorkbench.catalog.title") }}</h3>
              <div class="pm-section-actions">
                <label class="pm-batch-toggle chk">
                  <input v-model="batchMode" type="checkbox" />
                  {{ t("pluginWorkbench.catalog.batchToggle") }}
                </label>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  @click="scaffoldWizardVisible = true"
                >
                  {{ t("pluginWorkbench.catalog.newPlugin") }}
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="!selectedWorkspacePlugin"
                  @click="onPackSelectedPlugin"
                >
                  {{ t("pluginWorkbench.catalog.packCurrent") }}
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginUpdatesCheckLoading"
                  @click="onCheckUpdates"
                >
                  {{ t("pluginWorkbench.catalog.checkUpdates") }}
                </button>
              </div>
            </div>
            <p v-if="pluginPackStatus" class="pm-hint">{{ pluginPackStatus }}</p>
            <div
              v-if="batchMode && batchSelectedCount > 0"
              class="pm-batch-bar"
              role="toolbar"
              :aria-label="t('pluginWorkbench.aria.batchToolbar')"
            >
              <span class="pm-batch-count">{{ t("pluginWorkbench.catalog.selectedCount", { count: batchSelectedCount }) }}</span>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                {{ t("pluginWorkbench.catalog.batchEnable") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                {{ t("pluginWorkbench.catalog.batchDisable") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchUpdate">
                {{ t("pluginWorkbench.catalog.batchGit") }}
              </button>
            </div>
            <p v-if="!pluginStore.catalog.length" class="pm-muted">
              {{ t("pluginWorkbench.catalog.emptyScan") }}
            </p>

            <div v-else class="pm-wb" :aria-label="t('pluginWorkbench.catalog.workspaceAria')">
              <aside class="pm-wb-sidebar">
                <div class="pm-wb-sidebar-head">
                  <span class="pm-wb-sidebar-title">{{ t("pluginWorkbench.catalog.sidebarTitle") }}</span>
                  <span class="pm-wb-sidebar-count">{{ pluginStore.catalog.length }}</span>
                </div>
                <ul class="pm-wb-list" role="listbox" :aria-label="t('pluginWorkbench.catalog.listAria')">
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
                        <span class="pm-wb-chip">{{ p.isShell ? t("pluginWorkbench.catalog.chipShell") : t("pluginWorkbench.catalog.chipDir") }}</span>
                        <span
                          v-if="pluginStore.pluginUpdateById[p.id]?.hasUpdate"
                          class="pm-wb-pill"
                        >{{ t("pluginWorkbench.catalog.pillUpdate") }}</span>
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
                      {{ t("pluginWorkbench.catalog.detailSub") }}
                    </span>
                  </div>
                  <div class="pm-wb-main-actions">
                    <span
                      v-if="
                        pluginStore.pluginUpdateById[selectedWorkspacePlugin.id]?.hasUpdate
                      "
                      class="pm-badge"
                    >{{ t("pluginWorkbench.catalog.hasNew") }}</span>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onGitPullWorkspacePlugin"
                    >
                      {{ t("pluginWorkbench.catalog.pullGit") }}
                    </button>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      :disabled="
                        pluginStore.extractingPluginId === selectedWorkspacePlugin.id
                      "
                      @click="onUpdateFromZip(selectedWorkspacePlugin.id)"
                    >
                      {{ t("pluginWorkbench.catalog.pullZip") }}
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
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.previewTitle") }}</h3>
            <p class="pm-hint">
              {{ t("pluginWorkbench.slots.previewHint") }}
            </p>
            <div class="pm-embed-preview" aria-hidden="true">
              <PluginSlotEmbed
                slot-name="settings.plugins"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.settingsPluginsTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.settingsPluginsHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaSettingsPlugins')">
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
            <p v-if="!settingsPluginsOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.settingsPluginsEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.chatToolbarTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.chatToolbarHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaChatToolbar')">
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
            <p v-if="!toolbarOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.chatToolbarEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.settingsPanelTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.settingsPanelHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaSettingsPanel')">
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
            <p v-if="!settingsPanelOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.settingsPanelEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.roleDetailTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.roleDetailHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaRoleDetail')">
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
            <p v-if="!roleDetailOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.roleDetailEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.sidebarTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.sidebarHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaSidebar')">
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
            <p v-if="!sidebarOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.sidebarEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.chatHeaderTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.chatHeaderHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaChatHeader')">
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
            <p v-if="!chatHeaderOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.chatHeaderEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.settingsAdvancedTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.settingsAdvancedHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaSettingsAdvanced')">
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
            <p v-if="!settingsAdvancedOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.settingsAdvancedEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.overlayFloatingTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.overlayFloatingHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaOverlayFloating')">
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
            <p v-if="!overlayFloatingOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.overlayFloatingEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.launcherPaletteTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.launcherPaletteHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaLauncherPalette')">
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
            <p v-if="!launcherPaletteOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.launcherPaletteEmpty") }}</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginWorkbench.slots.debugDockTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginWorkbench.slots.debugDockHint") }}</p>
            <ol class="pm-order" :aria-label="t('pluginWorkbench.slots.ariaDebugDock')">
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
            <p v-if="!debugDockOrder.length" class="pm-muted">{{ t("pluginWorkbench.slots.debugDockEmpty") }}</p>
          </section>
          </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="pluginStore.closePanel()">{{ t("pluginWorkbench.footer.close") }}</button>
            <button type="button" class="pm-btn secondary" @click="onResetToPackDefault">
              {{ t("pluginWorkbench.footer.resetPack") }}
            </button>
            <button type="button" class="pm-btn primary" @click="onSave">{{ t("pluginWorkbench.footer.save") }}</button>
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
.pm-hint--strong {
  font-weight: 600;
  color: var(--text-primary);
}
.pm-hint-detail {
  margin: 4px 0 0;
  font-size: 11px;
  line-height: 1.45;
  word-break: break-word;
}
.pm-market-callout {
  margin: 0 0 10px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--accent, #3b82f6) 28%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 8%, var(--bg-elevated));
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
