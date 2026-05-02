<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/api/dialog";
import { useAppToast } from "../composables/useAppToast";
import { useExpertModelsStore } from "../stores/expertModelsStore";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import type { ExpertGraph, LocalModelFileDto } from "../utils/tauri-api";
import {
  expertModelsApplyToSession,
  expertModelsDeleteLocalBaseModel,
  expertModelsImportBaseGguf,
  expertModelsListLocalBaseModels,
  expertModelsRenameLocalBaseModel,
  expertModelsSetSessionOverride,
  ollamaModelsDelete,
  ollamaModelsHealth,
  ollamaModelsListNames,
  setPluginPermissionGrant,
} from "../utils/tauri-api";

const LLAMA_LOCAL_PLUGIN_ID = "com.oclive.llama.local";
const QUICK_PERMS = ["process:spawn", "network:*"];

const emit = defineEmits<{
  requestClose: [];
}>();

const { t } = useI18n();
const { showToast } = useAppToast();
const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const expertStore = useExpertModelsStore();

const rows = ref<LocalModelFileDto[]>([]);
const loading = ref(false);
const quickBusy = ref(false);
const ollamaOpen = ref(false);
const ollamaOk = ref<boolean | null>(null);
const ollamaNames = ref<string[]>([]);
const ollamaBusy = ref(false);
const ollamaDeleteName = ref("");
const renameDraft = ref<Record<string, string>>({});

const sortedRows = computed(() =>
  [...rows.value].sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" })),
);

const llamaPluginPresent = computed(() =>
  (pluginStore.catalog ?? []).some((p) => p.id === LLAMA_LOCAL_PLUGIN_ID),
);

function renameValue(row: LocalModelFileDto): string {
  return renameDraft.value[row.path] ?? row.name;
}

function setRenameDraft(row: LocalModelFileDto, v: string): void {
  renameDraft.value = { ...renameDraft.value, [row.path]: v };
}

