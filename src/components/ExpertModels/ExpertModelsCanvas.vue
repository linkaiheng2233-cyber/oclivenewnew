<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  VueFlow,
  useVueFlow,
  type Connection,
  type Edge,
  type Node,
} from "@vue-flow/core";
import "@vue-flow/core/dist/style.css";
import type { ExpertEdge, ExpertGraph, ExpertNode } from "../../utils/tauri-api";
import { expertModelsValidateGraph } from "../../utils/tauri-api";
import ExpertFlowExpertNode from "./ExpertFlowExpertNode.vue";

const { t } = useI18n();

const props = defineProps<{
  modelValue: ExpertGraph;
  selectedNodeId?: string | null;
}>();
const emit = defineEmits<{
  (e: "update:modelValue", v: ExpertGraph): void;
  (e: "update:selectedNodeId", v: string | null): void;
}>();

const nodeTypes = { expert: ExpertFlowExpertNode };

const { onConnect, addEdges, fitView } = useVueFlow();

const internalNodes = ref<Node[]>([]);
const internalEdges = ref<Edge[]>([]);
const selectedId = computed<string | null>({
  get() {
    const s = String(props.selectedNodeId ?? "").trim();
    return s ? s : null;
  },
  set(v) {
    emit("update:selectedNodeId", v);
  },
});

const selectedEdgeId = ref<string | null>(null);
const ctxMenu = ref<{
  open: boolean;
  x: number;
  y: number;
  kind: "node" | "edge" | "pane";
  id: string | null;
}>({ open: false, x: 0, y: 0, kind: "pane", id: null });

const validating = ref(false);
const validationOk = ref<boolean | null>(null);
const validationIssues = ref<
  { severity: string; code: string; message: string; nodeIds?: string[] }[]
>([]);
const validationErrorNodeIds = shallowRef(new Set<string>());
const validationStamp = ref(0);

const idSet = (nodes: ExpertNode[]): Set<string> =>
  new Set(nodes.map((n) => String((n as any).id ?? "").trim()).filter(Boolean));

function expertById(id: string): ExpertNode | undefined {
  return (props.modelValue.nodes ?? []).find((n) => String((n as any).id ?? "") === id) as
    | ExpertNode
    | undefined;
}

function nodeTypeOf(id: string): string | undefined {
  return expertById(id)?.type;
}

function isValidExpertConnection(c: Connection): boolean {
  if (!c.source || !c.target || c.source === c.target) return false;
  const st = nodeTypeOf(String(c.source));
  const tt = nodeTypeOf(String(c.target));
  if (!st || !tt) return false;
  if (tt === "base_model") return false;
  if (st === "event_trigger") return false;
  if (st === "base_model") {
    return tt === "lora_adapter" || tt === "cloud_model" || tt === "prompt_style" || tt === "event_trigger";
  }
  if (st === "lora_adapter") {
    return (
      tt === "lora_adapter" ||
      tt === "cloud_model" ||
      tt === "prompt_style" ||
      tt === "event_trigger"
    );
  }
  if (st === "cloud_model") {
    return tt === "lora_adapter" || tt === "prompt_style" || tt === "event_trigger";
  }
  if (st === "prompt_style") {
    return tt === "event_trigger" || tt === "lora_adapter" || tt === "cloud_model";
  }
  return false;
}

isValidConnection.value = isValidExpertConnection;

function syncSelectionClasses() {
  const sid = (selectedId.value ?? "").trim();
  const seid = (selectedEdgeId.value ?? "").trim();
  const err = validationErrorNodeIds.value;
  internalNodes.value = internalNodes.value.map((n) => ({
    ...n,
    class: [n.id === sid ? "emc-node--selected" : "", err.has(n.id) ? "emc-node--error" : ""]
      .filter(Boolean)
      .join(" "),
  }));
  internalEdges.value = internalEdges.value.map((e) => ({
    ...e,
    class: [
      e.id === seid ? "emc-edge--selected" : "",
      err.has(String(e.source)) || err.has(String(e.target)) ? "emc-edge--error" : "",
    ]
      .filter(Boolean)
      .join(" "),
  }));
}

