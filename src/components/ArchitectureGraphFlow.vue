<script setup lang="ts">
import { applyNodeChanges, VueFlow } from "@vue-flow/core";
import { markRaw, onMounted, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import { MiniMap } from "@vue-flow/minimap";
import ArchBackendEdge from "./architecture-graph/ArchBackendEdge.vue";
import ArchBusNode from "./architecture-graph/ArchBusNode.vue";
import ArchComplexNode from "./architecture-graph/ArchComplexNode.vue";
import ArchGraphFitView from "./architecture-graph/ArchGraphFitView.vue";
import ArchKernelNode from "./architecture-graph/ArchKernelNode.vue";
import ArchModuleNode from "./architecture-graph/ArchModuleNode.vue";
import ArchPluginNode from "./architecture-graph/ArchPluginNode.vue";
import { archGraphActionsKey } from "./architecture-graph/archGraphContext";
import { useArchitectureGraphLayout } from "../composables/useArchitectureGraphLayout";
import { useArchitectureGraphModel, type CoreModule } from "../composables/useArchitectureGraphModel";
import { BACKEND_COLORS, GRAPH_SURFACE } from "../lib/graphEditorTheme";

const handleInColor = BACKEND_COLORS.builtin.handle;
const handleOutColor = BACKEND_COLORS.directory.handle;
import { useAppToast } from "../composables/useAppToast";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";
import { setSessionPluginBackend } from "../utils/tauri-api";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();

const { t } = useI18n();
const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const busy = ref(false);

const graphLayout = useArchitectureGraphLayout();

const {
  nodes: builtNodes,
  edges: builtEdges,
  togglePluginExpand,
  hiddenPluginCount,
} = useArchitectureGraphModel();

const nodes = ref([]);
const edges = ref([]);

function syncGraphFromModel() {
  nodes.value = builtNodes.value.map((n) => {
    const applied = graphLayout.applyToNode(n.id, n.type, n.position.x, n.position.y);
    return {
      ...n,
      position: { x: applied.x, y: applied.y },
      width: applied.width,
      height: applied.height,
      style: applied.style,
    };
  });
  edges.value = [...builtEdges.value];
}

function onNodesChange(changes: unknown[]) {
  const list = changes as Array<{
    type: string;
    id?: string;
    position?: { x: number; y: number };
    dimensions?: { width: number; height: number };
    dragging?: boolean;
  }>;
  nodes.value = applyNodeChanges(list, nodes.value);
  let dirty = false;
  for (const c of list) {
    if (c.type === "position" && c.id && c.position && c.dragging === false) {
      const base = builtNodes.value.find((b) => b.id === c.id);
      if (base) {
        graphLayout.setPosition(
          c.id,
          c.position.x - base.position.x,
          c.position.y - base.position.y,
        );
        dirty = true;
      }
    }
    if (c.type === "dimensions" && c.id && c.dimensions) {
      graphLayout.setSize(c.id, Math.round(c.dimensions.width), Math.round(c.dimensions.height));
      dirty = true;
    }
  }
  if (dirty) graphLayout.save();
}

function onResetLayout() {
  graphLayout.reset();
  syncGraphFromModel();
  showToast("success", t("pluginWorkbench.graph.layoutResetDone"));
}

watch(builtNodes, syncGraphFromModel, { deep: true });
watch(builtEdges, syncGraphFromModel, { deep: true });
watch(
  () => roleStore.roleInfo.pluginBackendsEffective,
  syncGraphFromModel,
  { deep: true },
);

const nodeTypes = {
  archKernel: markRaw(ArchKernelNode),
  archBus: markRaw(ArchBusNode),
  archModule: markRaw(ArchModuleNode),
  archPlugin: markRaw(ArchPluginNode),
  archComplex: markRaw(ArchComplexNode),
};

const edgeTypes = {
  archBackend: markRaw(ArchBackendEdge),
};

provide(archGraphActionsKey, {
  busy: () => busy.value,
  onBackendChange: async (module: CoreModule, value: string) => {
    const backend = value === "__pack_default__" ? null : value;
    busy.value = true;
    try {
      const info = await setSessionPluginBackend(roleStore.currentRoleId, module, backend);
      roleStore.applyRoleInfo(info);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    } finally {
      busy.value = false;
    }
  },
  onFocusPlugin: (id: string) => emit("focus-plugin", id),
  onToggleExpand: (module: CoreModule) => {
    if (hiddenPluginCount(module) > 0) togglePluginExpand(module);
  },
  onTogglePluginDisabled: (id: string) => {
    try {
      pluginStore.setPluginDisabled(id, !pluginStore.isPluginDisabled(id));
      showToast("success", t("pluginWorkbench.graph.ctxToggled"));
    } catch (err) {
      showToast("error", err instanceof Error ? err.message : String(err));
    }
  },
  onUninstallPlugin: async (id: string) => {
    try {
      await pluginStore.uninstallPluginFromGitIndex(id);
      showToast("success", t("pluginWorkbench.graph.ctxUninstalled", { id }));
    } catch (err) {
      showToast("error", err instanceof Error ? err.message : String(err));
    }
  },
});

onMounted(syncGraphFromModel);
</script>

<template>
  <div class="agf-root">
    <p class="agf-lead">{{ t("pluginWorkbench.graph.lead") }}</p>
    <p class="agf-vendor">
      {{ t("pluginWorkbench.graph.poweredBy") }}
      <a href="https://vueflow.dev/" target="_blank" rel="noopener noreferrer">Vue Flow</a>
      · {{ t("pluginWorkbench.graph.comfyRef") }}
    </p>

    <div class="agf-toolbar">
      <button type="button" class="agf-tb-btn" @click="onResetLayout">
        {{ t("pluginWorkbench.graph.resetLayout") }}
      </button>
      <span class="agf-tb-hint">{{ t("pluginWorkbench.graph.resizeHint") }}</span>
    </div>

    <div
      class="agf-vf"
      role="application"
      :aria-label="t('pluginWorkbench.graph.canvasAria')"
    >
      <VueFlow
        v-model:nodes="nodes"
        v-model:edges="edges"
        :node-types="nodeTypes"
        :edge-types="edgeTypes"
        :min-zoom="0.35"
        :max-zoom="1.8"
        :nodes-draggable="true"
        :nodes-connectable="false"
        :elements-selectable="true"
        :fit-view-on-init="false"
        :default-viewport="{ zoom: 0.85 }"
        @nodes-change="onNodesChange"
      >
        <Background
          variant="dots"
          :gap="GRAPH_SURFACE.gridGap"
          :size="GRAPH_SURFACE.gridDotSize"
          :pattern-color="GRAPH_SURFACE.gridDot"
          :bg-color="GRAPH_SURFACE.canvas"
        />
        <MiniMap pannable zoomable />
        <Controls />
        <ArchGraphFitView />
      </VueFlow>
    </div>

    <div class="agf-legend" role="list">
      <span class="agf-legend-item" role="listitem">
        <span class="agf-swatch agf-swatch--builtin" />{{ t("pluginWorkbench.graph.legendBuiltin") }}
      </span>
      <span class="agf-legend-item" role="listitem">
        <span class="agf-swatch agf-swatch--remote" />{{ t("pluginWorkbench.graph.legendRemote") }}
      </span>
      <span class="agf-legend-item" role="listitem">
        <span class="agf-swatch agf-swatch--directory" />{{ t("pluginWorkbench.graph.legendDirectory") }}
      </span>
      <span class="agf-legend-hint">{{ t("pluginWorkbench.graph.panHint") }}</span>
    </div>
  </div>
</template>

<style>
@import "@vue-flow/core/dist/style.css";
@import "@vue-flow/core/dist/theme-default.css";
@import "@vue-flow/controls/dist/style.css";
@import "@vue-flow/minimap/dist/style.css";
@import "@vue-flow/node-resizer/dist/style.css";
@import "./architecture-graph/archGraphTheme.css";
</style>

<style scoped>
.agf-root {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.agf-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.agf-vendor {
  margin: 0;
  font-size: 10px;
  color: var(--text-secondary);
}
.agf-vendor a {
  color: color-mix(in srgb, var(--text-accent, var(--accent)) 75%, var(--text-secondary));
}
.agf-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.agf-tb-btn {
  padding: 4px 12px;
  font-size: 11px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.agf-tb-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 30%, var(--border-light));
}
.agf-tb-hint {
  font-size: 10px;
  color: var(--text-secondary);
}
.agf-vf {
  --arch-node-bg: v-bind("GRAPH_SURFACE.nodeBg");
  --arch-node-elevated: v-bind("GRAPH_SURFACE.nodeElevated");
  --arch-node-border: v-bind("GRAPH_SURFACE.nodeBorder");
  --arch-node-shadow: v-bind("GRAPH_SURFACE.nodeShadow");
  --arch-selection: v-bind("GRAPH_SURFACE.selectionRing");
  --arch-text: v-bind("GRAPH_SURFACE.text");
  --arch-text-muted: v-bind("GRAPH_SURFACE.textMuted");
  height: min(560px, 58vh);
  min-height: 400px;
  border-radius: var(--radius-card);
  border: 1px solid #3c3c3c;
  background: v-bind("GRAPH_SURFACE.canvas");
  overflow: hidden;
}
.agf-vf :deep(.vue-flow) {
  width: 100%;
  height: 100%;
  background: v-bind("GRAPH_SURFACE.canvas");
}
.agf-vf :deep(.vue-flow__background) {
  background: v-bind("GRAPH_SURFACE.canvas");
}
.agf-vf :deep(.vue-flow__node) {
  border: none;
  background: transparent;
  box-shadow: none;
  padding: 0;
}
.agf-vf :deep(.vue-flow__node.selected) {
  box-shadow: none;
}
.agf-vf :deep(.vue-flow__handle) {
  width: 10px;
  height: 10px;
  border: 2px solid #2d2d30;
  background: #3c3c3c;
}
.agf-vf :deep(.vue-flow__handle.agn-handle--in) {
  border-color: v-bind(handleInColor);
  background: color-mix(in srgb, v-bind(handleInColor) 22%, #3c3c3c);
}
.agf-vf :deep(.vue-flow__handle.agn-handle--out) {
  border-color: v-bind(handleOutColor);
  background: color-mix(in srgb, v-bind(handleOutColor) 22%, #3c3c3c);
}
.agf-vf :deep(.vue-flow__edge-path) {
  stroke-width: 1.75;
}
.agf-vf :deep(.vue-flow__minimap) {
  border-radius: 8px;
  border: 1px solid #4e4e52;
  background: #2d2d30;
}
.agf-vf :deep(.vue-flow__minimap-mask) {
  fill: color-mix(in srgb, #252526 55%, transparent);
}
.agf-vf :deep(.vue-flow__controls) {
  box-shadow: none;
  border: 1px solid #4e4e52;
  border-radius: 8px;
  overflow: hidden;
  background: #2d2d30;
}
.agf-vf :deep(.vue-flow__controls-button) {
  background: #3c3c3c;
  border-bottom: 1px solid #4e4e52;
  fill: #c5c5c5;
}
.agf-vf :deep(.vue-flow__controls-button:hover) {
  background: #454548;
}
.agf-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  font-size: 11px;
  color: var(--text-secondary);
}
.agf-legend-hint {
  margin-left: auto;
  font-size: 10px;
}
.agf-swatch {
  display: inline-block;
  width: 18px;
  height: 0;
  border-top: 3px solid;
  margin-right: 4px;
  vertical-align: middle;
  opacity: 0.9;
}
.agf-swatch--builtin {
  border-color: v-bind("BACKEND_COLORS.builtin.stroke");
}
.agf-swatch--remote {
  border-color: v-bind("BACKEND_COLORS.remote.stroke");
  border-top-style: dashed;
}
.agf-swatch--directory {
  border-color: v-bind("BACKEND_COLORS.directory.stroke");
  border-top-style: dotted;
}
</style>
