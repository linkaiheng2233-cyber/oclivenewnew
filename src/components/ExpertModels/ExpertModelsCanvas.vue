<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { VueFlow, useVueFlow, type Edge, type Node, type Connection } from "@vue-flow/core";
import "@vue-flow/core/dist/style.css";
import type { ExpertEdge, ExpertGraph, ExpertNode } from "../../utils/tauri-api";

const { t } = useI18n();

const props = defineProps<{
  modelValue: ExpertGraph;
  selectedNodeId?: string | null;
}>();
const emit = defineEmits<{
  (e: "update:modelValue", v: ExpertGraph): void;
  (e: "update:selectedNodeId", v: string | null): void;
}>();

const { onConnect, onNodesChange, onEdgesChange, addEdges, fitView } = useVueFlow();

const internalNodes = ref<Node[]>([]);
const internalEdges = ref<Edge[]>([]);
const selectedId = computed<string | null>({
  get() {
    const t = String(props.selectedNodeId ?? "").trim();
    return t ? t : null;
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

const idSet = (nodes: ExpertNode[]): Set<string> =>
  new Set(nodes.map((n) => String((n as any).id ?? "").trim()).filter(Boolean));

function syncSelectionClasses() {
  const sid = (selectedId.value ?? "").trim();
  const seid = (selectedEdgeId.value ?? "").trim();
  internalNodes.value = internalNodes.value.map((n) => ({
    ...n,
    class: n.id === sid ? "emc-node--selected" : "",
  }));
  internalEdges.value = internalEdges.value.map((e) => ({
    ...e,
    class: e.id === seid ? "emc-edge--selected" : "",
  }));
}

watch([selectedId, selectedEdgeId], () => syncSelectionClasses());

function nodeLabel(n: ExpertNode): string {
  if (n.type === "base_model") return "BaseModel";
  if (n.type === "lora_adapter") return "LoRA";
  return "PromptStyle";
}

function toFlowNodes(graph: ExpertGraph): Node[] {
  const ns = graph.nodes ?? [];
  return ns.map((n) => {
    const id = (n as any).id as string;
    const ui = (n as any).ui as { x: number; y: number } | null | undefined;
    const x = ui?.x ?? 40;
    const y = ui?.y ?? 40;
    return {
      id,
      position: { x, y },
      data: {
        label: `${nodeLabel(n)} · ${id}`,
        expertType: n.type,
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
  // stable sort
  out.sort((a, b) => (a.from === b.from ? a.to.localeCompare(b.to) : a.from.localeCompare(b.from)));
  // dedup
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

function onOpenCtxMenu(
  ev: MouseEvent,
  kind: "node" | "edge" | "pane",
  id: string | null,
) {
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
  const nextNodes = nodes.map((n) => {
    const baseX = 60;
    const baseY = 60;
    if (n.type === "base_model") {
      return { ...(n as any), ui: { x: baseX, y: baseY } } as ExpertNode;
    }
    if (n.type === "prompt_style") {
      return { ...(n as any), ui: { x: baseX + 520, y: baseY } } as ExpertNode;
    }
    if (n.type === "lora_adapter") {
      const y = baseY + 120 + loraIdx * 90;
      loraIdx += 1;
      return { ...(n as any), ui: { x: baseX + 260, y } } as ExpertNode;
    }
    return n;
  });
  emitGraph({ ...g, nodes: nextNodes });
  // Give Vue Flow a tick to render, then fit.
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
  addEdges([
    {
      id: `e_${c.source}_${c.target}_${Date.now()}`,
      source: c.source!,
      target: c.target!,
    },
  ]);
});

onEdgesChange(() => {
  // edge changes are reflected in internalEdges via v-model binding; sync on next tick by watcher below
});

onNodesChange(() => {
  // same as edges
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
  const warnings: string[] = [];
  if (bases === 0) warnings.push(String(t("expertModels.canvas.warnings.missingBase")));
  if (bases > 1) warnings.push(String(t("expertModels.canvas.warnings.multipleBase")));
  if (ps > 1) warnings.push(String(t("expertModels.canvas.warnings.multiplePromptStyle")));
  return warnings;
});

function addNode(kind: "base" | "lora" | "style") {
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
    kind === "base" ? mkId("base") : kind === "lora" ? mkId("lora") : mkId("style");
  const ui = { x: 60 + (g.nodes?.length ?? 0) * 30, y: 60 + (g.nodes?.length ?? 0) * 20 };
  let node: ExpertNode;
  if (kind === "base") node = { type: "base_model", id, ggufPath: "", ui };
  else if (kind === "lora")
    node = { type: "lora_adapter", id, ggufPath: "", strength: 1, enabled: true, order: 0, ui };
  else node = { type: "prompt_style", id, style: {}, ui };
  emitGraph({ ...g, nodes: [...(g.nodes ?? []), node] });
}
</script>

<template>
  <div class="emc-root">
    <div class="emc-top">
      <div class="emc-actions">
        <button type="button" class="emc-btn" @click="addNode('base')">{{ t("expertModels.canvas.actions.addBase") }}</button>
        <button type="button" class="emc-btn" @click="addNode('lora')">{{ t("expertModels.canvas.actions.addLora") }}</button>
        <button type="button" class="emc-btn" @click="addNode('style')">{{ t("expertModels.canvas.actions.addPromptStyle") }}</button>
        <button type="button" class="emc-btn" @click="tidyLayout">{{ t("expertModels.canvas.actions.tidyLayout") }}</button>
        <button type="button" class="emc-btn" @click="onFitView">{{ t("expertModels.canvas.actions.fitView") }}</button>
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
:deep(.emc-node--selected) {
  outline: 2px solid color-mix(in srgb, var(--danger-600, #c0392b) 40%, #ffffff);
  outline-offset: 2px;
  border-radius: 6px;
}
:deep(.emc-edge--selected path) {
  stroke: var(--danger-600, #c0392b) !important;
  stroke-width: 3 !important;
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

