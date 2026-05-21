<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed, defineAsyncComponent, ref, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import InstalledPluginWorkspaceDetail from "../components/InstalledPluginWorkspaceDetail.vue";

const ArchitectureGraph = defineAsyncComponent(
  () => import("../components/ArchitectureGraphFlow.vue"),
);
import PluginScaffoldWizard from "../components/PluginScaffoldWizard.vue";
import SlotLayoutDiagram from "../components/SlotLayoutDiagram.vue";
import { useAppToast } from "../composables/useAppToast";
import { useModalFocusRestore } from "../composables/useModalFocusRestore";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { applyAuthorSuggestedPluginBackends, packPlugin } from "../utils/tauri-api";

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const { showToast } = useAppToast();
const { t } = useI18n();

const batchMode = ref(false);
const batchSelected = ref<Record<string, boolean>>({});
const scaffoldWizardVisible = ref(false);
const pluginPackStatus = ref("");
const selectedWorkspacePluginId = ref("");

const panelDialogRef = ref<HTMLElement | null>(null);
const panelFirstTabRef = ref<HTMLButtonElement | null>(null);
useModalFocusRestore(toRef(pluginStore, "panelVisible"), panelDialogRef, {
  primary: panelFirstTabRef,
});

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
  if (!v) clearBatchSelection();
});