watch([selectedId, selectedEdgeId, validationStamp], () => syncSelectionClasses());

function patchExpertNode(nodeId: string, patch: Record<string, unknown>): void {
  const g = props.modelValue;
  const next = (g.nodes ?? []).map((n) =>
    String((n as any).id ?? "") === nodeId ? ({ ...(n as any), ...patch } as ExpertNode) : n,
  );
  emitGraph({ ...g, nodes: next });
}

function nodeLabel(n: ExpertNode): string {
  if (n.type === "base_model") return "BaseModel";
  if (n.type === "lora_adapter") return "LoRA";
  if (n.type === "cloud_model") return "Cloud";
  if (n.type === "event_trigger") return "Event";
  return "PromptStyle";
}

function toFlowNodes(graph: ExpertGraph): Node[] {
  const err = validationErrorNodeIds.value;
  const ns = graph.nodes ?? [];
  return ns.map((n) => {
    const id = (n as any).id as string;
    const ui = (n as any).ui as { x: number; y: number } | null | undefined;
    const x = ui?.x ?? 40;
    const y = ui?.y ?? 40;
    return {
      id,
      type: "expert",
      position: { x, y },
      data: {
        label: `${nodeLabel(n)} · ${id}`,
        expert: n,
        error: err.has(id),
        onPatch: (patch: Record<string, unknown>) => patchExpertNode(id, patch),
      },
      draggable: true,
    } satisfies Node;
  });
}

function toFlowEdges(graph: ExpertGraph): Edge[] {
  const es = graph.edges ?? [];
  return es
    .map((e, i) => {
      const from = (e.from ?? "").trim();
      const to = (e.to ?? "").trim();
      if (!from || !to) return null;
      return {
        id: `e_${from}_${to}_${i}`,
        source: from,
        target: to,
      } satisfies Edge;
    })
    .filter(Boolean) as Edge[];
}

function toExpertEdges(flowEdges: Edge[]): ExpertEdge[] {
  const out: ExpertEdge[] = [];
  for (const e of flowEdges) {
    const from = String(e.source ?? "").trim();
    const to = String(e.target ?? "").trim();
    if (!from || !to) continue;
    out.push({ from, to });
  }
  out.sort((a, b) => (a.from === b.from ? a.to.localeCompare(b.to) : a.from.localeCompare(b.from)));
  return out.filter((x, i) => i === 0 || x.from !== out[i - 1]!.from || x.to !== out[i - 1]!.to);
}

function applyPositions(graph: ExpertGraph, nodes: Node[]): ExpertGraph {
  const byId = new Map(nodes.map((n) => [n.id, n.position] as const));
  const nextNodes = (graph.nodes ?? []).map((n) => {
    const id = (n as any).id as string;
    const pos = byId.get(id);
    if (!pos) return n;
    return { ...(n as any), ui: { x: pos.x, y: pos.y } } as ExpertNode;
  });
  return { ...graph, nodes: nextNodes };
}

function emitGraph(next: ExpertGraph) {
  emit("update:modelValue", next);
}

function clearValidationDecorations(): void {
  validationIssues.value = [];
  validationOk.value = null;
  validationErrorNodeIds.value = new Set();
  validationStamp.value += 1;
}

function deleteSelectedNode() {
  const id = (selectedId.value ?? "").trim();
  if (!id) return;
  const g = props.modelValue;
  const nextNodes = (g.nodes ?? []).filter((n) => String((n as any).id ?? "") !== id);
  const nextEdges = (g.edges ?? []).filter(
    (e) => String(e.from ?? "").trim() !== id && String(e.to ?? "").trim() !== id,
  );
  selectedId.value = null;
  selectedEdgeId.value = null;
  clearValidationDecorations();
  emitGraph({ ...g, nodes: nextNodes, edges: nextEdges });
}

