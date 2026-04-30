<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { VueFlow, useVueFlow, type Edge, type Node, type Connection } from "@vue-flow/core";
import "@vue-flow/core/dist/style.css";
import type { ExpertEdge, ExpertGraph, ExpertNode } from "../../utils/tauri-api";

const props = defineProps<{
  modelValue: ExpertGraph;
  selectedNodeId?: string | null;
}>();
const emit = defineEmits<{
  (e: "update:modelValue", v: ExpertGraph): void;
  (e: "update:selectedNodeId", v: string | null): void;
}>();

const { onConnect, onNodesChange, onEdgesChange, addEdges } = useVueFlow();

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

const idSet = (nodes: ExpertNode[]): Set<string> =>
  new Set(nodes.map((n) => String((n as any).id ?? "").trim()).filter(Boolean));

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
  emitGraph({ ...g, nodes: nextNodes, edges: nextEdges });
}

watch(
  () => props.modelValue,
  (g) => {
    internalNodes.value = toFlowNodes(g);
    internalEdges.value = toFlowEdges(g);
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
  if (bases === 0) warnings.push("缺少 BaseModel 节点（将无法选择 base GGUF）。");
  if (bases > 1) warnings.push("存在多个 BaseModel：编译时会选择一个“主 Base”。");
  if (ps > 1) warnings.push("存在多个 PromptStyle：编译时会选择一个。");
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
        <button type="button" class="emc-btn" @click="addNode('base')">+ BaseModel</button>
        <button type="button" class="emc-btn" @click="addNode('lora')">+ LoRA</button>
        <button type="button" class="emc-btn" @click="addNode('style')">+ PromptStyle</button>
        <button
          type="button"
          class="emc-btn danger"
          :disabled="!selectedId"
          @click="deleteSelectedNode"
        >
          删除选中节点
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
      @pane-click="selectedId = null"
    />
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
</style>