watch(
  () => pluginStore.catalog.map((c) => c.id).join("\n"),
  () => {
    const next: Record<string, boolean> = {};
    for (const p of pluginStore.catalog) {
      if (batchSelected.value[p.id]) next[p.id] = true;
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

watch(
  () => pluginStore.focusPluginId,
  (id) => {
    if (id && pluginStore.catalog.some((c) => c.id === id)) {
      selectedWorkspacePluginId.value = id;
      pluginStore.clearFocusInstalledPlugin();
    }
  },
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
  if (ids.length === 0) return;
  try {
    pluginStore.batchEnablePluginIds(ids);
    showToast("success", t("pluginWorkbench.toast.batchEnable", { count: ids.length }));
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onBatchDisable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) return;
  pluginStore.batchDisablePluginIds(ids);
  showToast("success", t("pluginWorkbench.toast.batchDisable", { count: ids.length }));
  clearBatchSelection();
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
    filters: [{ name: t("pluginWorkbench.localZipFilterName"), extensions: ["zip"] }],
  });
  if (path === null || Array.isArray(path)) return;
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

function openMarket() {
  void pluginStore.openMarketPanel();
}

function onFocusPluginFromGraph(id: string) {
  pluginStore.requestFocusInstalledPlugin(id);
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
      @keydown.escape.stop="pluginStore.closePanel()"
    >
      <div ref="panelDialogRef" class="pm-dialog pm-dialog--studio" tabindex="-1" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">{{ t("pluginWorkbench.header.title") }}</h2>
            <span class="pm-studio-badge" :title="t('pluginWorkbench.header.badgeTitle')">{{
              t("pluginWorkbench.header.badge")
            }}</span>
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
          <div class="pm-top-actions">
            <button type="button" class="pm-btn secondary pm-btn--sm" @click="openMarket">
              {{ t("pluginWorkbench.openMarket") }}
            </button>
          </div>

          <div class="pm-tabs" role="tablist" :aria-label="t('pluginWorkbench.aria.tablist')">
            <button
              ref="panelFirstTabRef"
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'graph' }"
              :aria-selected="pluginStore.panelMainTab === 'graph'"
              @click="pluginStore.panelMainTab = 'graph'"
            >
              {{ t("pluginWorkbench.tabs.graph") }}
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'layout' }"
              :aria-selected="pluginStore.panelMainTab === 'layout'"
              @click="pluginStore.panelMainTab = 'layout'"
            >
              {{ t("pluginWorkbench.tabs.layout") }}
            </button>
          </div>

          <div class="pm-scroll">
            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginWorkbench.persist.title") }}</h3>
              <p class="pm-hint">{{ t("pluginWorkbench.persist.hint") }}</p>
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
              <p class="pm-hint">{{ t("pluginWorkbench.authorBackends.hint") }}</p>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onApplyAuthorSuggestedBackends">
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

            <section class="pm-section pm-section--catalog">
              <div class="pm-section-head">
                <h3 class="pm-h3">{{ t("pluginWorkbench.catalog.title") }}</h3>
                <div class="pm-section-actions">
                  <label class="pm-batch-toggle chk">
                    <input v-model="batchMode" type="checkbox" />
                    {{ t("pluginWorkbench.catalog.batchToggle") }}
                  </label>
                  <button type="button" class="pm-btn secondary pm-btn--sm" @click="scaffoldWizardVisible = true">
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
                <span class="pm-batch-count">{{
                  t("pluginWorkbench.catalog.selectedCount", { count: batchSelectedCount })
                }}</span>
                <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                  {{ t("pluginWorkbench.catalog.batchEnable") }}
                </button>
                <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                  {{ t("pluginWorkbench.catalog.batchDisable") }}
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
                            setBatchSelected(p.id, ($event.target as HTMLInputElement).checked)
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
                          <span class="pm-wb-chip">{{
                            p.isShell ? t("pluginWorkbench.catalog.chipShell") : t("pluginWorkbench.catalog.chipDir")
                          }}</span>
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
                      <span class="pm-wb-main-sub">{{ t("pluginWorkbench.catalog.detailSub") }}</span>
                    </div>
                    <div class="pm-wb-main-actions">
                      <span
                        v-if="pluginStore.pluginUpdateById[selectedWorkspacePlugin.id]?.hasUpdate"
                        class="pm-badge"
                      >{{ t("pluginWorkbench.catalog.hasNew") }}</span>
                      <button type="button" class="pm-btn secondary pm-btn--sm" @click="onGitPullWorkspacePlugin">
                        {{ t("pluginWorkbench.catalog.pullGit") }}
                      </button>
                      <button
                        type="button"
                        class="pm-btn secondary pm-btn--sm"
                        :disabled="pluginStore.extractingPluginId === selectedWorkspacePlugin.id"
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
                      @update:batch-selected="setBatchSelected(selectedWorkspacePlugin.id, $event)"
                    />
                  </div>
                </main>
              </div>
            </section>

            <div v-show="pluginStore.panelMainTab === 'graph'" class="pm-tab-panel" role="tabpanel">
              <ArchitectureGraph @focus-plugin="onFocusPluginFromGraph" />
            </div>
            <div v-show="pluginStore.panelMainTab === 'layout'" class="pm-tab-panel" role="tabpanel">
              <SlotLayoutDiagram />
            </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="pluginStore.closePanel()">
              {{ t("pluginWorkbench.footer.close") }}
            </button>
            <button type="button" class="pm-btn secondary" @click="onResetToPackDefault">
              {{ t("pluginWorkbench.footer.resetPack") }}
            </button>
            <button type="button" class="pm-btn primary" @click="onSave">
              {{ t("pluginWorkbench.footer.save") }}
            </button>
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
.pm-top-actions {
  flex-shrink: 0;
  padding: 8px 18px 0;
  display: flex;
  justify-content: flex-end;
}
.pm-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 18px 10px;
  border-bottom: 1px solid var(--border-light);
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
}
.pm-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
.pm-tab-panel {
  margin-top: 12px;
}
.pm-head {
  flex-shrink: 0;
  padding: 16px 40px 12px 18px;
  border-bottom: 1px solid var(--border-light);
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
  cursor: pointer;
  color: var(--text-secondary);
}
.pm-section {
  margin-bottom: 18px;
}
.pm-section--catalog {
  padding: 12px 14px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
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
.pm-wb {
  display: grid;
  grid-template-columns: minmax(200px, 260px) minmax(0, 1fr);
  gap: 0;
  min-height: min(360px, 48vh);
  max-height: min(52vh, 520px);
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
.pm-wb-list {
  list-style: none;
  margin: 0;
  padding: 4px 0;
  overflow: auto;
  flex: 1;
}
.pm-wb-li {
  display: flex;
  align-items: stretch;
}
.pm-wb-batch {
  display: flex;
  align-items: center;
  padding: 0 8px;
}
.pm-wb-item {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font: inherit;
}
.pm-wb-item--active {
  background: var(--bg-elevated);
  box-shadow: inset 3px 0 0 0 var(--accent);
}
.pm-wb-item-id {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  font-weight: 600;
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
}
.pm-wb-pill {
  font-size: 10px;
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
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
}
.pm-wb-main-h {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  font-family: ui-monospace, Menlo, Consolas, monospace;
}
.pm-wb-main-sub {
  font-size: 11px;
  color: var(--text-secondary);
}
.pm-wb-main-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px 12px;
}
.pm-h3 {
  margin: 0;
  font-size: 14px;
}
.pm-hint {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
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
  gap: 10px;
  padding: 12px 18px;
  border-top: 1px solid var(--border-light);
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
  border-color: transparent;
}
.pm-btn--sm {
  padding: 5px 10px;
  font-size: 12px;
}
.pm-batch-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
  padding: 8px 10px;
  border: 1px dashed var(--border-light);
  border-radius: 8px;
}
.pm-scope-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
}
.pm-scope-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.pm-author-summary {
  margin: 0 0 8px;
  font-size: 13px;
}
.pm-rec-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
}
.pm-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated));
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
@media (max-width: 720px) {
  .pm-wb {
    grid-template-columns: 1fr;
  }
  .pm-wb-sidebar {
    max-height: 180px;
    border-right: none;
    border-bottom: 1px solid var(--border-light);
  }
}
</style>
