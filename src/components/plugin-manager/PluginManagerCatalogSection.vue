<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import InstalledPluginWorkspaceDetail from '../InstalledPluginWorkspaceDetail.vue'
import PluginScaffoldWizard from '../PluginScaffoldWizard.vue'
import { useAppToast } from '../../composables/useAppToast'
import { usePluginManagerWorkspace } from '../../composables/usePluginManagerWorkspace'
import { usePluginStore } from '../../stores/pluginStore'
import { packPlugin } from '../../api'

const pluginStore = usePluginStore()
const { showToast } = useAppToast()
const { t } = useI18n()

const {
  batchMode,
  batchSelected,
  selectedWorkspacePluginId,
  selectedWorkspacePlugin,
  batchSelectedCount,
  batchSelectedIds,
  selectWorkspacePlugin,
  focusAdjacentCatalog,
  clearBatchSelection,
  setBatchSelected,
} = usePluginManagerWorkspace()

const scaffoldWizardVisible = ref(false)
const pluginPackStatus = ref('')

async function onBatchEnable() {
  const ids = batchSelectedIds.value
  if (ids.length === 0)
    return
  try {
    pluginStore.batchEnablePluginIds(ids)
    showToast('success', t('pluginWorkbench.toast.batchEnable', { count: ids.length }))
    clearBatchSelection()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onBatchDisable() {
  const ids = batchSelectedIds.value
  if (ids.length === 0)
    return
  pluginStore.batchDisablePluginIds(ids)
  showToast('success', t('pluginWorkbench.toast.batchDisable', { count: ids.length }))
  clearBatchSelection()
}

async function onGitPullWorkspacePlugin() {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? ''
  if (!pid)
    return
  try {
    await pluginStore.updateInstalledPluginFromGit(pid)
    showToast('success', t('pluginWorkbench.toast.gitPulled'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onCheckUpdates() {
  try {
    await pluginStore.checkPluginUpdatesFromRegistry()
    if (pluginStore.error) {
      showToast('error', pluginStore.error)
    }
    else {
      showToast('success', t('pluginWorkbench.toast.checkDone'))
    }
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onUpdateFromZip(pluginId: string) {
  const path = await open({
    multiple: false,
    filters: [{ name: t('pluginWorkbench.localZipFilterName'), extensions: ['zip'] }],
  })
  if (path === null || Array.isArray(path))
    return
  try {
    await pluginStore.installPluginFromLocalZip(pluginId, path)
    showToast('success', t('pluginWorkbench.toast.zipUpdated'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onPackSelectedPlugin(): Promise<void> {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? ''
  if (!pid) {
    pluginPackStatus.value = t('pluginWorkbench.pack.pickFirst')
    return
  }
  try {
    const r = await packPlugin(pid)
    pluginPackStatus.value = t('pluginWorkbench.pack.done', { path: r.archive_path })
  }
  catch (e) {
    pluginPackStatus.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <section class="pm-section pm-section--catalog">
    <div class="pm-section-head">
      <h3 class="pm-h3">
        {{ t("pluginWorkbench.catalog.title") }}
      </h3>
      <div class="pm-section-actions">
        <label class="pm-batch-toggle chk">
          <input v-model="batchMode" type="checkbox">
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
    <p v-if="pluginPackStatus" class="pm-hint">
      {{ pluginPackStatus }}
    </p>
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
              >
            </label>
            <button
              type="button"
              class="pm-wb-item"
              :class="{ 'pm-wb-item--active': p.id === selectedWorkspacePluginId }"
              role="option"
              :aria-selected="p.id === selectedWorkspacePluginId"
              :tabindex="p.id === selectedWorkspacePluginId ? 0 : -1"
              @click="selectWorkspacePlugin(p.id)"
              @keydown.down.prevent="focusAdjacentCatalog(1)"
              @keydown.up.prevent="focusAdjacentCatalog(-1)"
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
            <h4 class="pm-wb-main-h">
              {{ selectedWorkspacePlugin.id }}
            </h4>
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
  <PluginScaffoldWizard
    :visible="scaffoldWizardVisible"
    @close="scaffoldWizardVisible = false"
    @created="
      scaffoldWizardVisible = false;
      void pluginStore.refresh();
    "
  />
</template>

<style scoped>
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