function deleteSelectedEdge() {
  const eid = (selectedEdgeId.value ?? "").trim();
  if (!eid) return;
  internalEdges.value = internalEdges.value.filter((e) => e.id !== eid);
  selectedEdgeId.value = null;
}

function closeCtxMenu() {
  ctxMenu.value = { ...ctxMenu.value, open: false, id: null };
}

function onOpenCtxMenu(ev: MouseEvent, kind: "node" | "edge" | "pane", id: string | null) {
  ev.preventDefault();
  ev.stopPropagation();
  ctxMenu.value = { open: true, x: ev.clientX, y: ev.clientY, kind, id };
}

function deleteCtxTarget() {
  if (!ctxMenu.value.open) return;
  if (ctxMenu.value.kind === "node" && ctxMenu.value.id) {
    selectedId.value = ctxMenu.value.id;
    deleteSelectedNode();
  } else if (ctxMenu.value.kind === "edge" && ctxMenu.value.id) {
    selectedEdgeId.value = ctxMenu.value.id;
    deleteSelectedEdge();
  }
  closeCtxMenu();
}

function clearSelection() {
  selectedId.value = null;
  selectedEdgeId.value = null;
  closeCtxMenu();
}

function tidyLayout() {
  const g = props.modelValue;
  const nodes = g.nodes ?? [];
  let loraIdx = 0;
  let evtIdx = 0;
  const nextNodes = nodes.map((n) => {
    const baseX = 60;
    const baseY = 60;
    if (n.type === "base_model") {
      return { ...(n as any), ui: { x: baseX, y: baseY } } as ExpertNode;
    }
    if (n.type === "prompt_style") {
      return { ...(n as any), ui: { x: baseX + 520, y: baseY } } as ExpertNode;
    }
    if (n.type === "cloud_model") {
      return { ...(n as any), ui: { x: baseX + 520, y: baseY + 80 } } as ExpertNode;
    }
    if (n.type === "event_trigger") {
      const y = baseY + 360 + evtIdx * 100;
      evtIdx += 1;
      return { ...(n as any), ui: { x: baseX, y } } as ExpertNode;
    }
    if (n.type === "lora_adapter") {
      const y = baseY + 120 + loraIdx * 90;
      loraIdx += 1;
      return { ...(n as any), ui: { x: baseX + 260, y } } as ExpertNode;
    }
    return n;
  });
  clearValidationDecorations();
  emitGraph({ ...g, nodes: nextNodes });
  setTimeout(() => {
    try {
      fitView?.({ padding: 0.18, includeHiddenNodes: true });
    } catch {
      // ignore
    }
  }, 0);
}

function onFitView() {
  try {
    fitView?.({ padding: 0.18, includeHiddenNodes: true });
  } catch {
    // ignore
  }
}

