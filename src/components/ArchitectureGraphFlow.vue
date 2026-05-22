<script setup lang="ts">
import { applyNodeChanges, VueFlow, type Edge, type Node } from "@vue-flow/core";
import { computed, markRaw, onMounted, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import { MiniMap } from "@vue-flow/minimap";
import ArchAddSlotDialog from "./architecture-graph/ArchAddSlotDialog.vue";
import ArchBackendEdge from "./architecture-graph/ArchBackendEdge.vue";
import ArchBusNode from "./architecture-graph/ArchBusNode.vue";
import ArchComplexNode from "./architecture-graph/ArchComplexNode.vue";
import ArchConnectionLine from "./architecture-graph/ArchConnectionLine.vue";
import ArchGraphFitView from "./architecture-graph/ArchGraphFitView.vue";
import ArchKernelNode from "./architecture-graph/ArchKernelNode.vue";
import ArchModuleNode from "./architecture-graph/ArchModuleNode.vue";
import ArchGroupNode from "./architecture-graph/ArchGroupNode.vue";
import ArchPluginNode from "./architecture-graph/ArchPluginNode.vue";
import { archGraphActionsKey } from "./architecture-graph/archGraphContext";
import { useArchitectureGraphConnections } from "../composables/useArchitectureGraphConnections";
import { useArchitectureGraphLayout } from "../composables/useArchitectureGraphLayout";
import { useArchitectureGraphModel, type CoreModule } from "../composables/useArchitectureGraphModel";
import { BACKEND_COLORS, GRAPH_SURFACE } from "../lib/graphEditorTheme";

const handleInColor = BACKEND_COLORS.builtin.handle;
const handleOutColor = BACKEND_COLORS.directory.handle;
import { useAppToast } from "../composables/useAppToast";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";
import {
  addSlotToRegistry,
  canRemoveSlotKey,
  removeSlotFromRegistry,
  SLOT_REGISTRY_LAST_LLM,
  type SlotRegistryMap,
} from "../lib/slotRegistry";
import {
  clearSessionSlotOverride,
  saveRoleSlotRegistry,
  setSessionPluginBackend,
} from "../utils/tauri-api";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();

const { t } = useI18n();
const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const busy = ref(false);
const showAddSlotWizard = ref(false);
const removeSlotKey = ref("");

const packSlotKeys = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack;
  return pack ? Object.keys(pack).sort() : [];
});

const removeSlotDisabled = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack;
  const key = removeSlotKey.value.trim();
  if (!pack || !key) return true;
  return !canRemoveSlotKey(pack, key);
});

watch(
  packSlotKeys,
  (keys) => {
    if (keys.length && !keys.includes(removeSlotKey.value)) {
      removeSlotKey.value = keys[0];
    }
  },
  { immediate: true },
);

const graphLayout = useArchitectureGraphLayout();

const {
  nodes: builtNodes,
  edges: builtEdges,
  usesBlueprint,
  togglePluginExpand,
  toggleGroupCollapse,
  hiddenPluginCount,
} = useArchitectureGraphModel();

const nodes = ref<Node[]>([]);
const edges = ref<Edge[]>([]);

const graphConnections = useArchitectureGraphConnections(nodes, edges);

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

function onEdgesChangeBlueprint() {
  /* v2：边由 slot_registry 派生，忽略 Vue Flow 边变更 */
}

watch(builtNodes, syncGraphFromModel, { deep: true });
watch(builtEdges, syncGraphFromModel, { deep: true });
watch(
  () => roleStore.roleInfo.pluginBackendsEffective,
  syncGraphFromModel,
  { deep: true },
);
watch(
  () => roleStore.roleInfo.slotRegistryEffective,
  syncGraphFromModel,
  { deep: true },
);
watch(
  () => roleStore.roleInfo.slotSessionOverriddenKeys,
  syncGraphFromModel,
  { deep: true },
);
watch(
  () => roleStore.roleInfo.blueprintGroupsPack,
  syncGraphFromModel,
  { deep: true },
);

const nodeTypes = {
  archKernel: markRaw(ArchKernelNode),
  archBus: markRaw(ArchBusNode),
  archGroup: markRaw(ArchGroupNode),
  archModule: markRaw(ArchModuleNode),
  archPlugin: markRaw(ArchPluginNode),
  archComplex: markRaw(ArchComplexNode),
};

