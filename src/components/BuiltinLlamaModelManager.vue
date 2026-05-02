<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import TrustConsentModal from "./TrustConsentModal.vue";
import { useCloudLlmTrustModal } from "../composables/useCloudLlmTrustModal";
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
  expertModelsSetGgufRepoMeta,
  expertModelsSetSessionOverride,
  ollamaModelsDelete,
  ollamaModelsHealth,
  ollamaModelsListNames,
  setPluginPermissionGrant,
} from "../utils/tauri-api";
import { appConfirm } from "../utils/confirmDialog";

const LLAMA_LOCAL_PLUGIN_ID = "com.oclive.llama.local";
const QUICK_PERMS = ["process:spawn", "network:*"];

const emit = defineEmits<{
  requestClose: [];
}>();

const { t } = useI18n();
const { showToast } = useAppToast();
const cloudTrust = useCloudLlmTrustModal();
const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const expertStore = useExpertModelsStore();

const rows = ref<LocalModelFileDto[]>([]);
const loading = ref(false);
const quickBusy = ref(false);
const ollamaOk = ref<boolean | null>(null);
const ollamaNames = ref<string[]>([]);
const ollamaBusy = ref(false);
const ollamaDeleteName = ref("");
const renameDraft = ref<Record<string, string>>({});
const repoNotesByPath = ref<Record<string, string>>({});
const repoUrlByPath = ref<Record<string, string>>({});
const repoTagsByPath = ref<Record<string, string>>({});
const repoSavePath = ref<string | null>(null);

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

function syncRepoDraftsFromRows(list: LocalModelFileDto[]): void {
  const n: Record<string, string> = {};
  const u: Record<string, string> = {};
  const tg: Record<string, string> = {};
  for (const r of list) {
    n[r.path] = r.repoNotes ?? "";
    u[r.path] = r.repoSourceUrl ?? "";
    tg[r.path] = (r.repoTags ?? []).join(", ");
  }
  repoNotesByPath.value = n;
  repoUrlByPath.value = u;
  repoTagsByPath.value = tg;
}

