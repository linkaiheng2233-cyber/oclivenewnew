<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open, save } from "@tauri-apps/api/dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { useAppToast } from "../../composables/useAppToast";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import ExpertModelsCanvas from "./ExpertModelsCanvas.vue";
import type { ExpertGraph, ExpertNode, PromptStyleOverride } from "../../utils/tauri-api";

const store = useExpertModelsStore();
const { showToast } = useAppToast();
const emit = defineEmits<{
  (e: "open-permissions", payload: { pluginId: string }): void;
}>();

const saving = ref(false);
const applying = ref(false);
const editorMode = ref<"canvas" | "form">("canvas");
const selectedCanvasNodeId = ref<string | null>(null);
const runFilterStatus = ref<"all" | "ok" | "failed" | "unknown">("all");
const runFilterText = ref("");
const expandedRunIndex = ref<number | null>(null);
const expandedRunDetail = ref<any | null>(null);

const sourceLabel = (s: string): string => {
  if (s === "session_override") return "会话覆盖";
  if (s === "role_default") return "角色默认";
  return "角色包默认";
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
  if (!Number.isFinite(v)) return "强度必须是数字。";
  if (v < 0) return "强度 < 0 通常不合理。";
  if (v > 2) return "强度 > 2 可能导致输出劣化或不稳定。";
  if (v > 1.4) return "强度偏高，建议先从 1.0–1.4 试起。";
  return null;
};

async function onRefresh(): Promise<void> {
  await store.refresh();
  await store.refreshWorkflows().catch(() => {});
  if (store.error) showToast("error", store.error);
}

async function onApplySession(): Promise<void> {
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.applyToSession();
    showToast(
      "success",
      `已应用到当前会话（将触发本地 llama 重启）。\nmodelPath=${r.modelPath ?? "(未设置)"}\nllamaArgs=${r.llamaArgs ?? "(空)"}`,
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
    applying.value = false;
  }
}

