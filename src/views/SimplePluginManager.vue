<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import { usePluginStore } from "../stores/pluginStore";
import { installPluginFromZip } from "../utils/tauri-api";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openAdvanced: [];
  openMarket: [];
}>();

const { t } = useI18n();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();

const expandedId = ref<string | null>(null);
const advancedFoldOpen = ref(false);
const busyId = ref<string | null>(null);

const rows = computed(() =>
  pluginStore.catalog.map((c) => ({
    ...c,
    disabled: pluginStore.isPluginDisabled(c.id),
    displayName: c.id,
  })),
);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void pluginStore.refresh();
      expandedId.value = null;
    }
  },
);

function toggleExpand(id: string): void {
  expandedId.value = expandedId.value === id ? null : id;
}

async function onToggleEnabled(id: string, enabled: boolean): Promise<void> {
  busyId.value = id;
  try {
    pluginStore.setPluginDisabled(id, !enabled);
    await pluginStore.persist();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busyId.value = null;
  }
}

async function onUninstall(id: string): Promise<void> {
  if (!window.confirm(t("simplePluginManager.confirmUninstall", { id }))) return;
  busyId.value = id;
  try {
    await pluginStore.uninstallPluginFromGitIndex(id);
    showToast("success", t("simplePluginManager.uninstalled", { id }));
    if (expandedId.value === id) expandedId.value = null;
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busyId.value = null;
  }
}

async function onInstallZip(): Promise<void> {
  const path = await open({
    multiple: false,
    filters: [
      { name: t("pluginWorkbench.localZipFilterName"), extensions: ["zip"] },
    ],
  });
  if (path === null || Array.isArray(path)) return;
  busyId.value = "__install__";
  try {
    const id = await installPluginFromZip(path);
    await pluginStore.refresh();
    showToast("success", t("simplePluginManager.installed", { id }));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busyId.value = null;
  }
}
</script>

<template>
  <div class="spm-root">
    <header class="spm-toolbar">
      <button type="button" class="spm-btn primary" @click="onInstallZip">
        {{ t("simplePluginManager.installZip") }}
      </button>
      <button type="button" class="spm-btn" @click="emit('openMarket')">
        {{ t("simplePluginManager.browseMarket") }}
      </button>
      <button
        type="button"
        class="spm-btn ghost"
        :aria-label="t('simplePluginManager.close')"
        @click="emit('close')"
      >
        ×
      </button>
    </header>

    <p v-if="pluginStore.error" class="spm-error" role="alert">{{ pluginStore.error }}</p>
    <p v-if="pluginStore.loading" class="spm-muted">{{ t("simplePluginManager.loading") }}</p>

    <ul v-else class="spm-list" role="list">
      <li v-if="rows.length === 0" class="spm-empty">
        {{ t("simplePluginManager.empty") }}
      </li>
      <li v-for="row in rows" :key="row.id" class="spm-row">
        <div class="spm-row-main">
          <button
            type="button"
            class="spm-name"
            :aria-expanded="expandedId === row.id"
            @click="toggleExpand(row.id)"
          >
            <span class="spm-title">{{ row.displayName }}</span>
            <span class="spm-ver">v{{ row.version }}</span>
          </button>
          <label class="spm-switch" :title="t('simplePluginManager.toggleHint')">
            <input
              type="checkbox"
              :checked="!row.disabled"
              :disabled="
                busyId === row.id ||
                (row.dependencyStatus !== 'ok' && row.disabled)
              "
              @change="onToggleEnabled(row.id, ($event.target as HTMLInputElement).checked)"
            />
            <span class="spm-switch-ui" />
          </label>
          <button
            type="button"
            class="spm-btn danger"
            :disabled="busyId === row.id"
            @click="onUninstall(row.id)"
          >
            {{ t("simplePluginManager.uninstall") }}
          </button>
        </div>
        <div v-if="expandedId === row.id" class="spm-detail">
          <p v-if="row.description" class="spm-detail-line">
            {{ row.description }}
          </p>
          <p v-if="row.author" class="spm-detail-line">
            {{ t("simplePluginManager.author") }}: {{ row.author }}
          </p>
          <p v-if="row.provides?.length" class="spm-detail-line">
            {{ t("simplePluginManager.provides") }}:
            {{ row.provides.join(", ") }}
          </p>
          <p
            v-if="row.permissions?.length"
            class="spm-detail-line"
          >
            {{ t("simplePluginManager.permissions") }}:
            {{ row.permissions.join(", ") }}
          </p>
          <p
            v-if="row.dependencyStatus !== 'ok'"
            class="spm-detail-warn"
          >
            {{ t("simplePluginManager.deps") }}:
            {{ (row.dependencyIssues ?? []).join("; ") }}
          </p>
        </div>
      </li>
    </ul>

    <footer class="spm-advanced">
      <button
        type="button"
        class="spm-advanced-toggle"
        :aria-expanded="advancedFoldOpen"
        @click="advancedFoldOpen = !advancedFoldOpen"
      >
        {{ t("simplePluginManager.advancedFold") }}
      </button>
      <div v-if="advancedFoldOpen" class="spm-advanced-body">
        <p class="spm-muted">{{ t("simplePluginManager.advancedHint") }}</p>
        <button type="button" class="spm-btn" @click="emit('openAdvanced')">
          {{ t("simplePluginManager.openAdvanced") }}
        </button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.spm-root {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
  flex: 1;
}
.spm-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
.spm-toolbar .ghost {
  margin-left: auto;
  min-width: 2rem;
}
.spm-btn {
  padding: 6px 12px;
  border-radius: var(--radius-sm, 6px);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.875rem;
}
.spm-btn.primary {
  background: var(--accent, #3b82f6);
  border-color: transparent;
  color: #fff;
}
.spm-btn.danger {
  color: #b91c1c;
  border-color: color-mix(in srgb, #b91c1c 35%, var(--border-light));
}
.spm-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.spm-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow: auto;
  flex: 1;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm, 6px);
}
.spm-row {
  border-bottom: 1px solid var(--border-light);
}
.spm-row:last-child {
  border-bottom: none;
}
.spm-row-main {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
}
.spm-name {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  color: inherit;
  padding: 0;
}
.spm-title {
  font-weight: 600;
}
.spm-ver {
  font-size: 0.8rem;
  color: var(--text-muted, #64748b);
}
.spm-switch {
  position: relative;
  display: inline-flex;
  cursor: pointer;
}
.spm-switch input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}
.spm-switch-ui {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: var(--border-light);
  transition: background 0.15s;
}
.spm-switch-ui::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.15s;
  box-shadow: 0 1px 2px rgb(0 0 0 / 20%);
}
.spm-switch input:checked + .spm-switch-ui {
  background: var(--accent, #3b82f6);
}
.spm-switch input:checked + .spm-switch-ui::after {
  transform: translateX(18px);
}
.spm-detail {
  padding: 0 12px 10px;
  font-size: 0.85rem;
  color: var(--text-muted, #64748b);
}
.spm-detail-line {
  margin: 4px 0;
}
.spm-detail-warn {
  margin: 4px 0;
  color: #b45309;
}
.spm-empty,
.spm-muted,
.spm-error {
  padding: 12px;
  margin: 0;
}
.spm-error {
  color: #b91c1c;
}
.spm-advanced {
  border-top: 1px solid var(--border-light);
  padding-top: 10px;
}
.spm-advanced-toggle {
  background: none;
  border: none;
  color: var(--text-muted, #64748b);
  cursor: pointer;
  font-size: 0.85rem;
  padding: 4px 0;
}
.spm-advanced-body {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
