<script setup lang="ts">
import { markRaw, onMounted, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { VueFlow } from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import { MiniMap } from "@vue-flow/minimap";
import ArchBackendEdge from "./architecture-graph/ArchBackendEdge.vue";
import ArchBusNode from "./architecture-graph/ArchBusNode.vue";
import ArchComplexNode from "./architecture-graph/ArchComplexNode.vue";
import ArchKernelNode from "./architecture-graph/ArchKernelNode.vue";
import ArchModuleNode from "./architecture-graph/ArchModuleNode.vue";
import ArchPluginNode from "./architecture-graph/ArchPluginNode.vue";
import { archGraphActionsKey } from "./architecture-graph/archGraphContext";
import { useArchitectureGraphModel, type CoreModule } from "../composables/useArchitectureGraphModel";
import ArchGraphFitView from "./architecture-graph/ArchGraphFitView.vue";
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

const {
  nodes: builtNodes,
  edges: builtEdges,
  togglePluginExpand,
  hiddenPluginCount,
} = useArchitectureGraphModel();

const nodes = ref([]);
const edges = ref([]);

function syncGraphFromModel() {
  const posMap = new Map(nodes.value.map((n) => [n.id, n.position]));
  nodes.value = builtNodes.value.map((n) => ({
    ...n,
    position: posMap.get(n.id) ?? n.position,
  }));
  edges.value = [...builtEdges.value];
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
      >
        <Background pattern-color="var(--graph-grid-color, rgba(128,128,128,0.2))" :gap="18" />
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
  color: var(--text-accent, var(--accent));
}
.agf-vf {
  height: min(560px, 58vh);
  min-height: 400px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--graph-canvas-bg, var(--bg-elevated));
  overflow: hidden;
}
.agf-vf :deep(.vue-flow) {
  width: 100%;
  height: 100%;
}
.agf-vf :deep(.vue-flow__handle) {
  width: 12px;
  height: 12px;
  border: 2px solid var(--bg-primary);
  background: var(--bg-elevated);
}
.agf-vf :deep(.vue-flow__handle.agn-handle--in) {
  border-color: #4caf50;
}
.agf-vf :deep(.vue-flow__handle.agn-handle--out) {
  border-color: #9c27b0;
}
.agf-vf :deep(.vue-flow__edge-path) {
  stroke-width: 2;
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
}
.agf-swatch--builtin {
  border-color: #4caf50;
}
.agf-swatch--remote {
  border-color: #2196f3;
  border-top-style: dashed;
}
.agf-swatch--directory {
  border-color: #9c27b0;
  border-top-style: dotted;
}
</style>
