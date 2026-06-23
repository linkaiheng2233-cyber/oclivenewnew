<script setup lang="ts">
import type { PluginMarketEntryDto } from '@oclive/shared/api'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { ensurePluginWorkbenchI18n } from '@oclive/shared/i18n/loadPluginWorkbench'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { usePluginMarketStore } from '@oclive/shared/stores/pluginMarketStore'

const props = withDefaults(
  defineProps<{
    embedded?: boolean
    visible?: boolean
  }>(),
  { embedded: false, visible: true },
)

const emit = defineEmits<{
  back: []
}>()

const marketStore = usePluginMarketStore()
const { showToast } = useAppToast()
const { t } = useI18n()

const searchQuery = ref('')
const categoryFilter = ref('')
const i18nReady = ref(false)

const dialogRef = ref<HTMLElement | null>(null)
const shareUrlRef = ref<HTMLInputElement | null>(null)

const panelOpen = computed(() =>
  props.embedded ? props.visible : marketStore.marketPanelVisible,
)

useModalFocusRestore(panelOpen, dialogRef, {
  primary: shareUrlRef,
})

async function ensureMarketI18n(): Promise<void> {
  await ensurePluginWorkbenchI18n()
  i18nReady.value = true
}

watch(
  panelOpen,
  async (vis) => {
    if (vis)
      await ensureMarketI18n()
    else if (!props.embedded)
      i18nReady.value = false
  },
  { immediate: true },
)

const categories = computed(() => {
  const set = new Set<string>()
  for (const row of marketStore.pluginMarketSnapshot?.plugins ?? []) {
    const c = row.category?.trim()
    if (c)
      set.add(c)
  }
  return [...set].sort()
})

const filteredPlugins = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  const cat = categoryFilter.value.trim()
  return (marketStore.pluginMarketSnapshot?.plugins ?? []).filter((row) => {
    if (cat && (row.category?.trim() ?? '') !== cat)
      return false
    if (!q)
      return true
    const hay = [
      row.id,
      row.name,
      row.description,
      row.author,
      ...(row.tags ?? []),
    ]
      .join(' ')
      .toLowerCase()
    return hay.includes(q)
  })
})

const marketErrorText = computed(() => {
  const e = marketStore.pluginMarketError
  if (!e)
    return ''
  if (e === 'share_url_required')
    return String(t('pluginWorkbench.market.errShareRequired'))
  if (e === 'share_url_invalid')
    return String(t('pluginWorkbench.market.errShareInvalid'))
  return e
})

const showCatalogGrid = computed(
  () =>
    !marketStore.pendingGitShareUrl
    && (filteredPlugins.value.length > 0
      || (!!marketStore.pluginMarketSnapshot && !marketStore.pluginMarketSyncing)),
)