async function onRollbackLastRun(): Promise<void> {
  const ok = window.confirm(
    "将回滚到上一次已应用的配置（Module 9 Ctrl+Z），并重新应用到当前会话。\n提示：可在「Run 历史」里回滚到任意一次。\n继续吗？",
  );
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.rollbackLastRun();
    showToast(
      "success",
      `已回滚并重新应用。\nmodelPath=${r.modelPath ?? "(未设置)"}\nllamaArgs=${r.llamaArgs ?? "(空)"}`,
    );
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
  if (d < 1000) return "刚刚";
  const s = Math.floor(d / 1000);
  if (s < 60) return `${s}s 前`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m 前`;
  const h = Math.floor(m / 60);
  if (h < 48) return `${h}h 前`;
  const day = Math.floor(h / 24);
  return `${day}d 前`;
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
  if (!tg) throw new Error("该 Run 没有保存 targetGraph（可能是旧版本记录），无法重试。");
  const ok = window.confirm(
    `将重试此目标配置并重新应用到当前会话：\nBase=${d.targetBaseName || "(未设置)"} / LoRA=${d.targetLoraCount} / PromptStyle=${d.targetHasPromptStyle ? "是" : "否"}\n继续吗？`,
  );
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.applySpecificToSession(tg, ts);
    showToast(
      "success",
      `已重试并应用。\nmodelPath=${r.modelPath ?? "(未设置)"}\nllamaArgs=${r.llamaArgs ?? "(空)"}`,
    );
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
    },
  };
  await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
  showToast("success", "已复制 Run 诊断信息。");
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
    if (!tg) throw new Error("该 Run 没有保存 targetGraph（可能是旧版本记录），无法保存为工作流。");
    const name = window.prompt("保存为工作流：请输入名称", suggestWorkflowNameFromRun(d))?.trim() ?? "";
    if (!name) return;
    const wf = await store.saveWorkflowFromConfig(name, tg, ts, null);
    workflowNameDraft.value = wf.name;
    showToast("success", `已保存到工作流库：${wf.name}`);
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
    if (!tg) throw new Error("该 Run 没有保存 targetGraph（可能是旧版本记录），无法导出工作流文件。");
    const payload = {
      version: 1,
      name: suggestWorkflowNameFromRun(d),
      graph: tg,
      promptStyle: ts ?? null,
    };
    const ok = window.confirm(
      `将导出工作流文件（可分享给他人导入复现）：\n` +
        `Base=${d.targetBaseName || "(未设置)"} / LoRA=${d.targetLoraCount} / PromptStyle=${d.targetHasPromptStyle ? "是" : "否"}\n` +
        `文件名：${payload.name}.oclive-workflow.json\n继续吗？`,
    );
    if (!ok) return;
    const content = JSON.stringify(payload, null, 2);
    const path = await save({
      defaultPath: `${payload.name}.oclive-workflow.json`,
      filters: [{ name: "Workflow JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await writeTextFile(path, content);
    showToast("success", "已导出工作流文件，可分享给其他人导入。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onExportLatestPinnedRun(): Promise<void> {
  const pinned = (store.runs ?? []).find((r) => r.pinned === true);
  if (!pinned) {
    showToast("info", "暂无星标 Run（★）。请先给某条 Run 点星标。");
    return;
  }
  await onExportRunAsWorkflowJson(pinned.indexFromLatest);
}

async function onRollbackToRun(indexFromLatest: number): Promise<void> {
  let summary = "";
  try {
    const d = await store.getRunDetail(indexFromLatest);
    summary = `\n将回滚到：Base=${d.snapshotBaseName || "(未设置)"} / LoRA=${d.snapshotLoraCount} / PromptStyle=${d.snapshotHasPromptStyle ? "是" : "否"}`;
  } catch {
    // ignore
  }
  const ok = window.confirm(`将回滚到选中的历史配置，并重新应用到当前会话。${summary}\n继续吗？`);
  if (!ok) return;
  saving.value = true;
  applying.value = true;
  try {
    const r = await store.rollbackToRun(indexFromLatest);
    showToast(
      "success",
      `已回滚并重新应用。\nmodelPath=${r.modelPath ?? "(未设置)"}\nllamaArgs=${r.llamaArgs ?? "(空)"}`,
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
    applying.value = false;
  }
}

async function onClearRuns(): Promise<void> {
  const ok = window.confirm("将清空当前会话的 Run 历史（全部）。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRuns();
    showToast("success", "已清空 Run 历史。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

const clearMode = ref<"all" | "ok" | "failed" | "unpinned">("all");
const clearKeepPinned = ref(true);

async function onClearRunsAdvanced(): Promise<void> {
  const modeLabel =
    clearMode.value === "ok"
      ? "仅清空 OK"
      : clearMode.value === "failed"
        ? "仅清空 FAILED"
        : clearMode.value === "unpinned"
          ? "仅清空未星标"
          : "清空全部";
  const ok = window.confirm(`${modeLabel}。${clearKeepPinned.value ? "将保留星标条目。" : ""}\n继续吗？`);
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRunsWithMode(clearMode.value, clearKeepPinned.value);
    showToast("success", "已执行清空操作。");
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
    title: "选择一个 Base GGUF（将复制到 models/gguf）",
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importBaseGguf(p);
    showToast("success", "已导入 Base 模型到 models/gguf。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onImportLora(): Promise<void> {
  const picked = await open({
    title: "选择一个 LoRA GGUF（将复制到 models/loras）",
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importLoraGguf(p);
    showToast("success", "已导入 LoRA 到 models/loras。");
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
    showToast("success", "已设置为角色默认。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearSessionOverride(): Promise<void> {
  const ok = window.confirm("将清除当前会话的 Expert Models 覆盖，并回退到角色默认/角色包默认。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearSessionOverrideAndApply();
    showToast("success", "已清除会话覆盖并重新应用。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearRoleDefault(): Promise<void> {
  const ok = window.confirm("将清除该角色的 Expert Models 默认配置（不会影响角色包原文件）。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRoleDefault();
    showToast("success", "已清除角色默认。");
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
    const wf = await store.saveWorkflow(name || "未命名工作流", null);
    workflowNameDraft.value = wf.name;
    showToast("success", `已保存工作流：${wf.name}`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onOverwriteWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", "请先选择一个工作流再覆盖保存。");
    return;
  }
  const ok = window.confirm("将覆盖保存当前选中的工作流。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    const name = workflowNameDraft.value.trim() || store.workflows.find((w) => w.id === wid)?.name || "工作流";
    const wf = await store.saveWorkflow(name, wid);
    workflowNameDraft.value = wf.name;
    showToast("success", `已覆盖保存：${wf.name}`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onLoadWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", "请先选择一个工作流。");
    return;
  }
  saving.value = true;
  try {
    const wf = await store.loadWorkflow(wid);
    workflowNameDraft.value = wf.name;
    showToast("success", `已载入工作流：${wf.name}`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onDeleteWorkflow(): Promise<void> {
  const wid = store.pickedWorkflowId.trim();
  if (!wid) {
    showToast("info", "请先选择一个工作流。");
    return;
  }
  const name = store.workflows.find((w) => w.id === wid)?.name ?? wid;
  const ok = window.confirm(`将删除工作流：${name}\n\n继续吗？`);
  if (!ok) return;
  saving.value = true;
  try {
    await store.deleteWorkflow(wid);
    showToast("success", "已删除工作流。");
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
  showToast("success", "已导出工作流文件。");
}

async function onImportWorkflowJson(): Promise<void> {
  const picked = await open({
    title: "导入工作流（JSON）",
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
    const name = String(v?.name ?? "导入工作流").trim() || "导入工作流";
    store.draftGraph = (v?.graph ?? { version: 1, nodes: [], edges: [] }) as ExpertGraph;
    store.draftPromptStyle = (v?.promptStyle ?? null) as any;
    // save into library
    const wf = await store.saveWorkflow(name, null);
    workflowNameDraft.value = wf.name;
    showToast("success", `已导入并保存到工作流库：${wf.name}`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <section class="em-root" aria-label="Expert Models（Module 9）">
    <header class="em-h">
      <div>
        <h3 class="em-title">Expert Models（Module 9）</h3>
        <p class="em-sub">
          选择 Base GGUF + LoRA 强度，并可选覆盖 PromptStyle。会话覆盖优先于角色默认；不设置时不改变现有行为。
        </p>
      </div>
      <div class="em-actions">
        <button class="em-btn secondary" type="button" :disabled="store.loading || saving" @click="onRefresh">
          刷新
        </button>
        <button class="em-btn" type="button" :disabled="store.loading || saving" @click="store.setDraftFromEffective">
          从有效配置回填编辑器
        </button>
      </div>
    </header>

    <div class="em-meta">
      <div class="em-pill">
        Graph 来源：<b>{{ sourceLabel(store.graphSource) }}</b>
      </div>
      <div class="em-pill">
        PromptStyle 来源：<b>{{ sourceLabel(store.promptStyleSource) }}</b>
      </div>
      <div v-if="store.llamaMissingMechanismPerms.length" class="em-warnbar">
        <div>
          <b>本地 Llama 尚未授权必要权限</b>
          <span class="em-muted2">
            缺少：{{ store.llamaMissingMechanismPerms.join("、") }}。未授权时会回退其他 LLM 或调用被拦截。
          </span>
        </div>
        <button
          type="button"
          class="em-btn danger"
          @click="emit('open-permissions', { pluginId: 'com.oclive.llama.local' })"
        >
          去授权
        </button>
      </div>
      <div v-if="store.error" class="em-err">{{ store.error }}</div>
    </div>

    <div class="em-workflows">
      <div class="em-card">
        <div class="em-card-h">工作流（第九模块配置包）</div>
        <div class="em-wf-row">
          <label class="em-muted" style="min-width: 72px">名称</label>
          <input v-model="workflowNameDraft" class="em-input" type="text" placeholder="给工作流起个名字…" />
        </div>
        <div class="em-wf-row">
          <label class="em-muted" style="min-width: 72px">库</label>
          <select v-model="store.pickedWorkflowId" class="em-select" style="flex: 1 1 auto">
            <option value="">（未选择）</option>
            <option v-for="w in store.workflows" :key="w.id" :value="w.id">
              {{ w.name }}
            </option>
          </select>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onLoadWorkflow">载入</button>
        </div>
        <div class="em-wf-actions">
          <button class="em-btn" type="button" :disabled="saving" @click="onSaveWorkflowAs">保存为新工作流</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onOverwriteWorkflow">覆盖保存</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onDeleteWorkflow">删除</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onExportWorkflowJson">导出文件</button>
          <button class="em-btn secondary" type="button" :disabled="saving" @click="onImportWorkflowJson">导入文件</button>
        </div>
        <div class="em-muted">
          提示：工作流会保存节点排布、连线与参数；可导出分享给其他创作者。\n
        </div>
      </div>
    </div>

    <div class="em-editorbar">
      <div class="em-pill">
        编辑器：
        <button
          type="button"
          class="em-mini"
          :class="{ on: editorMode === 'canvas' }"
          @click="editorMode = 'canvas'"
        >
          画布（连线）
        </button>
        <button
          type="button"
          class="em-mini"
          :class="{ on: editorMode === 'form' }"
          @click="editorMode = 'form'"
        >
          表单
        </button>
      </div>
      <div class="em-muted">提示：画布会把节点位置与连线写入 ExpertGraph（用于 M2 编译）。</div>
    </div>

    <div v-if="editorMode === 'canvas'" class="em-canvaswrap">
      <ExpertModelsCanvas v-model="store.draftGraph" v-model:selectedNodeId="selectedCanvasNodeId" />
    </div>

    <div v-if="editorMode === 'canvas' && selectedNode" class="em-inspector">
      <div class="em-card">
        <div class="em-card-h">节点属性：{{ (selectedNode as any).type }} · {{ (selectedNode as any).id }}</div>

        <template v-if="(selectedNode as any).type === 'base_model'">
          <select
            class="em-select"
            :value="(selectedNode as any).ggufPath"
            @change="patchSelectedNode({ ggufPath: ($event.target as HTMLSelectElement).value })"
          >
            <option value="">（不设置 / 保持当前）</option>
            <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
          <div class="em-muted">Base 只允许选择 `models/gguf/` 下的 GGUF。</div>
        </template>

        <template v-else-if="(selectedNode as any).type === 'lora_adapter'">
          <select
            class="em-select"
            :value="(selectedNode as any).ggufPath"
            @change="patchSelectedNode({ ggufPath: ($event.target as HTMLSelectElement).value })"
          >
            <option value="">（选择一个 LoRA…）</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>

          <label class="em-field" style="margin-top: 8px">
            <div class="em-label">强度（ComfyUI 风格，默认 1.0）</div>
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
            <span class="em-muted">启用该 LoRA</span>
          </label>
        </template>

        <template v-else-if="(selectedNode as any).type === 'prompt_style'">
          <div class="em-muted" style="margin-top: 0">
            提示：这里编辑的内容会同步到“PromptStyle（可选覆盖）”的草稿，并在应用时作为覆盖层生效。
          </div>
          <label class="em-field" style="margin-top: 8px">
            <div class="em-label">回复质量锚点</div>
            <textarea
              class="em-text"
              rows="3"
              :value="((selectedNode as any).style?.replyQualityAnchor ?? '')"
              @input="patchSelectedPromptStyle({ replyQualityAnchor: ($event.target as HTMLTextAreaElement).value })"
            />
          </label>
          <label class="em-field">
            <div class="em-label">核心人设</div>
            <textarea
              class="em-text"
              rows="3"
              :value="((selectedNode as any).style?.corePersonality ?? '')"
              @input="patchSelectedPromptStyle({ corePersonality: ($event.target as HTMLTextAreaElement).value })"
            />
          </label>
          <label class="em-field">
            <div class="em-label">描述</div>
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
      <summary class="em-advanced-sum">高级/兼容编辑（表单）</summary>
      <div class="em-advanced-body">
        <div class="em-grid">
      <div class="em-card">
        <div class="em-card-h">Base 模型（GGUF）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportBase">
            导入 GGUF…
          </button>
        </div>
        <select v-model="selectedBaseModelPath" class="em-select">
          <option value="">（不设置 / 保持当前）</option>
          <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
            {{ m.name }}
          </option>
        </select>
        <div class="em-muted">目录：`{app_data}/models/gguf/*.gguf`</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">LoRA（可多选）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportLora">
            导入 LoRA…
          </button>
        </div>
        <div class="em-lora-add">
          <select class="em-select" @change="addLora(($event.target as HTMLSelectElement).value)">
            <option value="">添加一个 LoRA…</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div v-if="loraNodes.length === 0" class="em-muted">尚未添加 LoRA。</div>
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
                强度
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
              <button class="em-mini" type="button" @click="moveLora(n.id, -1)">上移</button>
              <button class="em-mini" type="button" @click="moveLora(n.id, 1)">下移</button>
              <button class="em-mini danger" type="button" @click="removeLora(n.id)">移除</button>
            </div>
          </li>
        </ul>

        <div class="em-muted">目录：`{app_data}/models/loras/*.gguf`（也兼容放在 gguf 目录）</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">当前生效（用于排错）</div>
        <div class="em-muted" style="margin-top: 0">
          该块展示的是“当前生效配置”（会话覆盖 / 角色默认 / 角色包默认），不等同于你正在编辑的草稿。
        </div>
        <div class="em-kv">
          <div class="em-k">Base</div>
          <div class="em-v">
            <span v-if="effectiveBasePath" class="em-mono">{{
              effectiveBasePath.split(/[\\/]/).slice(-1)[0]
            }}</span>
            <span v-else class="em-muted">（未设置 / 保持当前）</span>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">LoRA</div>
          <div class="em-v">
            <div v-if="effectiveLoras.length === 0" class="em-muted">（无 / 未启用）</div>
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
            <span v-if="store.effectivePromptStyle" class="em-muted">（已覆盖）</span>
            <span v-else class="em-muted">（未覆盖）</span>
          </div>
        </div>
      </div>

      <div class="em-card">
        <div class="em-card-h">PromptStyle（可选覆盖）</div>
        <label class="em-field">
          <div class="em-label">回复质量锚点（覆盖角色包/默认）</div>
          <textarea
            class="em-text"
            rows="4"
            :value="store.draftPromptStyle?.replyQualityAnchor ?? ''"
            @input="ensurePromptStyle().replyQualityAnchor = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">核心人设（覆盖 role.core_personality）</div>
          <textarea
            class="em-text"
            rows="3"
            :value="store.draftPromptStyle?.corePersonality ?? ''"
            @input="ensurePromptStyle().corePersonality = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">描述（覆盖 role.description）</div>
          <textarea
            class="em-text"
            rows="2"
            :value="store.draftPromptStyle?.description ?? ''"
            @input="ensurePromptStyle().description = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <div class="em-muted">提示：未设置时，Prompt 行为与当前版本完全一致。</div>
      </div>
        </div>
      </div>
    </details>

    <div v-else class="em-grid">
      <div class="em-card">
        <div class="em-card-h">Base 模型（GGUF）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportBase">
            导入 GGUF…
          </button>
        </div>
        <select v-model="selectedBaseModelPath" class="em-select">
          <option value="">（不设置 / 保持当前）</option>
          <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
            {{ m.name }}
          </option>
        </select>
        <div class="em-muted">目录：`{app_data}/models/gguf/*.gguf`</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">LoRA（可多选）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportLora">
            导入 LoRA…
          </button>
        </div>
        <div class="em-lora-add">
          <select class="em-select" @change="addLora(($event.target as HTMLSelectElement).value)">
            <option value="">添加一个 LoRA…</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div v-if="loraNodes.length === 0" class="em-muted">尚未添加 LoRA。</div>
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
                强度
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
              <button class="em-mini" type="button" @click="moveLora(n.id, -1)">上移</button>
              <button class="em-mini" type="button" @click="moveLora(n.id, 1)">下移</button>
              <button class="em-mini danger" type="button" @click="removeLora(n.id)">移除</button>
            </div>
          </li>
        </ul>

        <div class="em-muted">目录：`{app_data}/models/loras/*.gguf`（也兼容放在 gguf 目录）</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">当前生效（用于排错）</div>
        <div class="em-muted" style="margin-top: 0">
          该块展示的是“当前生效配置”（会话覆盖 / 角色默认 / 角色包默认），不等同于你正在编辑的草稿。
        </div>
        <div class="em-kv">
          <div class="em-k">Base</div>
          <div class="em-v">
            <span v-if="effectiveBasePath" class="em-mono">{{
              effectiveBasePath.split(/[\\/]/).slice(-1)[0]
            }}</span>
            <span v-else class="em-muted">（未设置 / 保持当前）</span>
          </div>
        </div>
        <div class="em-kv">
          <div class="em-k">LoRA</div>
          <div class="em-v">
            <div v-if="effectiveLoras.length === 0" class="em-muted">（无 / 未启用）</div>
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
            <span v-if="store.effectivePromptStyle" class="em-muted">（已覆盖）</span>
            <span v-else class="em-muted">（未覆盖）</span>
          </div>
        </div>
      </div>

      <div class="em-card">
        <div class="em-card-h">PromptStyle（可选覆盖）</div>
        <label class="em-field">
          <div class="em-label">回复质量锚点（覆盖角色包/默认）</div>
          <textarea
            class="em-text"
            rows="4"
            :value="store.draftPromptStyle?.replyQualityAnchor ?? ''"
            @input="ensurePromptStyle().replyQualityAnchor = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">核心人设（覆盖 role.core_personality）</div>
          <textarea
            class="em-text"
            rows="3"
            :value="store.draftPromptStyle?.corePersonality ?? ''"
            @input="ensurePromptStyle().corePersonality = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">描述（覆盖 role.description）</div>
          <textarea
            class="em-text"
            rows="2"
            :value="store.draftPromptStyle?.description ?? ''"
            @input="ensurePromptStyle().description = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <div class="em-muted">提示：未设置时，Prompt 行为与当前版本完全一致。</div>
      </div>
    </div>

    <div class="em-footer">
      <button class="em-btn" type="button" :disabled="saving || store.loading" @click="onApplySession">
        {{ saving ? "应用中…" : "应用到当前会话（重启本地 llama）" }}
      </button>
      <button
        class="em-btn secondary"
        type="button"
        :disabled="saving || store.loading || !store.canRollbackLastRun"
        @click="onRollbackLastRun"
        title="回滚到上一次已应用的配置（仅当前会话）"
      >
        回滚上一次 Run
      </button>
      <details class="em-runs">
        <summary class="em-btn secondary" :aria-disabled="saving || store.loading">Run 历史（{{ store.runs.length }}）</summary>
        <div class="em-runs-body">
          <div v-if="applying" class="em-run-applying">
            <b>正在应用…</b>
            <span class="em-muted2">将触发本地 llama 重启；请稍等。</span>
          </div>
          <div class="em-runs-actions">
            <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onRefresh">
              刷新
            </button>
            <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onExportLatestPinnedRun">
              一键导出★
            </button>
            <select v-model="clearMode" class="em-select" style="min-width: 140px">
              <option value="all">清空全部</option>
              <option value="failed">仅清空 FAILED</option>
              <option value="ok">仅清空 OK</option>
              <option value="unpinned">仅清空未星标</option>
            </select>
            <label class="em-muted2" style="display: inline-flex; align-items: center; gap: 6px">
              <input v-model="clearKeepPinned" type="checkbox" />
              保留星标
            </label>
            <button
              class="em-btn secondary"
              type="button"
              :disabled="saving || store.loading || !store.runs.length"
              @click="onClearRunsAdvanced"
            >
              执行清空
            </button>
            <select v-model="runFilterStatus" class="em-select" style="min-width: 120px">
              <option value="all">全部</option>
              <option value="ok">OK</option>
              <option value="failed">FAILED</option>
              <option value="unknown">未知</option>
            </select>
            <input
              v-model="runFilterText"
              class="em-input"
              type="text"
              placeholder="搜索 Base 文件名…"
              style="min-width: 180px"
            />
          </div>
          <div v-if="!store.runs.length" class="em-muted">
            暂无 Run 历史。每次“应用到当前会话”前都会记录一条快照。
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
                      :title="r.pinned ? '取消星标（允许被裁剪/清空）' : '星标（优先保留）'"
                      @click="onTogglePinned(r.indexFromLatest, r.pinned ?? false)"
                    >
                      {{ r.pinned ? "★" : "☆" }}
                    </button>
                  </div>
                  <div class="em-run-meta">
                    <span class="em-pill2">Base：{{ r.targetBaseName || "(未设置)" }}</span>
                    <span class="em-pill2">LoRA：{{ r.targetLoraCount }}</span>
                    <span v-if="r.targetHasPromptStyle" class="em-pill2">PromptStyle</span>
                    <span v-if="r.applyOk === true" class="em-pill2 em-ok">OK</span>
                    <span v-else-if="r.applyOk === false" class="em-pill2 em-bad" :title="r.applyError || ''">FAILED</span>
                    <span v-if="r.applyDurationMs != null" class="em-pill2">耗时：{{ r.applyDurationMs }}ms</span>
                  </div>
                </div>
                <div class="em-run-actions">
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onToggleRunDetail(r.indexFromLatest)">
                    {{ expandedRunIndex === r.indexFromLatest ? "收起详情" : "详情" }}
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onRollbackToRun(r.indexFromLatest)">
                    回滚到此处
                  </button>
                  <button
                    v-if="r.applyOk === false"
                    class="em-btn secondary"
                    type="button"
                    :disabled="saving || store.loading"
                    @click="onRetryRun(r.indexFromLatest)"
                  >
                    重试
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onCopyRunDiagnostics(r.indexFromLatest)">
                    复制诊断
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onSaveRunAsWorkflow(r.indexFromLatest)">
                    保存为工作流
                  </button>
                  <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onExportRunAsWorkflowJson(r.indexFromLatest)">
                    导出工作流
                  </button>
                </div>
              </div>
              <div v-if="expandedRunIndex != null && expandedRunIndex === r.indexFromLatest" class="em-run-detail">
                <div v-if="!expandedRunDetail" class="em-muted">加载详情中…</div>
                <div v-else class="em-run-detail-grid">
                  <div>
                    <div class="em-muted">目标（apply）</div>
                    <div><b>Base</b>：{{ expandedRunDetail.targetBaseName || "(未设置)" }}</div>
                    <div><b>LoRA</b>：{{ expandedRunDetail.targetLoraCount }}</div>
                    <div><b>PromptStyle</b>：{{ expandedRunDetail.targetHasPromptStyle ? "是" : "否" }}</div>
                  </div>
                  <div>
                    <div class="em-muted">回滚快照（apply 前）</div>
                    <div><b>Base</b>：{{ expandedRunDetail.snapshotBaseName || "(未设置)" }}</div>
                    <div><b>LoRA</b>：{{ expandedRunDetail.snapshotLoraCount }}</div>
                    <div><b>PromptStyle</b>：{{ expandedRunDetail.snapshotHasPromptStyle ? "是" : "否" }}</div>
                  </div>
                  <div style="grid-column: 1 / -1" v-if="expandedRunDetail.applyOk === false">
                    <div class="em-muted">错误信息</div>
                    <pre class="em-pre">{{ expandedRunDetail.applyError || "(无错误信息)" }}</pre>
                  </div>
                  <div style="grid-column: 1 / -1" v-else-if="expandedRunDetail.applyOk === true">
                    <div class="em-muted">结果</div>
                    <div><b>modelPath</b>：{{ expandedRunDetail.applyModelPath || "(未返回)" }}</div>
                    <div><b>durationMs</b>：{{ expandedRunDetail.applyDurationMs ?? "(未返回)" }}</div>
                    <details>
                      <summary class="em-muted2">llamaArgs（展开）</summary>
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
        设为角色默认
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearSessionOverride">
        清除会话覆盖
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearRoleDefault">
        清除角色默认
      </button>
    </div>
  </section>
</template>

<style scoped>
.em-root {
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.em-h {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
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
}
.em-workflows {
  margin-top: 10px;
}
.em-wf-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.em-wf-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
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
  margin-top: 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
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
  margin-top: 12px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
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
  min-width: 520px;
  max-width: 720px;
}
.em-runs-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
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
@media (max-width: 1080px) {
  .em-grid {
    grid-template-columns: 1fr;
  }
}
</style>

