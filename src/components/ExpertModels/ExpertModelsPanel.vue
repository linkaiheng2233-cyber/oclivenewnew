<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { open, save } from "@tauri-apps/api/dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useAppToast } from "../../composables/useAppToast";
import {
  buildOclexpertPayload,
  OclexpertImportError,
  parseOclexpertJson,
  validateExpertGraphNodes,
} from "../../lib/oclexpert";
import { previewEventTrigger, type EventTriggerPreviewResult } from "../../lib/eventTriggerEval";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import { useRoleStore } from "../../stores/roleStore";
const ExpertModelsCanvas = defineAsyncComponent(() => import("./ExpertModelsCanvas.vue"));
import ExpertCloudEventSection from "./ExpertCloudEventSection.vue";
import OclexpertPublishWizard from "./OclexpertPublishWizard.vue";
import type {
  ExpertGraph,
  ExpertModelsApplyResult,
  ExpertNode,
  PromptStyleOverride,
} from "../../utils/tauri-api";

const props = withDefaults(
  defineProps<{
    /** 嵌入设置页：导入预览不 Teleport，根区域可滚动 */
    embedded?: boolean;
  }>(),
  { embedded: false },
);

const store = useExpertModelsStore();
const roleStore = useRoleStore();
const { showToast } = useAppToast();
const { t } = useI18n();
const emit = defineEmits<{
  (e: "open-permissions", payload: { pluginId: string }): void;
}>();

function notifyPrimaryApplyToast(r: ExpertModelsApplyResult): void {
  if (r.useRemoteLlm) {
    const mo = (r.remoteModelOverride ?? "").trim();
    showToast(
      "success",
      String(
        t("expertModels.toasts.appliedRemote", {
          model: mo || String(t("expertModels.cloudEvent.hostDefaultModel")),
        }),
      ),
    );
  } else {
    showToast(
      "success",
      String(
        t("expertModels.toasts.appliedToSession", {
          modelPath: r.modelPath ?? String(t("expertModels.common.notSet")),
          llamaArgs: r.llamaArgs ?? String(t("expertModels.common.empty")),
        }),
      ),
    );
  }
}

function toastSidecarStructuredIfAny(r: ExpertModelsApplyResult): void {
  const msg = r.sidecarNotice?.trim();
  if (!msg) return;
  showToast(
    "warning",
    String(t("expertModels.toasts.sidecarStructured", { code: "SIDECAR_NOTICE", message: msg })),
  );
}

const saving = ref(false);
const applying = ref(false);
const editorMode = ref<"canvas" | "form">("canvas");
const selectedCanvasNodeId = ref<string | null>(null);
const runFilterStatus = ref<"all" | "ok" | "failed" | "unknown">("all");
const runFilterText = ref("");
const expandedRunIndex = ref<number | null>(null);
const expandedRunDetail = ref<any | null>(null);

/** EventTrigger workbench: simulate user/bot lines for dry-run (aligned with kernel). */
const eventTriggerTestUser = ref("");
const eventTriggerTestBot = ref("");
const eventTriggerTestResult = ref<EventTriggerPreviewResult | null>(null);

watch(selectedCanvasNodeId, () => {
  eventTriggerTestResult.value = null;
});

function runEventTriggerWorkbenchTest(): void {
  const n = selectedNode.value;
  if (!n || n.type !== "event_trigger") return;
  eventTriggerTestResult.value = previewEventTrigger(
    {
      type: "event_trigger",
      matchSubstring: n.matchSubstring,
      memoryContent: n.memoryContent,
      enabled: n.enabled !== false,
      matchScope: n.matchScope ?? "any",
    },
    eventTriggerTestUser.value,
    eventTriggerTestBot.value,
  );
}

const sourceLabel = (s: string): string => {
  if (s === "session_override") return String(t("expertModels.source.sessionOverride"));
  if (s === "role_default") return String(t("expertModels.source.roleDefault"));
  return String(t("expertModels.source.rolePackDefault"));
};

const baseModelNode = computed(() => {
  const g = store.draftGraph;
  return g.nodes.find((n) => n.type === "base_model") as
    | { type: "base_model"; id: string; ggufPath: string; ui?: any }
    | undefined;
});

const effectiveBasePath = computed(() => {
  const g = store.effectiveGraph;
  const n = g.nodes.find((x) => x.type === "base_model") as
    | { type: "base_model"; id: string; ggufPath: string }
    | undefined;
  return (n?.ggufPath ?? "").trim();
});

type EffectiveLoraNode = Extract<ExpertNode, { type: "lora_adapter" }>;
const effectiveLoras = computed<EffectiveLoraNode[]>(() =>
  (store.effectiveGraph.nodes ?? [])
    .filter((n) => n.type === "lora_adapter")
    .map((n) => n as EffectiveLoraNode)
    .filter((n) => n.enabled)
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.id.localeCompare(b.id)),
);

const selectedBaseModelPath = computed({
  get(): string {
    return baseModelNode.value?.ggufPath ?? "";
  },
  set(v: string) {
    const g: ExpertGraph = store.draftGraph;
    const nextNodes: ExpertNode[] = [...(g.nodes ?? [])].filter(
      (n) => n.type !== "base_model",
    );
    const t = (v ?? "").trim();
    if (t) {
      nextNodes.unshift({ type: "base_model", id: "base", ggufPath: t, ui: baseModelNode.value?.ui ?? null } as any);
    }
    store.draftGraph = { ...g, nodes: nextNodes };
  },
});

type LoraNode = Extract<ExpertNode, { type: "lora_adapter" }>;
const loraNodes = computed<LoraNode[]>(() =>
  (store.draftGraph.nodes ?? [])
    .filter((n) => n.type === "lora_adapter")
    .map((n) => n as LoraNode)
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.id.localeCompare(b.id)),
);

const ensurePromptStyle = (): PromptStyleOverride => {
  if (!store.draftPromptStyle) store.draftPromptStyle = {};
  return store.draftPromptStyle;
};

const selectedNode = computed<ExpertNode | null>(() => {
  const id = (selectedCanvasNodeId.value ?? "").trim();
  if (!id) return null;
  const g = store.draftGraph;
  return (g.nodes ?? []).find((n) => String((n as any).id ?? "") === id) ?? null;
});

function patchSelectedNode(patch: Partial<any>): void {
  const id = (selectedCanvasNodeId.value ?? "").trim();
  if (!id) return;
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).map((n) =>
      String((n as any).id ?? "") === id ? ({ ...(n as any), ...patch } as any) : n,
    ),
  };
}

function patchSelectedPromptStyle(patch: Partial<PromptStyleOverride>): void {
  const id = (selectedCanvasNodeId.value ?? "").trim();
  if (!id) return;
  const n = selectedNode.value as any;
  if (!n || n.type !== "prompt_style") return;
  const next = { ...(n.style ?? {}), ...patch };
  patchSelectedNode({ style: next });
  store.draftPromptStyle = { ...(store.draftPromptStyle ?? {}), ...next };
}

function addLora(path: string): void {
  const p = (path ?? "").trim();
  if (!p) return;
  const g = store.draftGraph;
  const id = `lora_${Math.random().toString(36).slice(2, 8)}`;
  const order = loraNodes.value.length;
  store.draftGraph = {
    ...g,
    nodes: [
      ...(g.nodes ?? []),
      {
        type: "lora_adapter",
        id,
        ggufPath: p,
        strength: 1.0,
        enabled: true,
        order,
        ui: null,
      },
    ],
  };
}

function removeLora(id: string): void {
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).filter((n) => !(n.type === "lora_adapter" && n.id === id)),
  };
}

function moveLora(id: string, dir: -1 | 1): void {
  const list = [...loraNodes.value];
  const idx = list.findIndex((x) => x.id === id);
  if (idx < 0) return;
  const to = idx + dir;
  if (to < 0 || to >= list.length) return;
  const a = list[idx]!;
  const b = list[to]!;
  const next = list.map((x) => ({ ...x }));
  next[idx] = { ...b, order: a.order };
  next[to] = { ...a, order: b.order };
  const others = (store.draftGraph.nodes ?? []).filter((n) => n.type !== "lora_adapter");
  store.draftGraph = { ...store.draftGraph, nodes: [...others, ...next] };
}

const strengthWarning = (v: number): string | null => {
  if (!Number.isFinite(v)) return String(t("expertModels.strengthWarning.mustBeNumber"));
  if (v < 0) return String(t("expertModels.strengthWarning.ltZero"));
  if (v > 2) return String(t("expertModels.strengthWarning.gtTwo"));
  if (v > 1.4) return String(t("expertModels.strengthWarning.highSuggestion"));
  return null;
};

const isDraftGraphEmpty = computed(() => (store.draftGraph.nodes?.length ?? 0) === 0);

const canLoadRoleDefaultEmpty = computed(
  () => !!store.roleDefaultGraph && (store.roleDefaultGraph.nodes?.length ?? 0) > 0,
);

const draftGraphValidationMessage = computed((): string | null => {
  try {
    validateExpertGraphNodes(store.draftGraph);
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : String(e);
  }
});

type OclexpertImportPreview = {
  graph: ExpertGraph;
  promptStyle: PromptStyleOverride | null;
  suggestedName?: string;
  suggestedDescription?: string;
  suggestedAuthor?: string;
};