function graphBaseOnly(ggufPath: string): ExpertGraph {
  return {
    version: 1,
    nodes: [{ type: "base_model", id: "base", ggufPath, ui: null }],
    edges: [],
  };
}

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    rows.value = await expertModelsListLocalBaseModels();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function onImport(): Promise<void> {
  const picked = await open({ multiple: false, filters: [{ name: "GGUF", extensions: ["gguf"] }] });
  if (picked === null || Array.isArray(picked)) return;
  loading.value = true;
  try {
    const r = await expertModelsImportBaseGguf(picked);
    showToast("success", String(t("builtinLlamaModels.toastImported", { name: r.name })));
    await refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function onDelete(row: LocalModelFileDto): Promise<void> {
  if (!confirm(String(t("builtinLlamaModels.confirmDelete", { name: row.name })))) return;
  loading.value = true;
  try {
    await expertModelsDeleteLocalBaseModel(row.path);
    showToast("success", String(t("builtinLlamaModels.toastDeleted", { name: row.name })));
    await refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function onRename(row: LocalModelFileDto): Promise<void> {
  const next = renameValue(row).trim();
  if (!next || next === row.name) {
    showToast("info", String(t("builtinLlamaModels.renameUnchanged")));
    return;
  }
  loading.value = true;
  try {
    const r = await expertModelsRenameLocalBaseModel(row.path, next);
    showToast("success", String(t("builtinLlamaModels.toastRenamed", { name: r.name })));
    const nextDraft = { ...renameDraft.value };
    delete nextDraft[row.path];
    renameDraft.value = nextDraft;
    await refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function onQuickChat(row: LocalModelFileDto): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  if (!roleId) {
    showToast("error", String(t("builtinLlamaModels.noRole")));
    return;
  }
  if (!llamaPluginPresent.value) {
    showToast("error", String(t("builtinLlamaModels.pluginMissing", { id: LLAMA_LOCAL_PLUGIN_ID })));
    return;
  }
  const ok = confirm(
    String(
      t("builtinLlamaModels.confirmQuickStart", {
        name: row.name,
        id: LLAMA_LOCAL_PLUGIN_ID,
        list: QUICK_PERMS.map((p) => `- ${p}`).join("\n"),
      }),
    ),
  );
  if (!ok) return;

  quickBusy.value = true;
  try {
    for (const perm of QUICK_PERMS) {
      await setPluginPermissionGrant(LLAMA_LOCAL_PLUGIN_ID, perm, true);
    }
    await expertModelsSetSessionOverride({
      roleId,
      sessionId: null,
      graph: graphBaseOnly(row.path),
      promptStyle: null,
    });
    const r = await expertModelsApplyToSession({ roleId, sessionId: null });
    const notice = (r.sidecarNotice ?? "").trim();
    if (notice) {
      showToast("info", String(t("builtinLlamaModels.sidecarNotice", { message: notice })));
    }
    showToast("success", String(t("builtinLlamaModels.toastQuickStart", { name: row.name })));
    await pluginStore.refresh().catch(() => {});
    await expertStore.refresh().catch(() => {});
    await roleStore.refreshRoleInfo();
    emit("requestClose");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    quickBusy.value = false;
  }
}

function onOllamaDetailsToggle(ev: Event): void {
  const el = ev.target as HTMLDetailsElement | null;
  if (el?.open) void refreshOllama();
}

async function refreshOllama(): Promise<void> {
  ollamaBusy.value = true;
  ollamaOk.value = null;
  try {
    ollamaOk.value = await ollamaModelsHealth();
    if (ollamaOk.value) {
      ollamaNames.value = await ollamaModelsListNames();
    } else {
      ollamaNames.value = [];
    }
  } catch (e) {
    ollamaOk.value = false;
    ollamaNames.value = [];
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    ollamaBusy.value = false;
  }
}

async function onOllamaDelete(): Promise<void> {
  const n = ollamaDeleteName.value.trim();
  if (!n) {
    showToast("info", String(t("builtinLlamaModels.ollamaNeedName")));
    return;
  }
  if (!confirm(String(t("builtinLlamaModels.confirmOllamaDelete", { name: n })))) return;
  ollamaBusy.value = true;
  try {
    await ollamaModelsDelete(n);
    showToast("success", String(t("builtinLlamaModels.toastOllamaDeleted", { name: n })));
    ollamaDeleteName.value = "";
    await refreshOllama();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    ollamaBusy.value = false;
  }
}

onMounted(() => {
  void refresh();
  void pluginStore.refresh().catch(() => {});
});
</script>

<template>
  <section class="blm-root" :aria-label="String(t('builtinLlamaModels.aria'))">
    <div class="blm-head">
      <div>
        <h3 class="blm-h3">{{ t("builtinLlamaModels.title") }}</h3>
        <p class="blm-sub">{{ t("builtinLlamaModels.subtitle") }}</p>
      </div>
      <div class="blm-actions">
        <button type="button" class="blm-btn secondary" :disabled="loading" @click="refresh">
          {{ t("builtinLlamaModels.refresh") }}
        </button>
        <button type="button" class="blm-btn" :disabled="loading" @click="onImport">
          {{ t("builtinLlamaModels.importGguf") }}
        </button>
      </div>
    </div>

    <div v-if="loading && !rows.length" class="blm-muted">{{ t("builtinLlamaModels.loading") }}</div>
    <div v-else-if="!sortedRows.length" class="blm-muted">{{ t("builtinLlamaModels.empty") }}</div>
    <table v-else class="blm-table" :aria-label="String(t('builtinLlamaModels.tableAria'))">
      <thead>
        <tr>
          <th>{{ t("builtinLlamaModels.colName") }}</th>
          <th>{{ t("builtinLlamaModels.colPath") }}</th>
          <th class="blm-col-quick">{{ t("builtinLlamaModels.colQuick") }}</th>
          <th class="blm-col-actions">{{ t("builtinLlamaModels.colActions") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in sortedRows" :key="row.path">
          <td>
            <input
              class="blm-input"
              type="text"
              :aria-label="String(t('builtinLlamaModels.renameAria', { name: row.name }))"
              :value="renameValue(row)"
              @input="setRenameDraft(row, ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td class="blm-path" :title="row.path">{{ row.path }}</td>
          <td class="blm-quick-cell">
            <button
              type="button"
              class="blm-btn accent sm"
              :disabled="loading || quickBusy"
              @click="onQuickChat(row)"
            >
              {{ t("builtinLlamaModels.quickChat") }}
            </button>
          </td>
          <td class="blm-actions-cell">
            <button
              type="button"
              class="blm-btn secondary sm"
              :disabled="loading || quickBusy"
              @click="onRename(row)"
            >
              {{ t("builtinLlamaModels.applyRename") }}
            </button>
            <button
              type="button"
              class="blm-btn danger sm"
              :disabled="loading || quickBusy"
              @click="onDelete(row)"
            >
              {{ t("builtinLlamaModels.delete") }}
            </button>
          </td>
        </tr>
      </tbody>
    </table>

    <details class="blm-fallback" @toggle="onOllamaDetailsToggle">
      <summary>{{ t("builtinLlamaModels.ollamaSummary") }}</summary>
      <p class="blm-muted">{{ t("builtinLlamaModels.ollamaHint") }}</p>
      <div class="blm-ollama-row">
        <span class="blm-tag" :data-ok="ollamaOk === true ? '1' : '0'">
          {{
            ollamaBusy
              ? t("builtinLlamaModels.ollamaChecking")
              : ollamaOk === null
                ? t("builtinLlamaModels.ollamaUnknown")
                : ollamaOk
                  ? t("builtinLlamaModels.ollamaUp")
                  : t("builtinLlamaModels.ollamaDown")
          }}
        </span>
        <button type="button" class="blm-btn secondary sm" :disabled="ollamaBusy" @click="refreshOllama">
          {{ t("builtinLlamaModels.refreshOllama") }}
        </button>
      </div>
      <ul v-if="ollamaNames.length" class="blm-ollama-list">
        <li v-for="n in ollamaNames" :key="n">{{ n }}</li>
      </ul>
      <div v-else-if="ollamaOk && !ollamaBusy" class="blm-muted">{{ t("builtinLlamaModels.ollamaEmpty") }}</div>
      <div class="blm-ollama-del">
        <input
          v-model="ollamaDeleteName"
          class="blm-input"
          type="text"
          :placeholder="String(t('builtinLlamaModels.ollamaDeletePlaceholder'))"
          :disabled="ollamaBusy || !ollamaOk"
        />
        <button type="button" class="blm-btn danger sm" :disabled="ollamaBusy || !ollamaOk" @click="onOllamaDelete">
          {{ t("builtinLlamaModels.ollamaDelete") }}
        </button>
      </div>
    </details>
  </section>
</template>

<style scoped>
.blm-root {
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  border-radius: 10px;
  padding: 12px 14px;
  background: var(--bg-elevated, rgba(0, 0, 0, 0.2));
}
.blm-head {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 10px;
}
.blm-h3 {
  margin: 0 0 4px;
  font-size: 15px;
}
.blm-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
  max-width: 820px;
}
.blm-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.blm-actions-cell {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.blm-quick-cell {
  white-space: nowrap;
}
.blm-btn {
  border-radius: 8px;
  padding: 6px 12px;
  font-size: 12px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
  background: var(--accent, #3b82f6);
  color: #fff;
  cursor: pointer;
}
.blm-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.blm-btn.secondary {
  background: transparent;
  color: var(--text-primary, #e5e7eb);
}
.blm-btn.accent {
  background: #15803d;
  border-color: #14532d;
}
.blm-btn.danger {
  background: #b91c1c;
  border-color: #7f1d1d;
}
.blm-btn.sm {
  padding: 4px 8px;
  font-size: 11px;
}
.blm-muted {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 6px 0;
}
.blm-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}
.blm-table th,
.blm-table td {
  border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
  padding: 8px 6px;
  text-align: left;
  vertical-align: middle;
}
.blm-table th {
  color: var(--text-secondary);
  font-weight: 600;
}
.blm-col-actions {
  width: 200px;
  text-align: right;
}
.blm-col-quick {
  width: 120px;
}
.blm-path {
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
}
.blm-input {
  width: 100%;
  max-width: 220px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
  background: var(--bg-input, rgba(0, 0, 0, 0.25));
  color: inherit;
  font-size: 12px;
}
.blm-fallback {
  margin-top: 14px;
  padding-top: 10px;
  border-top: 1px dashed var(--border-subtle, rgba(255, 255, 255, 0.1));
}
.blm-fallback summary {
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}
.blm-ollama-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 8px 0;
}
.blm-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.2);
}
.blm-tag[data-ok="1"] {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}
.blm-ollama-list {
  margin: 6px 0 8px;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  max-height: 160px;
  overflow: auto;
}
.blm-ollama-del {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 6px;
}
.blm-ollama-del .blm-input {
  max-width: 320px;
}
</style>