function onKeydown(ev: KeyboardEvent) {
  if (ev.key !== "Delete" && ev.key !== "Backspace") return;
  const tag = (ev.target as HTMLElement | null)?.tagName?.toLowerCase();
  if (tag === "input" || tag === "textarea") return;
  if (selectedId.value) {
    ev.preventDefault();
    deleteSelectedNode();
    return;
  }
  if (selectedEdgeId.value) {
    ev.preventDefault();
    deleteSelectedEdge();
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

watch(
  () => props.modelValue,
  (g) => {
    internalNodes.value = toFlowNodes(g);
    internalEdges.value = toFlowEdges(g);
    syncSelectionClasses();
  },
  { immediate: true, deep: true },
);

onConnect((c: Connection) => {
  if (!isValidExpertConnection(c)) return;
  addEdges([
    {
      id: `e_${c.source}_${c.target}_${Date.now()}`,
      source: c.source!,
      target: c.target!,
    },
  ]);
});

watch(
  [internalNodes, internalEdges],
  () => {
    const g = props.modelValue;
    const ids = idSet(g.nodes ?? []);
    const filteredEdges = internalEdges.value.filter(
      (e) => ids.has(String(e.source)) && ids.has(String(e.target)),
    );
    const withPos = applyPositions(g, internalNodes.value);
    emitGraph({ ...withPos, edges: toExpertEdges(filteredEdges) });
  },
  { deep: true },
);

const health = computed(() => {
  const g = props.modelValue;
  const nodes = g.nodes ?? [];
  const bases = nodes.filter((n) => n.type === "base_model").length;
  const ps = nodes.filter((n) => n.type === "prompt_style").length;
  const clouds = nodes.filter((n) => n.type === "cloud_model" && n.enabled).length;
  const warnings: string[] = [];
  if (bases === 0 && clouds === 0) {
    warnings.push(String(t("expertModels.canvas.warnings.missingBaseOrCloud")));
  }
  if (bases > 1) warnings.push(String(t("expertModels.canvas.warnings.multipleBase")));
  if (clouds > 1) warnings.push(String(t("expertModels.canvas.warnings.multipleCloud")));
  if (ps > 1) warnings.push(String(t("expertModels.canvas.warnings.multiplePromptStyle")));
  return warnings;
});

const problemRows = computed(() => {
  const rows: { key: string; severity: "warn" | "error"; text: string; nodeIds: string[] }[] = [];
  for (const w of health.value) {
    rows.push({ key: `w:${w}`, severity: "warn", text: w, nodeIds: [] });
  }
  for (const iss of validationIssues.value) {
    const sev = iss.severity === "error" ? "error" : "warn";
    rows.push({
      key: `e:${iss.code}:${iss.message}`,
      severity: sev,
      text: iss.message,
      nodeIds: iss.nodeIds ?? [],
    });
  }
  return rows;
});

async function onValidateCompile(): Promise<void> {
  validating.value = true;
  validationOk.value = null;
  validationIssues.value = [];
  validationErrorNodeIds.value = new Set();
  try {
    const res = await expertModelsValidateGraph({ graph: props.modelValue });
    validationOk.value = res.ok;
    validationIssues.value = (res.issues ?? []).map((x) => ({
      severity: x.severity,
      code: x.code,
      message: x.message,
      nodeIds: x.nodeIds ?? [],
    }));
    const ids = new Set<string>();
    for (const iss of validationIssues.value) {
      for (const nid of iss.nodeIds ?? []) {
        if (nid.trim()) ids.add(nid.trim());
      }
    }
    validationErrorNodeIds.value = ids;
    validationStamp.value += 1;
    internalNodes.value = toFlowNodes(props.modelValue);
    internalEdges.value = toFlowEdges(props.modelValue);
    syncSelectionClasses();
  } catch (e) {
    validationOk.value = false;
    validationIssues.value = [
      {
        severity: "error",
        code: "invoke_failed",
        message: e instanceof Error ? e.message : String(e),
        nodeIds: [],
      },
    ];
    validationStamp.value += 1;
    syncSelectionClasses();
  } finally {
    validating.value = false;
  }
}

function focusNode(id: string): void {
  const tid = id.trim();
  if (!tid) return;
  selectedId.value = tid;
  selectedEdgeId.value = null;
}

function addNode(kind: "base" | "lora" | "style" | "cloud" | "event") {
  const g = props.modelValue;
  const existing = idSet(g.nodes ?? []);
  const mkId = (prefix: string) => {
    for (let i = 1; i < 9999; i += 1) {
      const id = `${prefix}_${i}`;
      if (!existing.has(id)) return id;
    }
    return `${prefix}_${Math.random().toString(36).slice(2, 8)}`;
  };
  const id =
    kind === "base"
      ? mkId("base")
      : kind === "lora"
        ? mkId("lora")
        : kind === "cloud"
          ? mkId("cloud")
          : kind === "event"
            ? mkId("evt")
            : mkId("style");
  const ui = { x: 60 + (g.nodes?.length ?? 0) * 30, y: 60 + (g.nodes?.length ?? 0) * 20 };
  let node: ExpertNode;
  if (kind === "base") node = { type: "base_model", id, ggufPath: "", ui };
  else if (kind === "lora")
    node = { type: "lora_adapter", id, ggufPath: "", strength: 1, enabled: true, order: 0, ui };
  else if (kind === "cloud")
    node = {
      type: "cloud_model",
      id,
      hostSource: "host",
      model: "",
      enabled: true,
      ui,
    };
  else if (kind === "event")
    node = {
      type: "event_trigger",
      id,
      matchSubstring: "",
      memoryContent: "",
      importance: 0.75,
      enabled: true,
      matchScope: "any",
      ui,
    };
  else node = { type: "prompt_style", id, style: {}, ui };
  clearValidationDecorations();
  emitGraph({ ...g, nodes: [...(g.nodes ?? []), node] });
}
</script>

<template>
  <div class="emc-root">
    <div class="emc-top">
      <div class="emc-actions">
        <button type="button" class="emc-btn" @click="addNode('base')">{{ t("expertModels.canvas.actions.addBase") }}</button>
        <button type="button" class="emc-btn" @click="addNode('lora')">{{ t("expertModels.canvas.actions.addLora") }}</button>
        <button type="button" class="emc-btn" @click="addNode('cloud')">{{ t("expertModels.canvas.actions.addCloud") }}</button>
        <button type="button" class="emc-btn" @click="addNode('event')">{{ t("expertModels.canvas.actions.addEvent") }}</button>
        <button type="button" class="emc-btn" @click="addNode('style')">{{ t("expertModels.canvas.actions.addPromptStyle") }}</button>
        <button type="button" class="emc-btn" @click="tidyLayout">{{ t("expertModels.canvas.actions.tidyLayout") }}</button>
        <button type="button" class="emc-btn" @click="onFitView">{{ t("expertModels.canvas.actions.fitView") }}</button>
        <button type="button" class="emc-btn primary" :disabled="validating" @click="onValidateCompile">
          {{ validating ? t("expertModels.canvas.validateRunning") : t("expertModels.canvas.validateCompile") }}
        </button>
        <button
          type="button"
          class="emc-btn danger"
          :disabled="!selectedId"
          @click="deleteSelectedNode"
        >
          {{ t("expertModels.canvas.actions.deleteSelectedNode") }}
        </button>
        <button
          type="button"
          class="emc-btn danger"
          :disabled="!selectedEdgeId"
          @click="deleteSelectedEdge"
        >
          {{ t("expertModels.canvas.actions.deleteSelectedEdge") }}
        </button>
      </div>
      <div v-if="health.length" class="emc-warn">
        <div v-for="w in health" :key="w">{{ w }}</div>
      </div>
    </div>

    <VueFlow
      v-model:nodes="internalNodes"
      v-model:edges="internalEdges"
      :node-types="nodeTypes"
      :is-valid-connection="isValidExpertConnection"
      fit-view-on-init
      class="emc-flow"
      @node-click="selectedId = $event.node.id"
      @edge-click="selectedEdgeId = $event.edge.id"
      @node-context-menu="onOpenCtxMenu($event.event, 'node', $event.node.id)"
      @edge-context-menu="onOpenCtxMenu($event.event, 'edge', $event.edge.id)"
      @pane-click="
        selectedId = null;
        selectedEdgeId = null;
        closeCtxMenu();
      "
      @pane-context-menu="onOpenCtxMenu($event.event, 'pane', null)"
    />

    <div class="emc-issues">
      <div class="emc-issues-h">{{ t("expertModels.canvas.issuesTitle") }}</div>
      <div v-if="validationOk === true" class="emc-issues-ok">{{ t("expertModels.canvas.validateOk") }}</div>
      <div v-else-if="validationOk === false && !problemRows.length" class="emc-muted">
        {{ t("expertModels.canvas.validateFailedEmpty") }}
      </div>
      <ul v-else-if="problemRows.length" class="emc-issues-ul">
        <li
          v-for="row in problemRows"
          :key="row.key"
          :class="['emc-issues-li', row.severity === 'error' ? 'is-err' : 'is-warn']"
        >
          <button type="button" class="emc-issues-btn" @click="row.nodeIds[0] && focusNode(row.nodeIds[0])">
            {{ row.text }}
            <span v-if="row.nodeIds.length" class="emc-issues-loc">({{ row.nodeIds.join(", ") }})</span>
          </button>
        </li>
      </ul>
      <div v-else class="emc-muted">{{ t("expertModels.canvas.issuesHint") }}</div>
    </div>

    <div v-if="ctxMenu.open" class="emc-ctx" :style="{ left: `${ctxMenu.x}px`, top: `${ctxMenu.y}px` }">
      <button
        v-if="ctxMenu.kind !== 'pane'"
        type="button"
        class="emc-ctx-item danger"
        @click="deleteCtxTarget"
      >
        {{ t("expertModels.canvas.actions.delete") }}
      </button>
      <button type="button" class="emc-ctx-item" @click="clearSelection">{{ t("expertModels.canvas.actions.clearSelection") }}</button>
      <button type="button" class="emc-ctx-item" @click="closeCtxMenu">{{ t("common.close") }}</button>
    </div>
  </div>
</template>

<style scoped>
.emc-root {
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-elevated);
  overflow: hidden;
}
.emc-top {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 10px;
  border-bottom: 1px solid var(--border-light);
}
.emc-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.emc-btn {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  cursor: pointer;
  font-size: 12px;
}
.emc-btn.primary {
  border-color: color-mix(in srgb, var(--accent, #357cff) 45%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #357cff) 12%, var(--bg-primary));
}
.emc-btn.danger {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
}
.emc-warn {
  color: var(--danger-600, #c0392b);
  font-size: 12px;
  line-height: 1.4;
}
.emc-flow {
  height: 360px;
  background: var(--bg-primary);
}
.emc-issues {
  border-top: 1px solid var(--border-light);
  padding: 8px 10px 10px;
  max-height: 140px;
  overflow: auto;
  background: var(--bg-secondary);
}
.emc-issues-h {
  font-weight: 700;
  font-size: 12px;
  margin-bottom: 6px;
}
.emc-issues-ok {
  font-size: 12px;
  color: var(--success, #2e7d32);
}
.emc-muted {
  font-size: 12px;
  color: var(--text-secondary);
}
.emc-issues-ul {
  margin: 0;
  padding: 0;
  list-style: none;
}
.emc-issues-li {
  margin-bottom: 4px;
}
.emc-issues-btn {
  width: 100%;
  text-align: left;
  font-size: 12px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  cursor: pointer;
}
.emc-issues-li.is-err .emc-issues-btn {
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
}
.emc-issues-li.is-warn .emc-issues-btn {
  border-color: color-mix(in srgb, #b8860b 40%, var(--border-light));
}
.emc-issues-loc {
  font-family: ui-monospace, monospace;
  font-size: 11px;
  color: var(--text-secondary);
}
:deep(.emc-node--selected) {
  outline: 2px solid color-mix(in srgb, var(--danger-600, #c0392b) 40%, #ffffff);
  outline-offset: 2px;
  border-radius: 6px;
}
:deep(.emc-node--error) {
  outline: 2px solid var(--danger-600, #c0392b);
  outline-offset: 2px;
  border-radius: 6px;
}
:deep(.emc-edge--selected path) {
  stroke: var(--danger-600, #c0392b) !important;
  stroke-width: 3 !important;
}
:deep(.emc-edge--error path) {
  stroke: var(--danger-600, #c0392b) !important;
  stroke-width: 2.5 !important;
  stroke-dasharray: 6 4;
}
.emc-ctx {
  position: fixed;
  z-index: 9999;
  min-width: 140px;
  padding: 6px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.22);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.emc-ctx-item {
  text-align: left;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  cursor: pointer;
  font-size: 12px;
}
.emc-ctx-item.danger {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
}
</style>