function summarizeExpertGraphNodes(graph: ExpertGraph): string {
  const nodes = graph.nodes ?? [];
  if (nodes.length === 0) return String(t("expertModels.oclexpert.previewGraphEmpty"));
  const counts = new Map<string, number>();
  for (const n of nodes) {
    const ty = String((n as { type?: string }).type ?? "?").trim() || "?";
    counts.set(ty, (counts.get(ty) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([ty, c]) => `${ty}×${c}`)
    .join(" · ");
}

function oclexpertImportPrivacySummary(graph: ExpertGraph): string {
  const hasTriggers = (graph.nodes ?? []).some((n) => (n as { type?: string }).type === "event_trigger");
  const hasCloud = (graph.nodes ?? []).some((n) => (n as { type?: string }).type === "cloud_model");
  const parts: string[] = [String(t("expertModels.oclexpert.previewPrivacyBaseline"))];
  if (hasTriggers) parts.push(String(t("expertModels.oclexpert.previewPrivacyTriggers")));
  if (hasCloud) parts.push(String(t("expertModels.oclexpert.previewPrivacyCloud")));
  return parts.join(" ");
}

const oclexpertDescriptionDraft = ref("");
const oclexpertAuthorDraft = ref("");
const oclexpertImportPreview = ref<OclexpertImportPreview | null>(null);
/** Last successful .oclexpert save path (for “publish to market” helper). */
const lastExportedOclexpertPath = ref("");
const publishWizardOpen = ref(false);

const OCLEXPERT_ROLES_INDEX = "https://github.com/linkaiheng2233-cyber/awesome-oclive-roles";

watch(
  () => roleStore.roleInfo?.author,
  (a) => {
    const s = (a ?? "").trim();
    if (s && !oclexpertAuthorDraft.value.trim()) oclexpertAuthorDraft.value = s;
  },
  { immediate: true },
);

function onNewBlankExpertRecipe(): void {
  store.draftGraph = { version: 1, nodes: [], edges: [] };
  store.draftPromptStyle = null;
}

function cancelOclexpertImportPreview(): void {
  oclexpertImportPreview.value = null;
}

async function confirmOclexpertImportPreview(): Promise<void> {
  const p = oclexpertImportPreview.value;
  if (!p) return;
  saving.value = true;
  try {
    store.draftGraph = p.graph;
    store.draftPromptStyle = p.promptStyle;
    const name =
      p.suggestedName?.trim() ||
      workflowNameDraft.value.trim() ||
      String(t("expertModels.oclexpert.importDefaultName"));
    const wf = await store.saveWorkflow(name, null);
    workflowNameDraft.value = wf.name;
    if (p.suggestedDescription?.trim()) oclexpertDescriptionDraft.value = p.suggestedDescription.trim();
    if (p.suggestedAuthor?.trim()) oclexpertAuthorDraft.value = p.suggestedAuthor.trim();
    oclexpertImportPreview.value = null;
    showToast("success", String(t("expertModels.oclexpert.toastImported", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onRefresh(): Promise<void> {
  await store.refresh();
  await store.refreshWorkflows().catch(() => {});
  if (store.error) showToast("error", store.error);
}

async function onApplySession(): Promise<void> {
  if (applying.value) return;
  applying.value = true;
  try {
    const r = await store.applyToSession();
    if (!r.ok) {
      showToast("error", String(t("expertModels.toasts.applyFailedHint")));
      return;
    }
    notifyPrimaryApplyToast(r);
    toastSidecarStructuredIfAny(r);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    applying.value = false;
  }
}

async function onRollbackLastRun(): Promise<void> {
  const ok = window.confirm(String(t("expertModels.confirm.rollbackLastRun")));
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.rollbackLastRun();
    notifyPrimaryApplyToast(r);
    toastSidecarStructuredIfAny(r);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
    applying.value = false;
  }
}

function formatRunTime(ms: number): string {
  const d = new Date(ms);
  if (!Number.isFinite(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function formatRelative(ms: number): string {
  const d = Date.now() - ms;
  if (!Number.isFinite(d)) return "";
  if (d < 1000) return String(t("expertModels.relative.justNow"));
  const s = Math.floor(d / 1000);
  if (s < 60) return String(t("expertModels.relative.secondsAgo", { n: s }));
  const m = Math.floor(s / 60);
  if (m < 60) return String(t("expertModels.relative.minutesAgo", { n: m }));
  const h = Math.floor(m / 60);
  if (h < 48) return String(t("expertModels.relative.hoursAgo", { n: h }));
  const day = Math.floor(h / 24);
  return String(t("expertModels.relative.daysAgo", { n: day }));
}

const filteredRuns = computed(() => {
  const text = runFilterText.value.trim().toLowerCase();
  return (store.runs ?? []).filter((r) => {
    const ok = r.applyOk === true;
    const failed = r.applyOk === false;
    const unknown = r.applyOk == null;
    if (runFilterStatus.value === "ok" && !ok) return false;
    if (runFilterStatus.value === "failed" && !failed) return false;
    if (runFilterStatus.value === "unknown" && !unknown) return false;
    if (text) {
      const hay = `${r.targetBaseName ?? ""}`.toLowerCase();
      if (!hay.includes(text)) return false;
    }
    return true;
  });
});

async function onToggleRunDetail(indexFromLatest: number): Promise<void> {
  if (expandedRunIndex.value === indexFromLatest) {
    expandedRunIndex.value = null;
    expandedRunDetail.value = null;
    return;
  }
  expandedRunIndex.value = indexFromLatest;
  expandedRunDetail.value = null;
  try {
    expandedRunDetail.value = await store.getRunDetail(indexFromLatest);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onRetryRun(indexFromLatest: number): Promise<void> {
  const d = await store.getRunDetail(indexFromLatest);
  const tg = (d.targetGraph ?? null) as ExpertGraph | null;
  const ts = (d.targetPromptStyle ?? null) as PromptStyleOverride | null;
  if (!tg)
    throw new Error(String(t("expertModels.runHistory.errors.noTargetGraphForRetry")));
  const ok = window.confirm(
    String(
      t("expertModels.confirm.retryRunApply", {
        base: d.targetBaseName || String(t("expertModels.common.notSet")),
        loras: d.targetLoraCount,
        promptStyle: d.targetHasPromptStyle
          ? String(t("expertModels.common.yes"))
          : String(t("expertModels.common.no")),
      }),
    ),
  );
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.applySpecificToSession(tg, ts);
    notifyPrimaryApplyToast(r);
    toastSidecarStructuredIfAny(r);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
    applying.value = false;
  }
}

async function onCopyRunDiagnostics(indexFromLatest: number): Promise<void> {
  const d = await store.getRunDetail(indexFromLatest);
  const payload = {
    version: 1,
    atMs: d.atMs,
    indexFromLatest: d.indexFromLatest,
    snapshot: {
      base: d.snapshotBaseName,
      loras: d.snapshotLoraCount,
      hasPromptStyle: d.snapshotHasPromptStyle,
    },
    target: {
      base: d.targetBaseName,
      loras: d.targetLoraCount,
      hasPromptStyle: d.targetHasPromptStyle,
    },
    apply: {
      ok: d.applyOk,
      error: d.applyError,
      modelPath: d.applyModelPath,
      llamaArgs: d.applyLlamaArgs,
      durationMs: d.applyDurationMs,
      sidecarNotice: d.applySidecarNotice,
    },
  };
  await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
  showToast("success", String(t("expertModels.runHistory.toastCopiedDiagnostics")));
}

function suggestWorkflowNameFromRun(d: any): string {
  const base = String(d?.targetBaseName ?? "").trim() || "workflow";
  const safe = base.replace(/[\\/:*?"<>|]/g, "_").slice(0, 60);
  const ts = new Date(d?.atMs ?? Date.now());
  const pad = (n: number) => String(n).padStart(2, "0");
  const stamp = `${ts.getFullYear()}${pad(ts.getMonth() + 1)}${pad(ts.getDate())}-${pad(ts.getHours())}${pad(ts.getMinutes())}`;
  return `${safe}-${stamp}`;
}

async function onSaveRunAsWorkflow(indexFromLatest: number): Promise<void> {
  saving.value = true;
  try {
    const d = await store.getRunDetail(indexFromLatest);
    const tg = (d.targetGraph ?? null) as ExpertGraph | null;
    const ts = (d.targetPromptStyle ?? null) as PromptStyleOverride | null;
    if (!tg)
      throw new Error(String(t("expertModels.runHistory.errors.noTargetGraphForSaveWorkflow")));
    const name =
      window
        .prompt(
          String(t("expertModels.runHistory.prompts.saveAsWorkflowName")),
          suggestWorkflowNameFromRun(d),
        )
        ?.trim() ?? "";
    if (!name) return;
    const wf = await store.saveWorkflowFromConfig(name, tg, ts, null);
    workflowNameDraft.value = wf.name;
    showToast("success", String(t("expertModels.runHistory.toastSavedToLibrary", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onExportRunAsWorkflowJson(indexFromLatest: number): Promise<void> {
  saving.value = true;
  try {
    const d = await store.getRunDetail(indexFromLatest);
    const tg = (d.targetGraph ?? null) as ExpertGraph | null;
    const ts = (d.targetPromptStyle ?? null) as PromptStyleOverride | null;
    if (!tg)
      throw new Error(String(t("expertModels.runHistory.errors.noTargetGraphForExportWorkflow")));
    const payload = {
      version: 1,
      name: suggestWorkflowNameFromRun(d),
      graph: tg,
      promptStyle: ts ?? null,
    };
    const ok = window.confirm(
      String(
        t("expertModels.confirm.exportWorkflowFile", {
          base: d.targetBaseName || String(t("expertModels.common.notSet")),
          loras: d.targetLoraCount,
          promptStyle: d.targetHasPromptStyle
            ? String(t("expertModels.common.yes"))
            : String(t("expertModels.common.no")),
          filename: `${payload.name}.oclive-workflow.json`,
        }),
      ),
    );
    if (!ok) return;
    const content = JSON.stringify(payload, null, 2);
    const path = await save({
      defaultPath: `${payload.name}.oclive-workflow.json`,
      filters: [{ name: "Workflow JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await writeTextFile(path, content);
    showToast("success", String(t("expertModels.runHistory.toastExportedShareable")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onExportLatestPinnedRun(): Promise<void> {
  const pinned = (store.runs ?? []).find((r) => r.pinned === true);
  if (!pinned) {
    showToast("info", String(t("expertModels.runHistory.toastNoPinnedRuns")));
    return;
  }
  await onExportRunAsWorkflowJson(pinned.indexFromLatest);
}

async function onRollbackToRun(indexFromLatest: number): Promise<void> {
  let summary = "";
  try {
    const d = await store.getRunDetail(indexFromLatest);
    summary = String(
      t("expertModels.confirm.rollbackSummaryLine", {
        base: d.snapshotBaseName || String(t("expertModels.common.notSet")),
        loras: d.snapshotLoraCount,
        promptStyle: d.snapshotHasPromptStyle
          ? String(t("expertModels.common.yes"))
          : String(t("expertModels.common.no")),
      }),
    );
  } catch {
    // ignore
  }
  const ok = window.confirm(String(t("expertModels.confirm.rollbackToSelectedRun", { summary })));
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.rollbackToRun(indexFromLatest);
    notifyPrimaryApplyToast(r);
    toastSidecarStructuredIfAny(r);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
    applying.value = false;
  }
}

async function onClearRuns(): Promise<void> {
  const ok = window.confirm(String(t("expertModels.confirm.clearRunsAll")));
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRuns();
    showToast("success", String(t("expertModels.runHistory.toastCleared")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

const clearMode = ref<"all" | "ok" | "failed" | "unpinned">("all");
const clearKeepPinned = ref(true);

async function onClearRunsAdvanced(): Promise<void> {
  const modeLabel = String(
    t(
      clearMode.value === "ok"
        ? "expertModels.runHistory.clearMode.ok"
        : clearMode.value === "failed"
          ? "expertModels.runHistory.clearMode.failed"
          : clearMode.value === "unpinned"
            ? "expertModels.runHistory.clearMode.unpinned"
            : "expertModels.runHistory.clearMode.all",
    ),
  );
  const ok = window.confirm(
    String(
      t("expertModels.confirm.clearRunsWithMode", {
        modeLabel,
        keepPinned: clearKeepPinned.value ? String(t("expertModels.runHistory.keepPinned")) : "",
      }),
    ),
  );
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRunsWithMode(clearMode.value, clearKeepPinned.value);
    showToast("success", String(t("expertModels.runHistory.toastClearedWithMode")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onTogglePinned(indexFromLatest: number, pinned: boolean | null | undefined): Promise<void> {
  try {
    await store.setRunPinned(indexFromLatest, !pinned);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onImportBase(): Promise<void> {
  const picked = await open({
    title: String(t("expertModels.import.baseDialogTitle")),
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importBaseGguf(p);
    showToast("success", String(t("expertModels.toasts.importedBase")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onImportLora(): Promise<void> {
  const picked = await open({
    title: String(t("expertModels.import.loraDialogTitle")),
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importLoraGguf(p);
    showToast("success", String(t("expertModels.toasts.importedLora")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onSetRoleDefault(): Promise<void> {
  saving.value = true;
  try {
    await store.setRoleDefault();
    showToast("success", String(t("expertModels.toasts.setAsRoleDefault")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearSessionOverride(): Promise<void> {
  const ok = window.confirm(String(t("expertModels.confirm.clearSessionOverrideAndApply")));
  if (!ok) return;
  saving.value = true;
  try {
    const r = await store.clearSessionOverrideAndApply();
    showToast("success", String(t("expertModels.toasts.clearedSessionOverrideAndApplied")));
    toastSidecarStructuredIfAny(r);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearRoleDefault(): Promise<void> {
  const ok = window.confirm(String(t("expertModels.confirm.clearRoleDefault")));
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRoleDefault();
    showToast("success", String(t("expertModels.toasts.clearedRoleDefault")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  if (!store.baseModels.length && !store.loading) void onRefresh();
});

const workflowNameDraft = ref<string>("");

async function onSaveWorkflowAs(): Promise<void> {
  saving.value = true;
  try {
    const name = workflowNameDraft.value.trim();
    const wf = await store.saveWorkflow(name || String(t("expertModels.workflows.unnamedDefault")), null);
    workflowNameDraft.value = wf.name;
    showToast("success", String(t("expertModels.workflows.toastSaved", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onOverwriteWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", String(t("expertModels.workflows.toastPickFirstForOverwrite")));
    return;
  }
  const ok = window.confirm(String(t("expertModels.workflows.confirmOverwrite")));
  if (!ok) return;
  saving.value = true;
  try {
    const name =
      workflowNameDraft.value.trim() ||
      store.workflows.find((w) => w.id === wid)?.name ||
      String(t("expertModels.workflows.defaultName"));
    const wf = await store.saveWorkflow(name, wid);
    workflowNameDraft.value = wf.name;
    showToast("success", String(t("expertModels.workflows.toastOverwritten", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onLoadWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", String(t("expertModels.workflows.toastPickFirst")));
    return;
  }
  saving.value = true;
  try {
    const wf = await store.loadWorkflow(wid);
    try {
      validateExpertGraphNodes(store.draftGraph);
    } catch (ve) {
      showToast("error", ve instanceof Error ? ve.message : String(ve));
      if (window.confirm(String(t("expertModels.oclexpert.offerResetEffective")))) {
        store.setDraftFromEffective();
      } else {
        await store.refresh();
      }
      return;
    }
    workflowNameDraft.value = wf.name;
    showToast("success", String(t("expertModels.workflows.toastLoaded", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onDeleteWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", String(t("expertModels.workflows.toastPickFirst")));
    return;
  }
  const name = store.workflows.find((w) => w.id === wid)?.name ?? wid;
  const ok = window.confirm(String(t("expertModels.workflows.confirmDelete", { name })));
  if (!ok) return;
  saving.value = true;
  try {
    await store.deleteWorkflow(wid);
    showToast("success", String(t("expertModels.workflows.toastDeleted")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onExportWorkflowJson(): Promise<void> {
  const payload = {
    version: 1,
    name: workflowNameDraft.value.trim() || "workflow",
    graph: store.draftGraph,
    promptStyle: store.draftPromptStyle ?? null,
  };
  const content = JSON.stringify(payload, null, 2);
  const path = await save({
    defaultPath: `${payload.name}.oclive-workflow.json`,
    filters: [{ name: "Workflow JSON", extensions: ["json"] }],
  });
  if (!path) return;
  await writeTextFile(path, content);
  showToast("success", String(t("expertModels.workflows.toastExported")));
}

async function onImportWorkflowJson(): Promise<void> {
  const picked = await open({
    title: String(t("expertModels.workflows.dialogImportTitle")),
    multiple: false,
    directory: false,
    filters: [{ name: "Workflow JSON", extensions: ["json"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    const raw = await readTextFile(p);
    const v = JSON.parse(raw ?? "{}") as any;
    const name =
      String(v?.name ?? String(t("expertModels.workflows.importDefaultName"))).trim() ||
      String(t("expertModels.workflows.importDefaultName"));
    const g = (v?.graph ?? { version: 1, nodes: [], edges: [] }) as ExpertGraph;
    try {
      validateExpertGraphNodes(g);
    } catch (ve) {
      showToast("error", ve instanceof Error ? ve.message : String(ve));
      if (window.confirm(String(t("expertModels.oclexpert.offerResetEffective")))) {
        store.setDraftFromEffective();
      }
      return;
    }
    store.draftGraph = g;
    store.draftPromptStyle = (v?.promptStyle ?? null) as any;
    // save into library
    const wf = await store.saveWorkflow(name, null);
    workflowNameDraft.value = wf.name;
    showToast("success", String(t("expertModels.workflows.toastImportedAndSaved", { name: wf.name })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

function onPublishWizardSyncDrafts(p: { name: string; description: string; author: string }): void {
  workflowNameDraft.value = p.name;
  oclexpertDescriptionDraft.value = p.description;
  oclexpertAuthorDraft.value = p.author;
}

function openCommunityRecipesIndex(): void {
  void openExternal(OCLEXPERT_ROLES_INDEX);
}

async function onExportOclexpert(): Promise<void> {
  const name = workflowNameDraft.value.trim();
  const desc = oclexpertDescriptionDraft.value.trim();
  const author = oclexpertAuthorDraft.value.trim();
  if (!name || !desc || !author) {
    showToast("warning", String(t("expertModels.oclexpert.exportRequiredFields")));
    return;
  }
  const base = name;
  const payload = buildOclexpertPayload(store.draftGraph, store.draftPromptStyle, {
    name,
    description: desc,
    author,
  });
  const path = await save({
    defaultPath: `${base}.oclexpert`,
    filters: [
      { name: String(t("expertModels.oclexpert.filterName")), extensions: ["oclexpert"] },
      { name: "JSON", extensions: ["json"] },
    ],
  });
  if (!path) return;
  await writeTextFile(path, JSON.stringify(payload, null, 2));
  lastExportedOclexpertPath.value = path;
  showToast("success", String(t("expertModels.oclexpert.toastExported")));
}

async function onImportOclexpert(): Promise<void> {
  const picked = await open({
    title: String(t("expertModels.oclexpert.dialogTitle")),
    multiple: false,
    filters: [
      { name: String(t("expertModels.oclexpert.filterName")), extensions: ["oclexpert", "json"] },
    ],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    const raw = await readTextFile(p);
    const parsed = parseOclexpertJson(raw);
    oclexpertImportPreview.value = {
      graph: parsed.graph,
      promptStyle: parsed.promptStyle,
      suggestedName: parsed.suggestedName,
      suggestedDescription: parsed.suggestedDescription,
      suggestedAuthor: parsed.suggestedAuthor,
    };
  } catch (e) {
    const msg =
      e instanceof OclexpertImportError
        ? e.message
        : e instanceof Error
          ? e.message
          : String(e);
    showToast("error", msg);
    if (window.confirm(String(t("expertModels.oclexpert.offerResetEffective")))) {
      store.setDraftFromEffective();
    }
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <section
    class="em-root"
    :class="{ 'em-root--embedded': props.embedded }"
    :aria-label="t('expertModels.title')"
  >
    <header class="em-h">
      <div>
        <h3 class="em-title">{{ t("expertModels.title") }}</h3>
        <p class="em-sub">
          {{ t("expertModels.subtitle") }}
        </p>
      </div>
      <div class="em-actions">
        <button class="em-btn secondary" type="button" :disabled="store.loading || saving" @click="onRefresh">
          {{ t("expertModels.actions.refresh") }}
        </button>
        <button class="em-btn" type="button" :disabled="store.loading || saving" @click="store.setDraftFromEffective">
          {{ t("expertModels.actions.backfillFromEffective") }}
        </button>
      </div>
    </header>

    <div class="em-meta">
      <div class="em-pill">
        {{ t("expertModels.meta.graphSource") }}：<b>{{ sourceLabel(store.graphSource) }}</b>
      </div>
      <div class="em-pill">
        {{ t("expertModels.meta.promptStyleSource") }}：<b>{{ sourceLabel(store.promptStyleSource) }}</b>
      </div>
      <div v-if="store.llamaMissingMechanismPerms.length" class="em-warnbar">
        <div>
          <b>{{ t("expertModels.permsMissing.title") }}</b>
          <span class="em-muted2">
            {{ t("expertModels.permsMissing.hint", { list: store.llamaMissingMechanismPerms.join('、') }) }}
          </span>
        </div>
        <button
          type="button"
          class="em-btn danger"
          @click="emit('open-permissions', { pluginId: 'com.oclive.llama.local' })"
        >
          {{ t("expertModels.permsMissing.goGrant") }}
        </button>
      </div>
      <div v-if="store.error" class="em-err">{{ store.error }}</div>
    </div>

    <div class="em-workflows">
      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.workflows.title") }}</div>
        <div class="em-wf-form">
          <label class="em-wf-label" for="em-workflow-name">{{ t("expertModels.workflows.nameLabel") }}</label>
          <input
            id="em-workflow-name"
            v-model="workflowNameDraft"
            class="em-input"
            type="text"
            :placeholder="t('expertModels.workflows.namePlaceholder')"
          />
          <label class="em-wf-label" for="em-oclexpert-desc">{{ t("expertModels.oclexpert.metaDescriptionLabel") }}</label>
          <textarea
            id="em-oclexpert-desc"
            v-model="oclexpertDescriptionDraft"
            class="em-text"
            rows="2"
            :placeholder="String(t('expertModels.oclexpert.metaDescriptionPlaceholder'))"
          />
          <label class="em-wf-label" for="em-oclexpert-author">{{ t("expertModels.oclexpert.metaAuthorLabel") }}</label>
          <input
            id="em-oclexpert-author"
            v-model="oclexpertAuthorDraft"
            class="em-input"
            type="text"
            :placeholder="String(t('expertModels.oclexpert.metaAuthorPlaceholder'))"
          />
          <label class="em-wf-label" for="em-workflow-lib">{{ t("expertModels.workflows.libraryLabel") }}</label>
          <div class="em-wf-inline">
            <select id="em-workflow-lib" v-model="store.pickedWorkflowId" class="em-select em-wf-inline-grow">
              <option value="">{{ t("expertModels.workflows.notSelected") }}</option>
              <option v-for="w in store.workflows" :key="w.id" :value="w.id">
                {{ w.name }}
              </option>
            </select>
            <button class="em-btn secondary em-wf-inline-btn" type="button" :disabled="saving" @click="onLoadWorkflow">
              {{ t("expertModels.workflows.load") }}
            </button>
          </div>
        </div>
        <div class="em-wf-actions">
          <button class="em-btn" type="button" :disabled="saving" @click="onSaveWorkflowAs">{{ t("expertModels.workflows.saveAsNew") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onOverwriteWorkflow">{{ t("expertModels.workflows.overwriteSave") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onDeleteWorkflow">{{ t("expertModels.workflows.delete") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onExportWorkflowJson">{{ t("expertModels.workflows.exportFile") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onImportWorkflowJson">{{ t("expertModels.workflows.importFile") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onExportOclexpert">{{ t("expertModels.oclexpert.export") }}</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onImportOclexpert">{{ t("expertModels.oclexpert.import") }}</button>
        </div>
        <div class="em-wf-market">
          <button type="button" class="em-btn secondary" @click="publishWizardOpen = true">
            {{ t("expertModels.oclexpert.publishRecipe") }}
          </button>
          <button type="button" class="em-btn secondary" @click="openCommunityRecipesIndex">
            {{ t("expertModels.oclexpert.browseRecipes") }}
          </button>
          <span class="em-muted em-wf-market-hint">{{ t("expertModels.oclexpert.publishRecipeShortHint") }}</span>
        </div>
        <p class="em-muted em-wf-hint">
          {{ t("expertModels.workflows.hint") }}
        </p>
      </div>
    </div>

    <div class="em-editorbar">
      <div class="em-pill">
        {{ t("expertModels.editor.label") }}：
        <button
          type="button"
          class="em-mini"
          :class="{ on: editorMode === 'canvas' }"
          @click="editorMode = 'canvas'"
        >
          {{ t("expertModels.editor.canvas") }}
        </button>
        <button
          type="button"
          class="em-mini"
          :class="{ on: editorMode === 'form' }"
          @click="editorMode = 'form'"
        >
          {{ t("expertModels.editor.form") }}
        </button>
      </div>
      <div class="em-muted">{{ t("expertModels.editor.canvasHint") }}</div>
    </div>

    <div v-if="draftGraphValidationMessage" class="em-card em-integrity">
      <div class="em-card-h">{{ t("expertModels.graphIntegrity.title") }}</div>
      <p class="em-muted em-block-spaced">{{ draftGraphValidationMessage }}</p>
      <div class="em-wf-actions em-wf-actions--compact">
        <button type="button" class="em-btn secondary" @click="store.setDraftFromEffective">
          {{ t("expertModels.graphIntegrity.resetEffective") }}
        </button>
        <button type="button" class="em-btn secondary" @click="editorMode = 'form'">
          {{ t("expertModels.graphIntegrity.openForm") }}
        </button>
      </div>
    </div>

    <div v-else-if="isDraftGraphEmpty" class="em-card em-empty-graph">
      <p class="em-muted em-block-spaced">{{ t("expertModels.emptyState.lead") }}</p>
      <div class="em-wf-actions em-wf-actions--compact">
        <button type="button" class="em-btn" @click="store.setDraftFromEffective">
          {{ t("expertModels.emptyState.loadEffective") }}
        </button>
        <button
          v-if="canLoadRoleDefaultEmpty"
          type="button"
          class="em-btn secondary"
          @click="store.loadRoleDefaultIntoDraft"
        >
          {{ t("expertModels.emptyState.loadRoleDefault") }}
        </button>
        <button type="button" class="em-btn secondary" @click="onNewBlankExpertRecipe">
          {{ t("expertModels.emptyState.newBlank") }}
        </button>
      </div>
    </div>

    <div class="em-cloud-event-wrap">
      <ExpertCloudEventSection />
    </div>

    <div v-if="editorMode === 'canvas'" class="em-canvaswrap">
      <ExpertModelsCanvas v-model="store.draftGraph" v-model:selectedNodeId="selectedCanvasNodeId" />
    </div>

    <div v-if="editorMode === 'canvas' && selectedNode" class="em-inspector">
      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.inspector.title") }}：{{ (selectedNode as any).type }} · {{ (selectedNode as any).id }}</div>

        <template v-if="(selectedNode as any).type === 'base_model'">
          <select
            class="em-select"
            :value="(selectedNode as any).ggufPath"
            @change="patchSelectedNode({ ggufPath: ($event.target as HTMLSelectElement).value })"
          >
            <option value="">{{ t("expertModels.common.notSet") }}</option>
            <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
          <div class="em-muted">{{ t("expertModels.inspector.baseHint") }}</div>
        </template>

        <template v-else-if="(selectedNode as any).type === 'lora_adapter'">
          <select
            class="em-select"
            :value="(selectedNode as any).ggufPath"
            @change="patchSelectedNode({ ggufPath: ($event.target as HTMLSelectElement).value })"
          >
            <option value="">{{ t("expertModels.inspector.pickLora") }}</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>

          <label class="em-field" style="margin-top: 8px">
            <div class="em-label">{{ t("expertModels.inspector.strengthLabel") }}</div>
            <input
              class="em-num"
              type="number"
              step="0.05"
              :value="(selectedNode as any).strength"
              @input="patchSelectedNode({ strength: Number(($event.target as HTMLInputElement).value) })"
            />
            <div v-if="strengthWarning(Number((selectedNode as any).strength))" class="em-warn">
              {{ strengthWarning(Number((selectedNode as any).strength)) }}
            </div>
          </label>

          <label class="em-row" style="margin-top: 8px">
            <input
              type="checkbox"
              :checked="(selectedNode as any).enabled"
              @change="patchSelectedNode({ enabled: ($event.target as HTMLInputElement).checked })"
            />
            <span class="em-muted">{{ t("expertModels.inspector.enableLora") }}</span>
          </label>
        </template>

        <template v-else-if="(selectedNode as any).type === 'cloud_model'">
          <div class="em-muted">{{ t("expertModels.inspector.cloudHint") }}</div>
          <label class="em-field" style="margin-top: 8px">
            <div class="em-label">{{ t("expertModels.cloudEvent.modelIdLabel") }}</div>
            <input
              class="em-input"
              type="text"
              :value="(selectedNode as any).model ?? ''"
              :placeholder="String(t('expertModels.cloudEvent.modelIdPlaceholder'))"
              @input="
                patchSelectedNode({
                  model: ($event.target as HTMLInputElement).value.trim() || null,
                })
              "
            />
          </label>
          <label class="em-row" style="margin-top: 8px">
            <input
              type="checkbox"
              :checked="(selectedNode as any).enabled !== false"
              @change="patchSelectedNode({ enabled: ($event.target as HTMLInputElement).checked })"
            />
            <span class="em-muted">{{ t("expertModels.cloudEvent.enabled") }}</span>
          </label>
        </template>

        <template v-else-if="(selectedNode as any).type === 'event_trigger'">
          <div class="em-muted">{{ t("expertModels.inspector.eventHint") }}</div>

          <div class="em-etw-section">
            <div class="em-etw-h">{{ t("expertModels.eventTriggerWorkbench.sectionCondition") }}</div>
            <label class="em-field" style="margin-top: 6px">
              <div class="em-label">{{ t("expertModels.eventTriggerWorkbench.scopeLabel") }}</div>
              <select
                class="em-select"
                :value="(selectedNode as any).matchScope ?? 'any'"
                @change="
                  patchSelectedNode({
                    matchScope: ($event.target as HTMLSelectElement).value as any,
                  })
                "
              >
                <option value="any">{{ t("expertModels.eventTriggerWorkbench.scopeAny") }}</option>
                <option value="user_only">{{ t("expertModels.eventTriggerWorkbench.scopeUser") }}</option>
                <option value="bot_only">{{ t("expertModels.eventTriggerWorkbench.scopeBot") }}</option>
              </select>
            </label>
            <label class="em-field">
              <div class="em-label">{{ t("expertModels.eventTriggerWorkbench.keywordLabel") }}</div>
              <input
                class="em-input"
                type="text"
                :value="(selectedNode as any).matchSubstring"
                @input="patchSelectedNode({ matchSubstring: ($event.target as HTMLInputElement).value })"
              />
            </label>
          </div>

          <div class="em-etw-section">
            <div class="em-etw-h">{{ t("expertModels.eventTriggerWorkbench.sectionMemory") }}</div>
            <div class="em-muted em-etw-sub">
              {{ t("expertModels.eventTriggerWorkbench.memoryHint") }}
              <code class="em-etw-code">{{ t("expertModels.eventTriggerWorkbench.placeholderTokens") }}</code>
            </div>
            <label class="em-field">
              <div class="em-label">{{ t("expertModels.cloudEvent.memoryLabel") }}</div>
              <textarea
                class="em-text"
                rows="4"
                :value="(selectedNode as any).memoryContent"
                @input="patchSelectedNode({ memoryContent: ($event.target as HTMLTextAreaElement).value })"
              />
            </label>
            <label class="em-field">
              <div class="em-label">{{ t("expertModels.cloudEvent.importanceLabel") }}</div>
              <input
                class="em-num"
                type="number"
                step="0.05"
                min="0"
                max="1"
                :value="(selectedNode as any).importance"
                @input="patchSelectedNode({ importance: Number(($event.target as HTMLInputElement).value) })"
              />
            </label>
            <label class="em-row" style="margin-top: 8px">
              <input
                type="checkbox"
                :checked="(selectedNode as any).enabled !== false"
                @change="patchSelectedNode({ enabled: ($event.target as HTMLInputElement).checked })"
              />
              <span class="em-muted">{{ t("expertModels.cloudEvent.enabled") }}</span>
            </label>
          </div>

          <div class="em-etw-section em-etw-test">
            <div class="em-etw-h">{{ t("expertModels.eventTriggerWorkbench.sectionTest") }}</div>
            <label class="em-field">
              <div class="em-label">{{ t("expertModels.eventTriggerWorkbench.testUserLabel") }}</div>
              <textarea
                v-model="eventTriggerTestUser"
                class="em-text"
                rows="2"
                :placeholder="String(t('expertModels.eventTriggerWorkbench.testUserPlaceholder'))"
              />
            </label>
            <label class="em-field">
              <div class="em-label">{{ t("expertModels.eventTriggerWorkbench.testBotLabel") }}</div>
              <textarea
                v-model="eventTriggerTestBot"
                class="em-text"
                rows="2"
                :placeholder="String(t('expertModels.eventTriggerWorkbench.testBotPlaceholder'))"
              />
            </label>
            <button type="button" class="em-btn secondary" @click="runEventTriggerWorkbenchTest">
              {{ t("expertModels.eventTriggerWorkbench.testRun") }}
            </button>

            <div v-if="eventTriggerTestResult" class="em-etw-result">
              <template v-if="eventTriggerTestResult.kind === 'ok'">
                <div class="em-etw-ok">{{ t("expertModels.eventTriggerWorkbench.testResultFires") }}</div>
                <div v-if="eventTriggerTestResult.hitUser" class="em-muted">
                  {{ t("expertModels.eventTriggerWorkbench.testHitUser") }}
                </div>
                <div v-if="eventTriggerTestResult.hitBot" class="em-muted">
                  {{ t("expertModels.eventTriggerWorkbench.testHitBot") }}
                </div>
                <div class="em-etw-resolved-h">{{ t("expertModels.eventTriggerWorkbench.testResolved") }}</div>
                <pre class="em-etw-pre">{{ eventTriggerTestResult.resolvedMemory }}</pre>
              </template>
              <template v-else>
                <div class="em-etw-no">{{ t("expertModels.eventTriggerWorkbench.testResultNoFire") }}</div>
                <div class="em-muted">
                  {{
                    eventTriggerTestResult.reason === "disabled"
                      ? t("expertModels.eventTriggerWorkbench.testReasonDisabled")
                      : eventTriggerTestResult.reason === "empty_keyword"
                        ? t("expertModels.eventTriggerWorkbench.testReasonEmptyKeyword")
                        : eventTriggerTestResult.reason === "empty_memory"
                          ? t("expertModels.eventTriggerWorkbench.testReasonEmptyMemory")
                          : t("expertModels.eventTriggerWorkbench.testReasonNoMatch")
                  }}
                </div>
              </template>
            </div>
          </div>
        </template>

        <template v-else-if="(selectedNode as any).type === 'prompt_style'">
          <div class="em-muted" style="margin-top: 0">
            {{ t("expertModels.inspector.promptStyleHint") }}
          </div>
          <label class="em-field" style="margin-top: 8px">
            <div class="em-label">{{ t("expertModels.promptStyle.replyQualityAnchor") }}</div>
            <textarea
              class="em-text"
              rows="3"
              :value="((selectedNode as any).style?.replyQualityAnchor ?? '')"
              @input="patchSelectedPromptStyle({ replyQualityAnchor: ($event.target as HTMLTextAreaElement).value })"
            />
          </label>
          <label class="em-field">
            <div class="em-label">{{ t("expertModels.promptStyle.corePersonality") }}</div>
            <textarea
              class="em-text"
              rows="3"
              :value="((selectedNode as any).style?.corePersonality ?? '')"
              @input="patchSelectedPromptStyle({ corePersonality: ($event.target as HTMLTextAreaElement).value })"
            />
          </label>
          <label class="em-field">
            <div class="em-label">{{ t("expertModels.promptStyle.description") }}</div>
            <textarea
              class="em-text"
              rows="2"
              :value="((selectedNode as any).style?.description ?? '')"
              @input="patchSelectedPromptStyle({ description: ($event.target as HTMLTextAreaElement).value })"
            />
          </label>
        </template>
      </div>
    </div>

    <details v-if="editorMode === 'canvas'" class="em-advanced" open>
      <summary class="em-advanced-sum">{{ t("expertModels.advancedForm.title") }}</summary>
      <div class="em-advanced-body">
        <div class="em-grid">
      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.baseTitle") }}</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportBase">
            {{ t("expertModels.form.importBase") }}
          </button>
        </div>
        <select v-model="selectedBaseModelPath" class="em-select">
          <option value="">{{ t("expertModels.form.keepCurrent") }}</option>
          <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
            {{ m.name }}
          </option>
        </select>
        <div class="em-muted">{{ t("expertModels.form.baseDirHint") }}</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.loraTitle") }}</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportLora">
            {{ t("expertModels.form.importLora") }}
          </button>
        </div>
        <div class="em-lora-add">
          <select class="em-select" @change="addLora(($event.target as HTMLSelectElement).value)">
            <option value="">{{ t("expertModels.form.addLoraPlaceholder") }}</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div v-if="loraNodes.length === 0" class="em-muted">{{ t("expertModels.form.noLora") }}</div>
        <ul v-else class="em-lora-list">
          <li v-for="n in loraNodes" :key="n.id" class="em-lora">
            <label class="em-row">
              <input
                type="checkbox"
                :checked="n.enabled"
                @change="
                  store.draftGraph = {
                    ...store.draftGraph,
                    nodes: store.draftGraph.nodes.map((x) =>
                      x.type === 'lora_adapter' && x.id === n.id
                        ? { ...x, enabled: ($event.target as HTMLInputElement).checked }
                        : x,
                    ),
                  }
                "
              />
              <span class="em-mono">{{ n.ggufPath.split(/[\\/]/).slice(-1)[0] }}</span>
            </label>

            <div class="em-row em-row2">
              <label class="em-muted">
                {{ t("expertModels.form.strengthShort") }}
                <input
                  class="em-num"
                  type="number"
                  step="0.05"
                  :value="n.strength"
                  @input="
                    store.draftGraph = {
                      ...store.draftGraph,
                      nodes: store.draftGraph.nodes.map((x) =>
                        x.type === 'lora_adapter' && x.id === n.id
                          ? { ...x, strength: Number(($event.target as HTMLInputElement).value) }
                          : x,
                      ),
                    }
                  "
                />
              </label>
              <span v-if="strengthWarning(n.strength)" class="em-warn">
                {{ strengthWarning(n.strength) }}
              </span>
            </div>

            <div class="em-lora-actions">
              <button class="em-mini" type="button" @click="moveLora(n.id, -1)">{{ t("expertModels.form.moveUp") }}</button>
              <button class="em-mini" type="button" @click="moveLora(n.id, 1)">{{ t("expertModels.form.moveDown") }}</button>
              <button class="em-mini danger" type="button" @click="removeLora(n.id)">{{ t("expertModels.form.remove") }}</button>
            </div>
          </li>
        </ul>

        <div class="em-muted">{{ t("expertModels.form.loraDirHint") }}</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.effective.title") }}</div>
        <div class="em-muted" style="margin-top: 0">
          {{ t("expertModels.effective.hint") }}
        </div>
        <div class="em-kv">
          <div class="em-k">Base</div>
          <div class="em-v">
            <span v-if="effectiveBasePath" class="em-mono">{{
              effectiveBasePath.split(/[\\/]/).slice(-1)[0]
            }}</span>
            <span v-else class="em-muted">{{ t("expertModels.form.keepCurrent") }}</span>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">LoRA</div>
          <div class="em-v">
            <div v-if="effectiveLoras.length === 0" class="em-muted">{{ t("expertModels.effective.noLoras") }}</div>
            <ul v-else class="em-eff-list">
              <li v-for="n in effectiveLoras" :key="n.id" class="em-eff-li">
                <span class="em-mono">{{ n.ggufPath.split(/[\\/]/).slice(-1)[0] }}</span>
                <span class="em-eff-strength">× {{ n.strength.toFixed(2) }}</span>
              </li>
            </ul>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">PromptStyle</div>
          <div class="em-v">
            <span v-if="store.effectivePromptStyle" class="em-muted">{{ t("expertModels.effective.promptStyleOverridden") }}</span>
            <span v-else class="em-muted">{{ t("expertModels.effective.promptStyleNotOverridden") }}</span>
          </div>
        </div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.promptStyleTitle") }}</div>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.replyQualityAnchorHint") }}</div>
          <textarea
            class="em-text"
            rows="4"
            :value="store.draftPromptStyle?.replyQualityAnchor ?? ''"
            @input="ensurePromptStyle().replyQualityAnchor = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.corePersonalityHint") }}</div>
          <textarea
            class="em-text"
            rows="3"
            :value="store.draftPromptStyle?.corePersonality ?? ''"
            @input="ensurePromptStyle().corePersonality = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.descriptionHint") }}</div>
          <textarea
            class="em-text"
            rows="2"
            :value="store.draftPromptStyle?.description ?? ''"
            @input="ensurePromptStyle().description = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <div class="em-muted">{{ t("expertModels.form.promptStyleFooterHint") }}</div>
      </div>
        </div>
      </div>
    </details>

    <div v-else class="em-grid">
      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.baseTitle") }}</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportBase">
            {{ t("expertModels.form.importBase") }}
          </button>
        </div>
        <select v-model="selectedBaseModelPath" class="em-select">
          <option value="">{{ t("expertModels.form.keepCurrent") }}</option>
          <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
            {{ m.name }}
          </option>
        </select>
        <div class="em-muted">{{ t("expertModels.form.baseDirHint") }}</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.loraTitle") }}</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportLora">
            {{ t("expertModels.form.importLora") }}
          </button>
        </div>
        <div class="em-lora-add">
          <select class="em-select" @change="addLora(($event.target as HTMLSelectElement).value)">
            <option value="">{{ t("expertModels.form.addLoraPlaceholder") }}</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div v-if="loraNodes.length === 0" class="em-muted">{{ t("expertModels.form.noLora") }}</div>
        <ul v-else class="em-lora-list">
          <li v-for="n in loraNodes" :key="n.id" class="em-lora">
            <label class="em-row">
              <input
                type="checkbox"
                :checked="n.enabled"
                @change="
                  store.draftGraph = {
                    ...store.draftGraph,
                    nodes: store.draftGraph.nodes.map((x) =>
                      x.type === 'lora_adapter' && x.id === n.id
                        ? { ...x, enabled: ($event.target as HTMLInputElement).checked }
                        : x,
                    ),
                  }
                "
              />
              <span class="em-mono">{{ n.ggufPath.split(/[\\/]/).slice(-1)[0] }}</span>
            </label>

            <div class="em-row em-row2">
              <label class="em-muted">
                {{ t("expertModels.form.strengthShort") }}
                <input
                  class="em-num"
                  type="number"
                  step="0.05"
                  :value="n.strength"
                  @input="
                    store.draftGraph = {
                      ...store.draftGraph,
                      nodes: store.draftGraph.nodes.map((x) =>
                        x.type === 'lora_adapter' && x.id === n.id
                          ? { ...x, strength: Number(($event.target as HTMLInputElement).value) }
                          : x,
                      ),
                    }
                  "
                />
              </label>
              <span v-if="strengthWarning(n.strength)" class="em-warn">
                {{ strengthWarning(n.strength) }}
              </span>
            </div>

            <div class="em-lora-actions">
              <button class="em-mini" type="button" @click="moveLora(n.id, -1)">{{ t("expertModels.form.moveUp") }}</button>
              <button class="em-mini" type="button" @click="moveLora(n.id, 1)">{{ t("expertModels.form.moveDown") }}</button>
              <button class="em-mini danger" type="button" @click="removeLora(n.id)">{{ t("expertModels.form.remove") }}</button>
            </div>
          </li>
        </ul>

        <div class="em-muted">{{ t("expertModels.form.loraDirHint") }}</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.effective.title") }}</div>
        <div class="em-muted" style="margin-top: 0">
          {{ t("expertModels.effective.hint") }}
        </div>
        <div class="em-kv">
          <div class="em-k">Base</div>
          <div class="em-v">
            <span v-if="effectiveBasePath" class="em-mono">{{
              effectiveBasePath.split(/[\\/]/).slice(-1)[0]
            }}</span>
            <span v-else class="em-muted">{{ t("expertModels.form.keepCurrent") }}</span>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">LoRA</div>
          <div class="em-v">
            <div v-if="effectiveLoras.length === 0" class="em-muted">{{ t("expertModels.effective.noLoras") }}</div>
            <ul v-else class="em-eff-list">
              <li v-for="n in effectiveLoras" :key="n.id" class="em-eff-li">
                <span class="em-mono">{{ n.ggufPath.split(/[\\/]/).slice(-1)[0] }}</span>
                <span class="em-eff-strength">× {{ n.strength.toFixed(2) }}</span>
              </li>
            </ul>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">PromptStyle</div>
          <div class="em-v">
            <span v-if="store.effectivePromptStyle" class="em-muted">{{ t("expertModels.effective.promptStyleOverridden") }}</span>
            <span v-else class="em-muted">{{ t("expertModels.effective.promptStyleNotOverridden") }}</span>
          </div>
        </div>
      </div>

      <div class="em-card">
        <div class="em-card-h">{{ t("expertModels.form.promptStyleTitle") }}</div>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.replyQualityAnchorHint") }}</div>
          <textarea
            class="em-text"
            rows="4"
            :value="store.draftPromptStyle?.replyQualityAnchor ?? ''"
            @input="ensurePromptStyle().replyQualityAnchor = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.corePersonalityHint") }}</div>
          <textarea
            class="em-text"
            rows="3"
            :value="store.draftPromptStyle?.corePersonality ?? ''"
            @input="ensurePromptStyle().corePersonality = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <label class="em-field">
          <div class="em-label">{{ t("expertModels.form.descriptionHint") }}</div>
          <textarea
            class="em-text"
            rows="2"
            :value="store.draftPromptStyle?.description ?? ''"
            @input="ensurePromptStyle().description = ($event.target as HTMLTextAreaElement).value"
            :placeholder="String(t('expertModels.form.emptyMeansNoOverride'))"
          />
        </label>
        <div class="em-muted">{{ t("expertModels.form.promptStyleFooterHint") }}</div>
      </div>
    </div>

    <div class="em-footer">
      <button class="em-btn" type="button" :disabled="applying || store.loading" @click="onApplySession">
        {{ applying ? t("expertModels.footer.applying") : t("expertModels.footer.applyToSession") }}
      </button>
      <button
        class="em-btn secondary"
        type="button"
        :disabled="saving || store.loading || !store.canRollbackLastRun"
        @click="onRollbackLastRun"
        :title="String(t('expertModels.footer.rollbackLastTitle'))"
      >
        {{ t("expertModels.footer.rollbackLast") }}
      </button>
      <details class="em-runs">
        <summary class="em-btn secondary" :aria-disabled="saving || store.loading">{{ t("expertModels.runHistory.ui.title", { n: store.runs.length }) }}</summary>
        <div class="em-runs-body">
          <div v-if="applying" class="em-run-applying">
            <b>{{ t("expertModels.runHistory.ui.applyingTitle") }}</b>
            <span class="em-muted2">{{ t("expertModels.runHistory.ui.applyingHint") }}</span>
          </div>
          <div class="em-runs-actions">
            <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onRefresh">
              {{ t("expertModels.actions.refresh") }}
            </button>
            <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onExportLatestPinnedRun">
              {{ t("expertModels.runHistory.ui.exportPinned") }}
            </button>
            <select v-model="clearMode" class="em-select em-select--runs-wide">
              <option value="all">{{ t("expertModels.runHistory.clearMode.all") }}</option>
              <option value="failed">{{ t("expertModels.runHistory.clearMode.failed") }}</option>
              <option value="ok">{{ t("expertModels.runHistory.clearMode.ok") }}</option>
              <option value="unpinned">{{ t("expertModels.runHistory.clearMode.unpinned") }}</option>
            </select>
            <label class="em-muted2 em-runs-check-row">
              <input v-model="clearKeepPinned" type="checkbox" />
              {{ t("expertModels.runHistory.keepPinned") }}
            </label>
            <button
              class="em-btn secondary"
              type="button"
              :disabled="saving || store.loading || !store.runs.length"
              @click="onClearRunsAdvanced"
            >
              {{ t("expertModels.runHistory.ui.clearExecute") }}
            </button>
            <select v-model="runFilterStatus" class="em-select em-select--runs">
              <option value="all">{{ t("expertModels.runHistory.ui.filterStatus.all") }}</option>
              <option value="ok">OK</option>
              <option value="failed">FAILED</option>
              <option value="unknown">{{ t("expertModels.runHistory.ui.filterStatus.unknown") }}</option>
            </select>
            <input
              v-model="runFilterText"
              class="em-input em-input--runs-search"
              type="text"
              :placeholder="String(t('expertModels.runHistory.ui.searchBasePlaceholder'))"
            />
          </div>
          <div v-if="!store.runs.length" class="em-muted">
            {{ t("expertModels.runHistory.ui.emptyHint") }}
          </div>
          <div v-else class="em-run-list">
            <div v-for="r in filteredRuns" :key="String(r.indexFromLatest)" class="em-run-wrap">
              <div class="em-run-item">
                <div class="em-run-main">
                  <div class="em-run-title">
                    <b>#{{ r.indexFromLatest + 1 }}</b>
                    <span class="em-muted2" :title="new Date(r.atMs).toLocaleString()">
                      {{ formatRelative(r.atMs) }}（{{ formatRunTime(r.atMs) }}）
                    </span>
                    <button
                      class="em-btn secondary em-pin"
                      type="button"
                      :disabled="saving || store.loading"
                      :title="
                        r.pinned
                          ? t('expertModels.runHistory.ui.pinTitle.unpin')
                          : t('expertModels.runHistory.ui.pinTitle.pin')
                      "
                      @click="onTogglePinned(r.indexFromLatest, r.pinned ?? false)"
                    >
                      {{ r.pinned ? "★" : "☆" }}
                    </button>
                  </div>
                  <div class="em-run-meta">
                    <span class="em-pill2">{{ t("expertModels.runHistory.ui.basePill", { name: r.targetBaseName || String(t("expertModels.common.notSet")) }) }}</span>
                    <span class="em-pill2">LoRA：{{ r.targetLoraCount }}</span>
                    <span v-if="r.targetHasPromptStyle" class="em-pill2">PromptStyle</span>
                    <span v-if="r.applyOk === true" class="em-pill2 em-ok">OK</span>
                    <span
                      v-if="r.applyOk === true && r.applySidecarNotice"
                      class="em-pill2 em-warn"
                      :title="r.applySidecarNotice"
                    >
                      {{ t("expertModels.runHistory.ui.sidecarWarnPill") }}
                    </span>
                    <span v-if="r.applyOk === false" class="em-pill2 em-bad" :title="r.applyError || ''">FAILED</span>
                    <span v-if="r.applyDurationMs != null" class="em-pill2">{{ t("expertModels.runHistory.ui.durationPill", { ms: r.applyDurationMs }) }}</span>
                  </div>
                </div>
                <div class="em-run-actions">
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onToggleRunDetail(r.indexFromLatest)">
                    {{
                      expandedRunIndex === r.indexFromLatest
                        ? t("expertModels.runHistory.ui.collapseDetail")
                        : t("expertModels.runHistory.ui.expandDetail")
                    }}
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onRollbackToRun(r.indexFromLatest)">
                    {{ t("expertModels.runHistory.ui.rollbackToHere") }}
                  </button>
                  <button
                    v-if="r.applyOk === false"
                    class="em-btn secondary"
                    type="button"
                    :disabled="saving || store.loading"
                    @click="onRetryRun(r.indexFromLatest)"
                  >
                    {{ t("expertModels.runHistory.ui.retry") }}
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onCopyRunDiagnostics(r.indexFromLatest)">
                    {{ t("expertModels.runHistory.ui.copyDiagnostics") }}
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onSaveRunAsWorkflow(r.indexFromLatest)">
                    {{ t("expertModels.runHistory.ui.saveAsWorkflow") }}
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onExportRunAsWorkflowJson(r.indexFromLatest)">
                    {{ t("expertModels.runHistory.ui.exportWorkflow") }}
                  </button>
                </div>
              </div>
              <div v-if="expandedRunIndex != null && expandedRunIndex === r.indexFromLatest" class="em-run-detail">
                <div v-if="!expandedRunDetail" class="em-muted">{{ t("expertModels.runHistory.ui.loadingDetail") }}</div>
                <div v-else class="em-run-detail-grid">
                  <div>
                    <div class="em-muted">{{ t("expertModels.runHistory.ui.targetTitle") }}</div>
                    <div><b>Base</b>：{{ expandedRunDetail.targetBaseName || String(t("expertModels.common.notSet")) }}</div>
                    <div><b>LoRA</b>：{{ expandedRunDetail.targetLoraCount }}</div>
                    <div><b>PromptStyle</b>：{{ expandedRunDetail.targetHasPromptStyle ? t("expertModels.common.yes") : t("expertModels.common.no") }}</div>
                  </div>
                  <div>
                    <div class="em-muted">{{ t("expertModels.runHistory.ui.snapshotTitle") }}</div>
                    <div><b>Base</b>：{{ expandedRunDetail.snapshotBaseName || String(t("expertModels.common.notSet")) }}</div>
                    <div><b>LoRA</b>：{{ expandedRunDetail.snapshotLoraCount }}</div>
                    <div><b>PromptStyle</b>：{{ expandedRunDetail.snapshotHasPromptStyle ? t("expertModels.common.yes") : t("expertModels.common.no") }}</div>
                  </div>
                  <div style="grid-column: 1 / -1" v-if="expandedRunDetail.applyOk === false">
                    <div class="em-muted">{{ t("expertModels.runHistory.ui.errorTitle") }}</div>
                    <pre class="em-pre">{{ expandedRunDetail.applyError || String(t("expertModels.common.empty")) }}</pre>
                  </div>
                  <div style="grid-column: 1 / -1" v-else-if="expandedRunDetail.applyOk === true">
                    <div class="em-muted">{{ t("expertModels.runHistory.ui.resultTitle") }}</div>
                    <div><b>modelPath</b>：{{ expandedRunDetail.applyModelPath || String(t("expertModels.runHistory.ui.notReturned")) }}</div>
                    <div><b>durationMs</b>：{{ expandedRunDetail.applyDurationMs ?? String(t("expertModels.runHistory.ui.notReturned")) }}</div>
                    <div v-if="expandedRunDetail.applySidecarNotice" class="em-muted" style="margin-top: 8px">
                      {{ t("expertModels.runHistory.ui.sidecarNoticeLabel") }}
                    </div>
                    <pre v-if="expandedRunDetail.applySidecarNotice" class="em-pre">{{ expandedRunDetail.applySidecarNotice }}</pre>
                    <details>
                      <summary class="em-muted2">{{ t("expertModels.runHistory.ui.llamaArgsExpand") }}</summary>
                      <pre class="em-pre">{{ expandedRunDetail.applyLlamaArgs || "" }}</pre>
                    </details>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </details>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onSetRoleDefault">
        {{ t("expertModels.footer.setRoleDefault") }}
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearSessionOverride">
        {{ t("expertModels.footer.clearSessionOverride") }}
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearRoleDefault">
        {{ t("expertModels.footer.clearRoleDefault") }}
      </button>
    </div>

    <OclexpertPublishWizard
      v-model="publishWizardOpen"
      :embedded="props.embedded"
      :graph="store.draftGraph"
      :prompt-style="store.draftPromptStyle ?? null"
      :initial-name="workflowNameDraft"
      :initial-description="oclexpertDescriptionDraft"
      :initial-author="oclexpertAuthorDraft"
      :last-export-path="lastExportedOclexpertPath"
      @sync-drafts="onPublishWizardSyncDrafts"
    />

    <Teleport to="body" :disabled="props.embedded">
      <div
        v-if="oclexpertImportPreview"
        class="em-oclexpert-backdrop"
        :class="{ 'em-oclexpert-backdrop--inplace': props.embedded }"
        role="dialog"
        aria-modal="true"
        @click.self="cancelOclexpertImportPreview"
      >
        <div class="em-oclexpert-modal" @click.stop>
          <div class="em-card-h">{{ t("expertModels.oclexpert.previewTitle") }}</div>
          <dl class="em-oclexpert-dl">
            <dt>{{ t("expertModels.oclexpert.previewName") }}</dt>
            <dd>{{ oclexpertImportPreview.suggestedName || "—" }}</dd>
            <dt>{{ t("expertModels.oclexpert.previewDescription") }}</dt>
            <dd>{{ oclexpertImportPreview.suggestedDescription || "—" }}</dd>
            <dt>{{ t("expertModels.oclexpert.previewAuthor") }}</dt>
            <dd>{{ oclexpertImportPreview.suggestedAuthor || "—" }}</dd>
            <dt>{{ t("expertModels.oclexpert.previewGraphSummary") }}</dt>
            <dd>{{ summarizeExpertGraphNodes(oclexpertImportPreview.graph) }}</dd>
            <dt>{{ t("expertModels.oclexpert.previewPrivacy") }}</dt>
            <dd>{{ oclexpertImportPrivacySummary(oclexpertImportPreview.graph) }}</dd>
          </dl>
          <div class="em-oclexpert-actions">
            <button class="em-btn secondary" type="button" :disabled="saving" @click="cancelOclexpertImportPreview">
              {{ t("expertModels.oclexpert.previewCancel") }}
            </button>
            <button class="em-btn" type="button" :disabled="saving" @click="confirmOclexpertImportPreview">
              {{ t("expertModels.oclexpert.previewConfirm") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.em-root {
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.em-root--embedded {
  position: relative;
  isolation: isolate;
  max-height: min(78vh, 900px);
  overflow-x: hidden;
  overflow-y: auto;
  box-sizing: border-box;
}
.em-h {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px 16px;
  flex-wrap: wrap;
}
.em-h > div:first-child {
  flex: 1 1 220px;
  min-width: 0;
}
.em-title {
  margin: 0 0 6px;
  font-size: 16px;
}
.em-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.em-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  flex: 0 1 auto;
}
.em-workflows {
  margin-top: 10px;
}
.em-wf-form {
  display: grid;
  grid-template-columns: minmax(100px, 32%) minmax(0, 1fr);
  gap: 10px 14px;
  align-items: start;
  margin-top: 4px;
}
.em-wf-label {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  line-height: 1.35;
  padding-top: 8px;
}
.em-wf-inline {
  display: flex;
  align-items: stretch;
  gap: 8px;
  min-width: 0;
}
.em-wf-inline-grow {
  flex: 1 1 auto;
  min-width: 0;
}
.em-wf-inline-btn {
  flex: 0 0 auto;
  white-space: nowrap;
}
.em-wf-hint {
  margin: 10px 0 0;
  line-height: 1.45;
}
.em-block-spaced {
  margin: 0 0 10px;
  line-height: 1.45;
}
.em-wf-actions {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(136px, 1fr));
  gap: 8px;
  margin-top: 12px;
}
.em-wf-actions.em-wf-actions--compact {
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
.em-wf-actions .em-btn {
  width: 100%;
  text-align: center;
}
.em-wf-market {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--border-light);
}
.em-wf-market-hint {
  flex: 1 1 200px;
  font-size: 12px;
}
.em-wf-form .em-input,
.em-wf-form .em-text {
  width: 100%;
  box-sizing: border-box;
}
.em-input {
  flex: 1 1 auto;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.em-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 13px;
}
.em-btn.secondary {
  background: transparent;
}
.em-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
  align-items: center;
}
.em-editorbar {
  margin-top: 12px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 10px 14px;
  align-items: center;
}
.em-editorbar .em-muted {
  margin: 0;
  line-height: 1.45;
  min-width: 0;
}
.em-cloud-event-wrap {
  margin-top: 10px;
}
.em-canvaswrap {
  margin-top: 10px;
}
.em-inspector {
  margin-top: 10px;
}
.em-advanced {
  margin-top: 10px;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-elevated);
  overflow: hidden;
}
.em-advanced-sum {
  cursor: pointer;
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 700;
  list-style: none;
}
.em-advanced-body {
  padding: 0 12px 12px;
}
.em-pill {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  color: var(--text-secondary);
}
.em-warnbar {
  flex: 1 1 520px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--danger-600, #c0392b) 30%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 10%, var(--bg-elevated));
  color: var(--text-primary);
  font-size: 12px;
}
.em-muted2 {
  margin-left: 6px;
  color: var(--text-secondary);
}
.em-btn.danger {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
}
.em-err {
  color: var(--danger-600, #c0392b);
  font-size: 12px;
}
.em-grid {
  margin-top: 10px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
  gap: 12px;
}
.em-card {
  min-width: 0;
  padding: 10px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.em-card-h {
  font-size: 13px;
  font-weight: 700;
  margin-bottom: 8px;
}
.em-row3 {
  display: flex;
  justify-content: flex-end;
  margin: -2px 0 8px;
}
.em-kv {
  display: grid;
  grid-template-columns: 70px minmax(0, 1fr);
  gap: 8px;
  align-items: start;
  margin-top: 10px;
}
.em-k {
  color: var(--text-secondary);
  font-size: 12px;
}
.em-v {
  min-width: 0;
  font-size: 12px;
}
.em-eff-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.em-eff-li {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}
.em-eff-strength {
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}
.em-mini {
  margin-left: 6px;
  padding: 3px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
}
.em-mini.on {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-weight: 700;
}
.em-select {
  width: 100%;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.em-muted {
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}
.em-lora-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.em-lora {
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 8px;
  background: var(--bg-primary);
}
.em-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.em-row2 {
  margin-top: 6px;
  align-items: baseline;
  flex-wrap: wrap;
}
.em-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.em-num {
  width: 120px;
  margin-left: 6px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.em-warn {
  color: color-mix(in srgb, #f59e0b 75%, var(--text-primary));
  font-size: 12px;
}
.em-lora-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}
.em-mini {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
}
.em-mini.danger {
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
  color: var(--danger-600, #c0392b);
}
.em-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 10px;
}
.em-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.em-text {
  width: 100%;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.45;
  resize: vertical;
}
.em-footer {
  margin-top: 14px;
  display: flex;
  flex-wrap: wrap;
  gap: 10px 12px;
  align-items: center;
}
.em-runs {
  display: inline-block;
}
.em-runs > summary {
  list-style: none;
}
.em-runs > summary::-webkit-details-marker {
  display: none;
}
.em-runs-body {
  margin-top: 8px;
  padding: 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  width: 100%;
  max-width: min(720px, 100%);
  min-width: 0;
  box-sizing: border-box;
}
.em-runs-actions {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
  margin-bottom: 10px;
  align-items: center;
}
.em-runs-actions .em-select {
  min-width: 0;
}
.em-runs-check-row {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.em-input--runs-search {
  min-width: 0;
  width: 100%;
}
.em-run-applying {
  display: flex;
  gap: 8px;
  align-items: baseline;
  padding: 8px;
  border-radius: 10px;
  border: 1px solid rgba(53, 124, 255, 0.35);
  background: rgba(53, 124, 255, 0.08);
  margin-bottom: 8px;
}
.em-run-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.em-run-wrap {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.em-run-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.em-run-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.em-pin {
  padding: 0 8px;
  height: 22px;
}
.em-run-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}
.em-run-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}
.em-run-detail {
  padding: 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.em-run-detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.em-pre {
  margin: 6px 0 0;
  padding: 8px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 12px;
  line-height: 1.45;
  overflow: auto;
  white-space: pre-wrap;
}
.em-pill2 {
  display: inline-flex;
  align-items: center;
  height: 20px;
  padding: 0 8px;
  border-radius: 999px;
  font-size: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-secondary);
}
.em-pill2.em-ok {
  border-color: rgba(46, 160, 67, 0.45);
  color: rgba(46, 160, 67, 0.95);
}
.em-pill2.em-bad {
  border-color: rgba(248, 81, 73, 0.5);
  color: rgba(248, 81, 73, 0.95);
}
.em-pill2.em-warn {
  border-color: rgba(210, 153, 34, 0.55);
  color: rgba(210, 153, 34, 0.98);
}
@media (max-width: 1080px) {
  .em-grid {
    grid-template-columns: 1fr;
  }
}
.em-integrity {
  margin-top: 12px;
  border-color: color-mix(in srgb, #f59e0b 35%, var(--border-light));
}
.em-empty-graph {
  margin-top: 12px;
}
.em-oclexpert-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: color-mix(in srgb, #000 48%, transparent);
}
.em-oclexpert-backdrop--inplace {
  position: absolute;
  border-radius: inherit;
}
.em-oclexpert-modal {
  width: min(440px, 100%);
  padding: 16px 18px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.em-oclexpert-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}
.em-oclexpert-dl {
  margin: 10px 0 0;
  display: grid;
  grid-template-columns: 88px 1fr;
  gap: 6px 10px;
  font-size: 13px;
}
.em-oclexpert-dl dt {
  margin: 0;
  color: var(--text-secondary);
  font-weight: 600;
}
.em-oclexpert-dl dd {
  margin: 0;
  word-break: break-word;
}
@media (max-width: 560px) {
  .em-wf-form {
    grid-template-columns: 1fr;
  }
  .em-wf-label {
    padding-top: 0;
  }
  .em-editorbar {
    grid-template-columns: 1fr;
  }
}
.em-etw-section {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--border-light);
}
.em-etw-h {
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 6px;
}
.em-etw-sub {
  font-size: 12px;
  margin-bottom: 6px;
  line-height: 1.45;
}
.em-etw-code {
  font-size: 11px;
  padding: 1px 4px;
  border-radius: 4px;
  background: var(--bg-secondary);
}
.em-etw-test {
  padding-bottom: 4px;
}
.em-etw-result {
  margin-top: 10px;
  padding: 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.em-etw-ok {
  font-weight: 600;
  color: var(--success, #2e7d32);
  margin-bottom: 6px;
}
.em-etw-no {
  font-weight: 600;
  margin-bottom: 6px;
}
.em-etw-resolved-h {
  margin-top: 8px;
  font-weight: 600;
  font-size: 12px;
}
.em-etw-pre {
  margin: 6px 0 0;
  padding: 8px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
  max-height: 160px;
  overflow: auto;
}
</style>