function setRepoNotes(path: string, v: string): void {
  repoNotesByPath.value = { ...repoNotesByPath.value, [path]: v };
}
function setRepoUrl(path: string, v: string): void {
  repoUrlByPath.value = { ...repoUrlByPath.value, [path]: v };
}
function setRepoTags(path: string, v: string): void {
  repoTagsByPath.value = { ...repoTagsByPath.value, [path]: v };
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
    syncRepoDraftsFromRows(rows.value);
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
  if (!(await appConfirm(String(t("builtinLlamaModels.confirmDelete", { name: row.name }))))) return;
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

async function onSaveRepo(row: LocalModelFileDto): Promise<void> {
  repoSavePath.value = row.path;
  try {
    const notes = (repoNotesByPath.value[row.path] ?? "").trim();
    const sourceUrl = (repoUrlByPath.value[row.path] ?? "").trim();
    const rawTags = repoTagsByPath.value[row.path] ?? "";
    const tags = rawTags
      .split(/[,，]/)
      .map((x) => x.trim())
      .filter(Boolean);
    await expertModelsSetGgufRepoMeta({ path: row.path, notes, sourceUrl, tags });
    showToast("success", String(t("builtinLlamaModels.repo.toastSaved")));
    await refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    repoSavePath.value = null;
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
  const ok = await appConfirm(
    String(
      t("builtinLlamaModels.confirmQuickStart", {
        name: row.name,
        id: LLAMA_LOCAL_PLUGIN_ID,
        list: QUICK_PERMS.map((p) => `- ${p}`).join("\n"),
      }),
    ),
    { title: String(t("builtinLlamaModels.title")), type: "warning" },
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
  if (!(await appConfirm(String(t("builtinLlamaModels.confirmOllamaDelete", { name: n }))))) return;
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
  <TrustConsentModal
    v-model="cloudTrust.visible"
    :title="cloudTrust.modalTitle"
    :subtitle="cloudTrust.modalSubtitle"
    :trust-summary-title="cloudTrust.trustSummaryTitle"
    :trust-summary="cloudTrust.trustSummaryBody"
    :hint="cloudTrust.modalHint"
    :capabilities="cloudTrust.capabilities"
    :confirm-label="cloudTrust.confirmLabel"
    variant="trust"
    require-explicit-dismiss
  />
  <section class="blm-root" :aria-label="String(t('builtinLlamaModels.aria'))">
    <div class="blm-top">
      <h3 class="blm-h3">{{ t("builtinLlamaModels.title") }}</h3>
      <button type="button" class="blm-btn secondary" :disabled="loading" @click="refresh">
        {{ t("builtinLlamaModels.refresh") }}
      </button>
    </div>
    <div class="blm-card blm-card--cloud">
      <h4 class="blm-card-title">{{ t("builtinLlamaModels.cloudTrust.title") }}</h4>
      <p class="blm-card-desc">{{ t("builtinLlamaModels.cloudTrust.hint") }}</p>
      <button type="button" class="blm-btn secondary blm-btn-inline" @click="cloudTrust.open">
        {{ t("builtinLlamaModels.cloudTrust.reviewCta") }}
      </button>
    </div>
    <p class="blm-lead">{{ t("builtinLlamaModels.guide.lead") }}</p>

    <div class="blm-card">
      <h4 class="blm-card-title">{{ t("builtinLlamaModels.guide.step1Title") }}</h4>
      <p class="blm-card-desc">{{ t("builtinLlamaModels.guide.step1Body") }}</p>
      <button type="button" class="blm-btn blm-btn-block" :disabled="loading" @click="onImport">
        {{ t("builtinLlamaModels.guide.step1Button") }}
      </button>
    </div>

    <div class="blm-card blm-card--soft">
      <h4 class="blm-card-title">{{ t("builtinLlamaModels.guide.findModelsTitle") }}</h4>
      <p class="blm-card-desc">{{ t("builtinLlamaModels.guide.findModelsBody") }}</p>
      <ul class="blm-ext-links">
        <li>
          <a
            class="blm-ext-link"
            href="https://huggingface.co/models?library=gguf"
            target="_blank"
            rel="noopener noreferrer"
          >{{ t("builtinLlamaModels.guide.linkHf") }}</a>
        </li>
        <li>
          <a
            class="blm-ext-link"
            href="https://modelscope.cn/models"
            target="_blank"
            rel="noopener noreferrer"
          >{{ t("builtinLlamaModels.guide.linkMs") }}</a>
        </li>
      </ul>
    </div>

    <div class="blm-card">
      <h4 class="blm-card-title">{{ t("builtinLlamaModels.guide.step2Title") }}</h4>
      <p class="blm-card-desc">{{ t("builtinLlamaModels.guide.step2Body") }}</p>
      <div v-if="loading && !rows.length" class="blm-muted">{{ t("builtinLlamaModels.loading") }}</div>
      <p v-else-if="!sortedRows.length" class="blm-muted">{{ t("builtinLlamaModels.empty") }}</p>
      <ul v-else class="blm-model-list" :aria-label="String(t('builtinLlamaModels.tableAria'))">
        <li v-for="(row, rowIdx) in sortedRows" :key="row.path" class="blm-model-card">
          <div class="blm-field">
            <label class="blm-lbl" :for="`blm-name-${rowIdx}`">{{ t("builtinLlamaModels.guide.nameLabel") }}</label>
            <input
              :id="`blm-name-${rowIdx}`"
              class="blm-input blm-input-wide"
              type="text"
              :aria-label="String(t('builtinLlamaModels.renameAria', { name: row.name }))"
              :value="renameValue(row)"
              @input="setRenameDraft(row, ($event.target as HTMLInputElement).value)"
            />
          </div>
          <p class="blm-path-line" :title="row.path">{{ row.path }}</p>
          <div v-if="row.repoTags?.length" class="blm-tag-row">
            <span v-for="tg in row.repoTags" :key="tg" class="blm-chip">{{ tg }}</span>
          </div>
          <div class="blm-repo">
            <p class="blm-repo-title">{{ t("builtinLlamaModels.repo.title") }}</p>
            <p class="blm-repo-hint">{{ t("builtinLlamaModels.repo.hint") }}</p>
            <label class="blm-lbl" :for="`blm-notes-${rowIdx}`">{{ t("builtinLlamaModels.repo.notesLabel") }}</label>
            <textarea
              :id="`blm-notes-${rowIdx}`"
              class="blm-textarea"
              rows="2"
              :value="repoNotesByPath[row.path] ?? ''"
              :placeholder="String(t('builtinLlamaModels.repo.notesPlaceholder'))"
              :disabled="loading || quickBusy || repoSavePath === row.path"
              @input="setRepoNotes(row.path, ($event.target as HTMLTextAreaElement).value)"
            />
            <label class="blm-lbl" :for="`blm-url-${rowIdx}`">{{ t("builtinLlamaModels.repo.urlLabel") }}</label>
            <input
              :id="`blm-url-${rowIdx}`"
              class="blm-input blm-input-wide"
              type="url"
              :value="repoUrlByPath[row.path] ?? ''"
              :placeholder="String(t('builtinLlamaModels.repo.urlPlaceholder'))"
              :disabled="loading || quickBusy || repoSavePath === row.path"
              @input="setRepoUrl(row.path, ($event.target as HTMLInputElement).value)"
            />
            <label class="blm-lbl" :for="`blm-tags-${rowIdx}`">{{ t("builtinLlamaModels.repo.tagsLabel") }}</label>
            <input
              :id="`blm-tags-${rowIdx}`"
              class="blm-input blm-input-wide"
              type="text"
              :value="repoTagsByPath[row.path] ?? ''"
              :placeholder="String(t('builtinLlamaModels.repo.tagsPlaceholder'))"
              :disabled="loading || quickBusy || repoSavePath === row.path"
              @input="setRepoTags(row.path, ($event.target as HTMLInputElement).value)"
            />
            <button
              type="button"
              class="blm-btn secondary blm-btn-block blm-repo-save"
              :disabled="loading || quickBusy || repoSavePath === row.path"
              @click="onSaveRepo(row)"
            >
              {{ t("builtinLlamaModels.repo.saveButton") }}
            </button>
          </div>
          <div class="blm-model-actions">
            <button
              type="button"
              class="blm-btn accent blm-btn-block"
              :disabled="loading || quickBusy"
              @click="onQuickChat(row)"
            >
              {{ t("builtinLlamaModels.guide.useForChat") }}
            </button>
            <div class="blm-pair">
              <button
                type="button"
                class="blm-btn secondary blm-btn-half"
                :disabled="loading || quickBusy"
                @click="onRename(row)"
              >
                {{ t("builtinLlamaModels.guide.saveName") }}
              </button>
              <button
                type="button"
                class="blm-btn danger blm-btn-half"
                :disabled="loading || quickBusy"
                @click="onDelete(row)"
              >
                {{ t("builtinLlamaModels.guide.removeFile") }}
              </button>
            </div>
          </div>
        </li>
      </ul>
    </div>

    <div class="blm-card blm-card--soft">
      <h4 class="blm-card-title">{{ t("builtinLlamaModels.guide.step3Title") }}</h4>
      <p class="blm-card-desc">{{ t("builtinLlamaModels.guide.step3Body") }}</p>
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
        <button type="button" class="blm-btn secondary" :disabled="ollamaBusy" @click="refreshOllama">
          {{ t("builtinLlamaModels.guide.checkOllama") }}
        </button>
      </div>
      <p v-if="ollamaNames.length" class="blm-ollama-caption">{{ t("builtinLlamaModels.guide.installedModels") }}</p>
      <ul v-if="ollamaNames.length" class="blm-ollama-list">
        <li v-for="n in ollamaNames" :key="n">{{ n }}</li>
      </ul>
      <div v-else-if="ollamaOk && !ollamaBusy" class="blm-muted">{{ t("builtinLlamaModels.ollamaEmpty") }}</div>
      <p class="blm-muted blm-ollama-del-hint">{{ t("builtinLlamaModels.guide.deleteLineHint") }}</p>
      <div class="blm-ollama-del">
        <input
          v-model="ollamaDeleteName"
          class="blm-input blm-input-wide"
          type="text"
          :placeholder="String(t('builtinLlamaModels.ollamaDeletePlaceholder'))"
          :disabled="ollamaBusy || !ollamaOk"
        />
        <button type="button" class="blm-btn danger" :disabled="ollamaBusy || !ollamaOk" @click="onOllamaDelete">
          {{ t("builtinLlamaModels.ollamaDelete") }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.blm-root {
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  border-radius: 10px;
  padding: 12px 14px;
  background: var(--bg-elevated, rgba(0, 0, 0, 0.2));
}
.blm-top {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}
.blm-h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
}
.blm-lead {
  margin: 0 0 14px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.blm-card {
  margin-bottom: 14px;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-light, rgba(255, 255, 255, 0.1));
  background: var(--bg-primary, rgba(0, 0, 0, 0.15));
}
.blm-card--soft {
  background: color-mix(in srgb, var(--bg-primary) 88%, var(--text-secondary) 12%);
  border-style: dashed;
}
.blm-card--cloud {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 30%, var(--border-light));
}
.blm-btn-inline {
  align-self: flex-start;
}
.blm-card-title {
  margin: 0 0 6px;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}
.blm-card-desc {
  margin: 0 0 12px;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-secondary);
}
.blm-btn {
  border-radius: 8px;
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
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
.blm-btn-block {
  display: block;
  width: 100%;
  box-sizing: border-box;
  text-align: center;
  padding: 11px 14px;
  font-size: 14px;
}
.blm-muted {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 6px 0;
}
.blm-model-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.blm-model-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  background: var(--bg-elevated, rgba(0, 0, 0, 0.2));
}
.blm-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 8px;
}
.blm-lbl {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.02em;
}
.blm-input {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
  background: var(--bg-input, rgba(0, 0, 0, 0.25));
  color: inherit;
  font-size: 13px;
}
.blm-input-wide {
  max-width: none;
}
.blm-path-line {
  margin: 0 0 10px;
  font-size: 11px;
  line-height: 1.4;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
  word-break: break-all;
}
.blm-model-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.blm-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.blm-btn-half {
  width: 100%;
  box-sizing: border-box;
  text-align: center;
  padding: 8px 10px;
  font-size: 12px;
}
.blm-ollama-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin: 8px 0 4px;
}
.blm-tag {
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.2);
}
.blm-tag[data-ok="1"] {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}
.blm-ollama-caption {
  margin: 10px 0 4px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}