async function onLoadShareUrl() {
  marketStore.pluginMarketError = null
  try {
    await marketStore.loadFromShareUrl(marketStore.shareCatalogUrl)
    if (marketStore.pendingGitShareUrl) {
      return
    }
    if (marketStore.pluginMarketSnapshot?.warning) {
      showToast('info', t('pluginWorkbench.market.toastOfflineCache'))
    }
    else {
      showToast('success', t('pluginWorkbench.market.toastCatalogLoaded'))
    }
  }
  catch (e) {
    if (
      e instanceof Error
      && (e.message === 'share_url_required' || e.message === 'share_url_invalid')
    ) {
      return
    }
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onInstallGitShare() {
  const url = marketStore.pendingGitShareUrl
  if (!url)
    return
  try {
    await marketStore.installFromGitShare(url)
    showToast(
      'success',
      t('pluginWorkbench.market.installedGoManage', { id: url }),
    )
    marketStore.pendingGitShareUrl = null
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onInstall(row: PluginMarketEntryDto) {
  if ((row.missingDependencies ?? []).length > 0) {
    showToast(
      'error',
      t('pluginWorkbench.toast.installDeps', {
        list: row.missingDependencies.join(', '),
      }),
    )
    return
  }
  try {
    await marketStore.installFromPluginMarket(row.id, row.git)
    showToast('success', t('pluginWorkbench.market.installedGoManage', { id: row.id }))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function onUpdate(row: PluginMarketEntryDto) {
  try {
    await marketStore.updateInstalledPluginFromGit(row.id)
    showToast('success', t('pluginWorkbench.toast.updatedGit', { id: row.id }))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

function openPluginManager() {
  close()
  hostEventBus.emit('ui:open_plugin_manager')
}

function clearGitShare() {
  marketStore.clearPendingGitShare()
}

function close() {
  if (props.embedded)
    emit('back')
  else
    marketStore.closeMarketPanel()
}

const shareUrlInputId = computed(() =>
  props.embedded ? 'mk-share-url-embedded' : 'mk-share-url',
)
</script>

<template>
  <component :is="embedded ? 'div' : 'Teleport'" v-bind="embedded ? {} : { to: 'body' }">
    <div
      v-if="panelOpen && i18nReady"
      :class="embedded ? 'mk-embedded-root' : 'mk-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('pluginWorkbench.market.dialogAria')"
      @click.self="!embedded && close()"
      @keydown.escape.stop="close()"
    >
      <div
        ref="dialogRef"
        :class="embedded ? 'mk-embedded' : 'mk-dialog'"
        tabindex="-1"
        @click.stop
      >
        <header class="mk-head" :class="{ 'mk-head--embedded': embedded }">
          <div>
            <h2 v-if="!embedded" class="mk-title">
              {{ t("pluginWorkbench.market.pageTitle") }}
            </h2>
            <p class="mk-sub" :class="{ 'mk-sub--embedded': embedded }">
              {{ t("pluginWorkbench.market.pageSub") }}
            </p>
          </div>
          <button
            v-if="embedded"
            type="button"
            class="mk-btn secondary mk-back-btn"
            @click="close"
          >
            {{ t("simplePluginManager.tabInstalled") }}
          </button>
          <button
            v-else
            type="button"
            class="mk-close"
            :aria-label="t('common.close')"
            @click="close"
          >
            ×
          </button>
        </header>

        <div class="mk-share-block">
          <label class="mk-share-label" :for="shareUrlInputId">
            {{ t("pluginWorkbench.market.shareUrlLabel") }}
          </label>
          <div class="mk-share-row">
            <input
              :id="shareUrlInputId"
              ref="shareUrlRef"
              v-model="marketStore.shareCatalogUrl"
              type="url"
              class="mk-share-input"
              :placeholder="t('pluginWorkbench.market.shareUrlPlaceholder')"
              :aria-label="t('pluginWorkbench.market.shareUrlAria')"
              :disabled="marketStore.pluginMarketSyncing"
              @keydown.enter.prevent="onLoadShareUrl"
            >
            <button
              type="button"
              class="mk-btn primary"
              :disabled="marketStore.pluginMarketSyncing"
              @click="onLoadShareUrl"
            >
              {{
                marketStore.pluginMarketSyncing
                  ? t("pluginWorkbench.market.loading")
                  : t("pluginWorkbench.market.loadCatalog")
              }}
            </button>
          </div>
          <p class="mk-muted mk-share-hint">
            {{ t("pluginWorkbench.market.shareUrlHint") }}
          </p>
        </div>

        <div class="mk-toolbar">
          <input
            v-model="searchQuery"
            type="search"
            class="mk-search"
            :placeholder="t('pluginWorkbench.market.searchPlaceholder')"
            :aria-label="t('pluginWorkbench.market.searchAria')"
            :disabled="!marketStore.pluginMarketSnapshot?.plugins?.length"
          >
          <select
            v-model="categoryFilter"
            class="mk-select"
            :aria-label="t('pluginWorkbench.market.categoryAria')"
            :disabled="!categories.length"
          >
            <option value="">
              {{ t("pluginWorkbench.market.categoryAll") }}
            </option>
            <option v-for="c in categories" :key="c" :value="c">
              {{ c }}
            </option>
          </select>
          <button
            v-if="!embedded"
            type="button"
            class="mk-btn secondary"
            @click="openPluginManager"
          >
            {{ t("pluginWorkbench.market.openManager") }}
          </button>
        </div>

        <p v-if="marketStore.pluginMarketSyncing" class="mk-sync-status" role="status" aria-live="polite">
          {{ t("pluginWorkbench.market.loading") }}
        </p>
        <p v-if="marketErrorText" class="mk-err" role="alert">
          {{ marketErrorText }}
        </p>
        <div
          v-else-if="marketStore.pluginMarketSnapshot?.warning"
          class="mk-callout"
          role="status"
        >
          <p>{{ t("pluginWorkbench.market.syncFailedTitle") }}</p>
          <p class="mk-muted">
            {{
              t("pluginWorkbench.market.syncFailedDetail", {
                detail: marketStore.pluginMarketSnapshot.warning,
              })
            }}
          </p>
        </div>
        <p
          v-else-if="marketStore.pluginMarketSnapshot?.offlineMode && showCatalogGrid"
          class="mk-hint"
        >
          {{ t("pluginWorkbench.market.offline") }}
        </p>

        <div class="mk-scroll" :class="{ 'mk-scroll--embedded': embedded }">
          <section
            v-if="marketStore.pendingGitShareUrl"
            class="mk-git-card"
            aria-labelledby="mk-git-title"
          >
            <h3 id="mk-git-title" class="mk-git-title">
              {{ t("pluginWorkbench.market.gitShareTitle") }}
            </h3>
            <p class="mk-muted">
              {{ t("pluginWorkbench.market.gitShareBody") }}
            </p>
            <p class="mk-git-url">
              <code>{{ marketStore.pendingGitShareUrl }}</code>
            </p>
            <div class="mk-git-actions">
              <button
                type="button"
                class="mk-btn primary"
                :disabled="marketStore.pluginMarketSyncing"
                @click="onInstallGitShare"
              >
                {{ t("pluginWorkbench.market.gitShareInstall") }}
              </button>
              <button type="button" class="mk-btn secondary" @click="clearGitShare">
                {{ t("pluginWorkbench.market.clearGitShare") }}
              </button>
            </div>
          </section>

          <p
            v-if="!marketStore.pendingGitShareUrl && !filteredPlugins.length && !marketErrorText"
            class="mk-muted mk-empty"
          >
            {{ t("pluginWorkbench.market.emptyIndex") }}
          </p>
          <ul v-else-if="filteredPlugins.length" class="mk-grid" role="list">
            <li v-for="row in filteredPlugins" :key="row.id" class="mk-card">
              <div class="mk-card-head">
                <strong class="mk-card-id">{{ row.id }}</strong>
                <span class="mk-card-ver">v{{ row.version }}</span>
              </div>
              <p class="mk-card-name">
                {{ row.name }}
              </p>
              <p v-if="row.author" class="mk-card-meta">
                {{ t("pluginWorkbench.market.author") }}: {{ row.author }}
              </p>
              <p v-if="row.category" class="mk-card-meta">
                {{ t("pluginWorkbench.market.category") }}: {{ row.category }}
              </p>
              <p v-if="row.description" class="mk-card-desc">
                {{ row.description }}
              </p>
              <p
                v-if="(row.missingDependencies ?? []).length"
                class="mk-err mk-card-deps"
              >
                {{ t("pluginWorkbench.market.deps", { list: row.missingDependencies.join(", ") }) }}
              </p>
              <div class="mk-card-actions">
                <button
                  v-if="!row.installed"
                  type="button"
                  class="mk-btn primary"
                  @click="onInstall(row)"
                >
                  {{ t("pluginWorkbench.market.install") }}
                </button>
                <template v-else>
                  <span v-if="row.hasUpdate" class="mk-badge">{{
                    t("pluginWorkbench.market.badgeUpdate")
                  }}</span>
                  <span v-else class="mk-muted">{{ t("pluginWorkbench.market.installed") }}</span>
                  <button
                    v-if="row.hasUpdate"
                    type="button"
                    class="mk-btn secondary"
                    @click="onUpdate(row)"
                  >
                    {{ t("pluginWorkbench.market.update") }}
                  </button>
                </template>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </component>
</template>

<style scoped>
.mk-embedded-root {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.mk-embedded {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: visible;
  background: transparent;
}
.mk-head--embedded {
  padding: var(--tool-space-3, 12px) var(--tool-space-4, 16px);
}
.mk-sub--embedded {
  margin-top: 0;
}
.mk-back-btn {
  flex-shrink: 0;
}
.mk-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10055;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 45%, transparent));
}
.mk-dialog {
  width: min(960px, 100%);
  max-height: min(92vh, 880px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.mk-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 18px 12px;
  border-bottom: 1px solid var(--border-light);
}
.mk-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}
.mk-sub {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.mk-close {
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
.mk-share-block {
  padding: 12px 18px;
  border-bottom: 1px solid var(--border-light);
}
.mk-share-label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 6px;
}
.mk-share-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.mk-share-input {
  flex: 1 1 240px;
  min-width: 0;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 13px;
}
.mk-share-hint {
  margin: 8px 0 0;
  line-height: 1.45;
}
.mk-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 12px 18px;
  border-bottom: 1px solid var(--border-light);
}
.mk-search {
  flex: 1 1 200px;
  min-width: 0;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.mk-select {
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.mk-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 14px 18px 18px;
}
.mk-scroll--embedded {
  flex: initial;
  min-height: auto;
  overflow: visible;
  padding: var(--tool-space-3, 12px) var(--tool-space-4, 16px) var(--tool-space-4, 16px);
}
.mk-git-card {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  padding: 14px 16px;
  background: var(--bg-elevated);
  margin-bottom: 12px;
}
.mk-git-title {
  margin: 0 0 8px;
  font-size: 15px;
}
.mk-git-url {
  margin: 10px 0;
  word-break: break-all;
  font-size: 12px;
}
.mk-git-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.mk-grid {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}
.mk-card {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  padding: 12px;
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mk-card-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.mk-card-id {
  font-size: 14px;
}
.mk-card-ver {
  font-size: 11px;
  color: var(--text-secondary);
}
.mk-card-name {
  margin: 0;
  font-weight: 600;
  font-size: 13px;
}
.mk-card-meta,
.mk-card-desc {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.mk-card-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}
.mk-btn {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  border: 1px solid var(--border-light);
}
.mk-btn.primary {
  background: var(--accent);
  color: var(--accent-fg, #fff);
  border-color: transparent;
}
.mk-btn.secondary {
  background: var(--bg-primary);
  color: var(--text-primary);
}
.mk-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.mk-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--accent) 18%, transparent);
  color: var(--text-accent);
}
.mk-err {
  color: var(--danger, #c44);
  font-size: 12px;
  margin: 0 18px;
}
.mk-hint,
.mk-muted {
  font-size: 12px;
  color: var(--text-secondary);
}
.mk-callout {
  margin: 0 18px;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--warning, #e8a020) 12%, var(--bg-elevated));
  font-size: 12px;
}
.mk-empty {
  text-align: center;
  padding: 24px;
}
.mk-sync-status {
  margin: 0 18px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