const edgeTypes = {
  archBackend: markRaw(ArchBackendEdge),
};

provide(archGraphActionsKey, {
  busy: () => busy.value,
  usesBlueprint: () => usesBlueprint.value,
  onBackendChange: async (targetKey: string, value: string) => {
    busy.value = true;
    try {
      if (usesBlueprint.value) {
        if (value === "__pack_default__") {
          const info = await clearSessionSlotOverride(
            roleStore.currentRoleId,
            targetKey,
          );
          roleStore.applyRoleInfo(info);
          return;
        }
        const pack = roleStore.roleInfo.slotRegistryPack;
        if (!pack?.[targetKey]) {
          showToast("error", t("pluginWorkbench.graph.connectUnknownPort"));
          return;
        }
        const next: SlotRegistryMap = {
          ...pack,
          [targetKey]: { ...pack[targetKey], backend: value },
        };
        let info = await saveRoleSlotRegistry(roleStore.currentRoleId, next);
        roleStore.applyRoleInfo(info);
        info = await clearSessionSlotOverride(roleStore.currentRoleId, targetKey);
        roleStore.applyRoleInfo(info);
      } else {
        const backend = value === "__pack_default__" ? null : value;
        const info = await setSessionPluginBackend(
          roleStore.currentRoleId,
          targetKey as CoreModule,
          backend,
        );
        roleStore.applyRoleInfo(info);
      }
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    } finally {
      busy.value = false;
    }
  },
  onClearSlotOverride: async (slotKey: string) => {
    busy.value = true;
    try {
      const info = await clearSessionSlotOverride(roleStore.currentRoleId, slotKey);
      roleStore.applyRoleInfo(info);
      showToast("success", t("pluginWorkbench.graph.resetSlotDefaultDone"));
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    } finally {
      busy.value = false;
    }
  },
  onFocusPlugin: (id: string) => emit("focus-plugin", id),
  onToggleExpand: (targetKey: string) => {
    if (hiddenPluginCount(targetKey) > 0) togglePluginExpand(targetKey);
  },
  onToggleGroupCollapse: (groupId: string) => {
    toggleGroupCollapse(groupId);
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

async function persistPackRegistry(next: SlotRegistryMap) {
  busy.value = true;
  try {
    const info = await saveRoleSlotRegistry(roleStore.currentRoleId, next);
    roleStore.applyRoleInfo(info);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
    throw e;
  } finally {
    busy.value = false;
  }
}

function openAddSlotWizard() {
  showAddSlotWizard.value = true;
}

async function onAddSlotConfirm(slotType: string, label: string) {
  const pack = roleStore.roleInfo.slotRegistryPack;
  if (!pack) return;
  try {
    const { registry, key } = addSlotToRegistry(pack, slotType, label);
    await persistPackRegistry(registry);
    removeSlotKey.value = key;
    showAddSlotWizard.value = false;
    showToast("success", t("pluginWorkbench.graph.addSlotDone"));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onRemoveSlot() {
  const pack = roleStore.roleInfo.slotRegistryPack;
  const key = removeSlotKey.value.trim();
  if (!pack || !key || !pack[key]) return;
  if (!canRemoveSlotKey(pack, key)) {
    showToast("error", t("pluginWorkbench.graph.removeSlotLastLlm"));
    return;
  }
  if (!window.confirm(t("pluginWorkbench.graph.removeSlotConfirm", { key }))) {
    return;
  }
  try {
    const next = removeSlotFromRegistry(pack, key);
    await persistPackRegistry(next);
    await clearSessionSlotOverride(roleStore.currentRoleId, key).catch(() => undefined);
    showToast("success", t("pluginWorkbench.graph.removeSlotDone"));
  } catch (e) {
    if (e instanceof Error && e.message === SLOT_REGISTRY_LAST_LLM) {
      showToast("error", t("pluginWorkbench.graph.removeSlotLastLlm"));
    } else {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }
}

onMounted(syncGraphFromModel);
</script>

<template>
  <div class="agf-root">
    <p class="agf-lead">
      {{
        usesBlueprint
          ? t("pluginWorkbench.graph.leadBlueprint")
          : t("pluginWorkbench.graph.lead")
      }}
    </p>
    <p class="agf-vendor">
      {{ t("pluginWorkbench.graph.poweredBy") }}
      <a href="https://vueflow.dev/" target="_blank" rel="noopener noreferrer">Vue Flow</a>
      · {{ t("pluginWorkbench.graph.comfyRef") }}
    </p>

    <div class="agf-toolbar">
      <button type="button" class="agf-tb-btn" @click="onResetLayout">
        {{ t("pluginWorkbench.graph.resetLayout") }}
      </button>
      <template v-if="usesBlueprint">
        <button
          type="button"
          class="agf-tb-btn"
          :disabled="busy || !roleStore.roleInfo.slotRegistryPack"
          @click="openAddSlotWizard"
        >
          {{ t("pluginWorkbench.graph.addSlot") }}
        </button>
        <label v-if="packSlotKeys.length" class="agf-tb-label">
          {{ t("pluginWorkbench.graph.removeSlotKey") }}
          <select v-model="removeSlotKey" class="agf-tb-select" :disabled="busy">
            <option v-for="k in packSlotKeys" :key="k" :value="k">{{ k }}</option>
          </select>
        </label>
        <button
          v-if="packSlotKeys.length"
          type="button"
          class="agf-tb-btn agf-tb-btn--danger"
          :disabled="busy || removeSlotDisabled"
          :title="removeSlotDisabled ? t('pluginWorkbench.graph.removeSlotLastLlm') : undefined"
          @click="onRemoveSlot"
        >
          {{ t("pluginWorkbench.graph.removeSlot") }}
        </button>
      </template>
      <span class="agf-tb-hint">{{ t("pluginWorkbench.graph.resizeHint") }}</span>
    </div>

    <ArchAddSlotDialog
      :open="showAddSlotWizard"
      :busy="busy"
      @close="showAddSlotWizard = false"
      @confirm="onAddSlotConfirm"
    />

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
        :connection-mode="graphConnections.connectionMode"
        :min-zoom="0.35"
        :max-zoom="1.8"
        :nodes-draggable="true"
        :nodes-connectable="!usesBlueprint"
        :edges-updatable="!usesBlueprint"
        :elements-selectable="true"
        :is-valid-connection="usesBlueprint ? () => false : graphConnections.isValidConnection"
        :fit-view-on-init="false"
        :default-viewport="{ zoom: 0.85 }"
        @nodes-change="onNodesChange"
        @edges-change="usesBlueprint ? onEdgesChangeBlueprint : graphConnections.onEdgesChange"
        @connect="usesBlueprint ? onEdgesChangeBlueprint : graphConnections.onConnect"
        @edge-update="usesBlueprint ? onEdgesChangeBlueprint : graphConnections.onEdgeUpdate"
        @connect-start="usesBlueprint ? onEdgesChangeBlueprint : graphConnections.onConnectStart"
        @connect-end="usesBlueprint ? onEdgesChangeBlueprint : graphConnections.onConnectEnd"
      >
        <template #connection-line="lineProps">
          <ArchConnectionLine v-bind="lineProps" />
        </template>
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
      <span class="agf-legend-hint">
        {{
          usesBlueprint
            ? t("pluginWorkbench.graph.connectHintBlueprint")
            : t("pluginWorkbench.graph.connectHint")
        }}
      </span>
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
.agf-tb-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.agf-tb-select {
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, #333);
  background: var(--surface-elevated, #1a1a1a);
  color: var(--text-primary);
}
.agf-tb-btn--danger {
  color: var(--danger, #e55);
}
.agf-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.agf-tb-btn {
  font-size: 12px;
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, #333);
  background: var(--surface-elevated, #1a1a1a);
  color: var(--text-primary);
  cursor: pointer;
}
.agf-tb-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.agf-tb-hint {
  font-size: 11px;
  color: var(--text-secondary);
}
.agf-vf {
  height: min(62vh, 520px);
  min-height: 280px;
  border-radius: 10px;
  border: 1px solid var(--border-subtle, #2a2a2a);
  overflow: hidden;
}
.agf-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 14px;
  font-size: 11px;
  color: var(--text-secondary);
}
.agf-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.agf-swatch {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}
.agf-swatch--builtin {
  background: #6b9bd1;
}
.agf-swatch--remote {
  background: #c9a227;
}
.agf-swatch--directory {
  background: #7dce82;
}
.agf-legend-hint {
  flex: 1 1 100%;
  font-size: 10px;
  opacity: 0.85;
}
</style>