.blm-ollama-list {
  margin: 0 0 8px;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-secondary);
  max-height: 160px;
  overflow: auto;
}
.blm-ollama-del-hint {
  margin-top: 10px;
}
.blm-ollama-del {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 6px;
}
.blm-ollama-del .blm-input {
  flex: 1 1 200px;
  min-width: 0;
}
.blm-ext-links {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.65;
}
.blm-ext-link {
  color: var(--text-accent, #60a5fa);
  text-decoration: underline;
  text-underline-offset: 2px;
}
.blm-ext-link:hover {
  color: color-mix(in srgb, var(--text-accent, #60a5fa) 88%, #fff 12%);
}
.blm-tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0 0 10px;
}
.blm-chip {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--bg-elevated) 85%, var(--accent) 15%);
}
.blm-repo {
  margin-bottom: 12px;
  padding: 10px 10px 12px;
  border-radius: 8px;
  border: 1px dashed var(--border-subtle, rgba(255, 255, 255, 0.12));
  background: color-mix(in srgb, var(--bg-primary) 92%, transparent);
}
.blm-repo-title {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  color: var(--text-primary);
}
.blm-repo-hint {
  margin: 0 0 10px;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.blm-textarea {
  width: 100%;
  box-sizing: border-box;
  min-height: 52px;
  margin-bottom: 8px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
  background: var(--bg-input, rgba(0, 0, 0, 0.25));
  color: inherit;
  font-size: 12px;
  font-family: inherit;
  resize: vertical;
}
.blm-repo .blm-lbl {
  margin-top: 4px;
}
.blm-repo-save {
  margin-top: 10px;
}
</style>
