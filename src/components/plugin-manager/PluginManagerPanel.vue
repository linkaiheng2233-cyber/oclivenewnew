<script setup lang="ts">
/** Advanced plugin manager (architecture graph, backends/models, slot layout); opened from Settings or the simple manager entry. */
import { computed, defineAsyncComponent, ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import SlotLayoutDiagram from '../SlotLayoutDiagram.vue'
import PluginManagerCatalogSection from './PluginManagerCatalogSection.vue'
import PluginManagerMainTabs from './PluginManagerMainTabs.vue'
import { useAppToast } from '../../composables/useAppToast'
import { useModalFocusRestore } from '../../composables/useModalFocusRestore'
import { usePluginStore } from '../../stores/pluginStore'
import { usePluginTraceStore } from '../../stores/pluginTraceStore'
import { useRoleStore } from '../../stores/roleStore'
import { applyAuthorSuggestedPluginBackends } from '../../api'

const ArchitectureGraph = defineAsyncComponent(
  () => import('../ArchitectureGraphFlow.vue'),
)
const ExpertConfigPanel = defineAsyncComponent(
  () => import('../expert/ExpertConfigPanel.vue'),
)

const pluginStore = usePluginStore()
const traceStore = usePluginTraceStore()
const roleStore = useRoleStore()
const { showToast } = useAppToast()
const { t } = useI18n()

const panelDialogRef = ref<HTMLElement | null>(null)
const mainTabsRef = ref<InstanceType<typeof PluginManagerMainTabs> | null>(null)
const panelFirstTabRef = computed(
  () => mainTabsRef.value?.firstTabRef ?? null,
)
useModalFocusRestore(toRef(traceStore, 'panelVisible'), panelDialogRef, {
  primary: panelFirstTabRef,
})

async function onSave() {
  try {
    await pluginStore.persist()
    showToast('success', t('pluginWorkbench.toast.saved'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onResetToPackDefault() {
  try {
    if (pluginStore.persistScope === 'global') {
      pluginStore.setPersistScope('role')
    }
    await pluginStore.resetToRolePackDefault()
    showToast('success', t('pluginWorkbench.toast.resetLayout'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onApplyAuthorSuggestedBackends() {
  try {
    const info = await applyAuthorSuggestedPluginBackends(roleStore.currentRoleId)
    roleStore.applyRoleInfo(info)
    showToast('success', t('pluginWorkbench.toast.authorBackends'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

function openMarket() {
  void traceStore.openMarketPanel()
}

function onFocusPluginFromGraph(id: string) {
  traceStore.requestFocusInstalledPlugin(id)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="traceStore.panelVisible"
      class="pm-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('pluginWorkbench.aria.dialogStudio')"
      @click.self="traceStore.closePanel()"
      @keydown.escape.stop="traceStore.closePanel()"
    >
      <div ref="panelDialogRef" class="pm-dialog pm-dialog--studio" tabindex="-1" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">
              {{ t("pluginWorkbench.header.title") }}
            </h2>
            <span class="pm-studio-badge" :title="t('pluginWorkbench.header.badgeTitle')">{{
              t("pluginWorkbench.header.badge")
            }}</span>
          </div>
          <p class="pm-sub">
            <kbd class="pm-kbd">Ctrl</kbd>+<kbd class="pm-kbd">Shift</kbd>+<kbd class="pm-kbd">F</kbd>
            {{ t("pluginWorkbench.header.sub") }}
          </p>
          <button type="button" class="pm-close" :aria-label="t('common.close')" @click="traceStore.closePanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted pm-dialog-pad">
          {{ t("pluginWorkbench.loading") }}
        </div>
        <p v-else-if="pluginStore.error" class="pm-err pm-dialog-pad">
          {{ pluginStore.error }}
        </p>

        <template v-else>
          <div class="pm-top-actions">
            <button type="button" class="pm-btn secondary pm-btn--sm" @click="openMarket">
              {{ t("pluginWorkbench.openMarket") }}
            </button>
          </div>

          <PluginManagerMainTabs ref="mainTabsRef" />

          <div class="pm-scroll">
            <section class="pm-section">
              <h3 class="pm-h3">
                {{ t("pluginWorkbench.persist.title") }}
              </h3>
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
                  >
                  {{ t("pluginWorkbench.persist.scopeRole") }}
                </label>
                <label class="pm-scope-label">
                  <input
                    type="radio"
                    name="pm-persist-scope"
                    :checked="pluginStore.persistScope === 'global'"
                    @change="pluginStore.setPersistScope('global')"
                  >
                  {{ t("pluginWorkbench.persist.scopeGlobal") }}
                </label>
              </div>
            </section>

            <section
              v-if="roleStore.roleInfo.authorPack?.suggested_plugin_backends"
              class="pm-section"
            >
              <h3 class="pm-h3">
                {{ t("pluginWorkbench.authorBackends.title") }}
              </h3>
              <p class="pm-hint">
                {{ t("pluginWorkbench.authorBackends.hint") }}
              </p>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onApplyAuthorSuggestedBackends">
                {{ t("pluginWorkbench.authorBackends.apply") }}
              </button>
            </section>

            <section v-if="roleStore.roleInfo.authorPack" class="pm-section">
              <h3 class="pm-h3">
                {{ t("pluginWorkbench.authorRec.title") }}
              </h3>
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
              <p v-else class="pm-muted">
                {{ t("pluginWorkbench.authorRec.noList") }}
              </p>
            </section>

            <PluginManagerCatalogSection />

            <div v-show="traceStore.panelMainTab === 'graph'" class="pm-tab-panel" role="tabpanel">
              <ExpertConfigPanel />
              <ArchitectureGraph @focus-plugin="onFocusPluginFromGraph" />
            </div>
            <div v-show="traceStore.panelMainTab === 'layout'" class="pm-tab-panel" role="tabpanel">
              <SlotLayoutDiagram />
            </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="traceStore.closePanel()">
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
</style>
