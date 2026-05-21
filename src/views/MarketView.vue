<script setup lang="ts">
import { computed, ref, watch, toRef } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import { useModalFocusRestore } from "../composables/useModalFocusRestore";
import { usePluginStore } from "../stores/pluginStore";
import type { PluginMarketEntryDto } from "../utils/tauri-api";

const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const { t } = useI18n();

const searchQuery = ref("");
const categoryFilter = ref("");

const dialogRef = ref<HTMLElement | null>(null);
const firstFocusRef = ref<HTMLInputElement | null>(null);
useModalFocusRestore(toRef(pluginStore, "marketPanelVisible"), dialogRef, {
  primary: firstFocusRef,
});

watch(
  () => pluginStore.marketPanelVisible,
  (vis) => {
    if (vis) {
      void pluginStore.loadCachedPluginMarket();
    }
  },
);

const categories = computed(() => {
  const set = new Set<string>();
  for (const row of pluginStore.pluginMarketSnapshot?.plugins ?? []) {
    const c = row.category?.trim();
    if (c) set.add(c);
  }
  return [...set].sort();
});

const filteredPlugins = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  const cat = categoryFilter.value.trim();
  return (pluginStore.pluginMarketSnapshot?.plugins ?? []).filter((row) => {
    if (cat && (row.category?.trim() ?? "") !== cat) return false;
    if (!q) return true;
    const hay = [
      row.id,
      row.name,
      row.description,
      row.author,
      ...(row.tags ?? []),
    ]
      .join(" ")
      .toLowerCase();
    return hay.includes(q);
  });
});

async function onSync() {
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

async function onInstall(row: PluginMarketEntryDto) {
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
    showToast("success", t("pluginWorkbench.market.installedGoManage", { id: row.id }));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdate(row: PluginMarketEntryDto) {
  try {
    await pluginStore.updateInstalledPluginFromGit(row.id);
    showToast("success", t("pluginWorkbench.toast.updatedGit", { id: row.id }));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

function openPluginManager() {
  pluginStore.closeMarketPanel();
  void pluginStore.openPanel();
}

function close() {
  pluginStore.closeMarketPanel();
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="pluginStore.marketPanelVisible"
      class="mk-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('pluginWorkbench.market.dialogAria')"
      @click.self="close"
      @keydown.escape.stop="close"
    >
      <div ref="dialogRef" class="mk-dialog" tabindex="-1" @click.stop>
        <header class="mk-head">
          <div>
            <h2 class="mk-title">{{ t("pluginWorkbench.market.pageTitle") }}</h2>
            <p class="mk-sub">{{ t("pluginWorkbench.market.pageSub") }}</p>
          </div>
          <button type="button" class="mk-close" :aria-label="t('common.close')" @click="close">
            ×
          </button>
        </header>

        <div class="mk-toolbar">
          <input
            ref="firstFocusRef"
            v-model="searchQuery"
            type="search"
            class="mk-search"
            :placeholder="t('pluginWorkbench.market.searchPlaceholder')"
            :aria-label="t('pluginWorkbench.market.searchAria')"
          />
          <select
            v-model="categoryFilter"
            class="mk-select"
            :aria-label="t('pluginWorkbench.market.categoryAria')"
          >
            <option value="">{{ t("pluginWorkbench.market.categoryAll") }}</option>
            <option v-for="c in categories" :key="c" :value="c">{{ c }}</option>
          </select>
          <button
            type="button"
            class="mk-btn secondary"
            :disabled="pluginStore.pluginMarketSyncing"
            @click="onSync"
          >
            {{
              pluginStore.pluginMarketSyncing
                ? t("pluginWorkbench.market.syncing")
                : t("pluginWorkbench.market.sync")
            }}
          </button>
          <button type="button" class="mk-btn secondary" @click="openPluginManager">
            {{ t("pluginWorkbench.market.openManager") }}
          </button>
        </div>

        <p v-if="pluginStore.pluginMarketError" class="mk-err">{{ pluginStore.pluginMarketError }}</p>
        <div
          v-else-if="pluginStore.pluginMarketSnapshot?.warning"
          class="mk-callout"
          role="status"
        >
          <p>{{ t("pluginWorkbench.market.syncFailedTitle") }}</p>
          <p class="mk-muted">
            {{
              t("pluginWorkbench.market.syncFailedDetail", {
                detail: pluginStore.pluginMarketSnapshot.warning,
              })
            }}
          </p>
        </div>
        <p
          v-else-if="pluginStore.pluginMarketSnapshot?.offlineMode"
          class="mk-hint"
        >
          {{ t("pluginWorkbench.market.offline") }}
        </p>

        <div class="mk-scroll">
          <p
            v-if="!filteredPlugins.length && !pluginStore.pluginMarketError"
            class="mk-muted mk-empty"
          >
            {{ t("pluginWorkbench.market.emptyIndex") }}
          </p>
          <ul v-else class="mk-grid" role="list">
            <li v-for="row in filteredPlugins" :key="row.id" class="mk-card">
              <div class="mk-card-head">
                <strong class="mk-card-id">{{ row.id }}</strong>
                <span class="mk-card-ver">v{{ row.version }}</span>
              </div>
              <p class="mk-card-name">{{ row.name }}</p>
              <p v-if="row.author" class="mk-card-meta">
                {{ t("pluginWorkbench.market.author") }}: {{ row.author }}
              </p>
              <p v-if="row.category" class="mk-card-meta">
                {{ t("pluginWorkbench.market.category") }}: {{ row.category }}
              </p>
              <p v-if="row.description" class="mk-card-desc">{{ row.description }}</p>
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
  </Teleport>
</template>

<style scoped>
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
</style>
