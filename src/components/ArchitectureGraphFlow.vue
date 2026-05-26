<script setup lang="ts">
import type { Edge, Node } from '@vue-flow/core'
import type { CoreModule } from '../composables/useArchitectureGraphModel'
import type { SlotRegistryMap } from '../lib/slotRegistry'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { applyNodeChanges, VueFlow } from '@vue-flow/core'
import { MiniMap } from '@vue-flow/minimap'
import { computed, markRaw, onMounted, provide, ref, watch } from 'vue'
import type { ExpertRoutingDoc } from '../api/role/expert'
import { getExpertRouting } from '../api/role/expert'
import ExpertFlowPanel from './architecture-graph/ExpertFlowPanel.vue'
import { expertLlmHighlights } from '../lib/expertRoutingGraph'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '../composables/useAppToast'
import { useArchitectureGraphLayout } from '../composables/useArchitectureGraphLayout'
import { useArchitectureGraphModel } from '../composables/useArchitectureGraphModel'
import { patchSlotRegistryBackend } from '../lib/archGraphSlotBackend'
import { BACKEND_COLORS, GRAPH_SURFACE } from '../lib/graphEditorTheme'
import {
  addSlotToRegistry,
  canRemoveSlotKey,
  removeSlotFromRegistry,
  SLOT_REGISTRY_LAST_LLM,

} from '../lib/slotRegistry'
import { usePluginStore } from '../stores/pluginStore'
import { usePluginTraceStore } from '../stores/pluginTraceStore'
import { usePluginTraceStore } from '../stores/pluginTraceStore'
import { useRoleStore } from '../stores/roleStore'
import {
  clearSessionSlotOverride,
  saveRoleSlotRegistry,
  setSessionPluginBackend,
  setSessionSlotOverride,
} from '../api'
import ArchAddSlotDialog from './architecture-graph/ArchAddSlotDialog.vue'
import ArchBackendEdge from './architecture-graph/ArchBackendEdge.vue'
import ArchBusNode from './architecture-graph/ArchBusNode.vue'
import ArchComplexNode from './architecture-graph/ArchComplexNode.vue'
import { archGraphActionsKey } from './architecture-graph/archGraphContext'
import ArchGraphFitView from './architecture-graph/ArchGraphFitView.vue'
import ArchGroupNode from './architecture-graph/ArchGroupNode.vue'
import ArchKernelNode from './architecture-graph/ArchKernelNode.vue'
import ArchModuleNode from './architecture-graph/ArchModuleNode.vue'
import ArchPluginNode from './architecture-graph/ArchPluginNode.vue'
import ArchRemoveSlotDialog from './architecture-graph/ArchRemoveSlotDialog.vue'

const emit = defineEmits<{
  'focus-plugin': [pluginId: string]
}>()
const handleInColor = BACKEND_COLORS.builtin.handle
const handleOutColor = BACKEND_COLORS.directory.handle

const { t } = useI18n()
const roleStore = useRoleStore()
const pluginStore = usePluginStore()
const traceStore = usePluginTraceStore()
const { showToast } = useAppToast()
const busy = ref(false)
const showAddSlotWizard = ref(false)
const showRemoveSlotDialog = ref(false)
const removeSlotKey = ref('')

const packSlotKeys = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack
  return pack ? Object.keys(pack).sort() : []
})

const removeSlotDisabled = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack
  const key = removeSlotKey.value.trim()
  if (!pack || !key)
    return true
  return !canRemoveSlotKey(pack, key)
})

watch(
  packSlotKeys,
  (keys) => {
    if (keys.length && !keys.includes(removeSlotKey.value)) {
      removeSlotKey.value = keys[0]
    }
  },
  { immediate: true },
)

const graphLayout = useArchitectureGraphLayout()

const {
  nodes: builtNodes,
  edges: builtEdges,
  usesBlueprint,
  togglePluginExpand,
  toggleGroupCollapse,
  expandAllDirectoryPlugins,
  hiddenPluginCount,
  dualCoreEnabled,
  pipelineExperimentalActions,
} = useArchitectureGraphModel()

const nodes = ref<Node[]>([])
const edges = ref<Edge[]>([])
const expertLlmHints = ref<Map<string, string>>(new Map())
const expertRoutingDoc = ref<ExpertRoutingDoc | null>(null)

const previewUserMessage = ref('')

async function refreshExpertHighlights() {
  const roleId = roleStore.currentRoleId
  if (!roleId) {
    expertLlmHints.value = new Map()
    expertRoutingDoc.value = null
    return
  }
  try {
    const doc = await getExpertRouting(roleId)
    expertRoutingDoc.value = doc
    expertLlmHints.value = expertLlmHighlights(
      doc ?? undefined,
      roleStore.roleInfo.slotRegistryEffective ?? roleStore.roleInfo.slotRegistryPack,
    )
  }
  catch {
    expertLlmHints.value = new Map()
    expertRoutingDoc.value = null
  }
  syncGraphFromModel()
}

function syncGraphFromModel() {
  nodes.value = builtNodes.value.map((n) => {
    const applied = graphLayout.applyToNode(n.id, n.type, n.position.x, n.position.y)
    const expertHint = expertLlmHints.value.get(n.id)
    const data = n.data && typeof n.data === 'object'
      ? {
          ...(n.data as Record<string, unknown>),
          ...(expertHint
            ? { expertHighlight: true, expertTriggerHint: expertHint }
            : {}),
        }
      : n.data
    return {
      ...n,
      data,
      position: { x: applied.x, y: applied.y },
      width: applied.width,
      height: applied.height,
      style: applied.style,
    }
  })
  edges.value = [...builtEdges.value]
}

function onNodesChange(changes: unknown[]) {
  const list = changes as Array<{
    type: string
    id?: string
    position?: { x: number, y: number }
    dimensions?: { width: number, height: number }
    dragging?: boolean
  }>
  nodes.value = applyNodeChanges(list, nodes.value)
  let dirty = false
  for (const c of list) {
    if (c.type === 'position' && c.id && c.position && c.dragging === false) {
      const base = builtNodes.value.find(b => b.id === c.id)
      if (base) {
        graphLayout.setPosition(
          c.id,
          c.position.x - base.position.x,
          c.position.y - base.position.y,
        )
        dirty = true
      }
    }
    if (c.type === 'dimensions' && c.id && c.dimensions) {
      graphLayout.setSize(c.id, Math.round(c.dimensions.width), Math.round(c.dimensions.height))
      dirty = true
    }
  }
  if (dirty)
    graphLayout.save()
}

function onResetLayout() {
  graphLayout.reset()
  syncGraphFromModel()
  showToast('success', t('pluginWorkbench.graph.layoutResetDone'))
}

function applyAutoWiringLayers(includePlugins: boolean) {
  const registry = roleStore.roleInfo.slotRegistryEffective
  if (registry && Object.keys(registry).length > 0 && includePlugins) {
    expandAllDirectoryPlugins(registry)
  }
  syncGraphFromModel()
}

function onAutoWireLayer(layer: 1 | 2 | 3) {
  applyAutoWiringLayers(layer === 3)
  const key
    = layer === 1
      ? 'pluginWorkbench.graph.autoWireLayer1Done'
      : layer === 2
        ? 'pluginWorkbench.graph.autoWireLayer2Done'
        : 'pluginWorkbench.graph.autoWireLayer3Done'
  showToast('success', t(key))
}

let lastAutoWireRegistrySig = ''
watch(
  () => {
    const reg = roleStore.roleInfo.slotRegistryEffective
    if (!usesBlueprint.value || !reg)
      return ''
    return Object.entries(reg)
      .map(([k, e]) => `${k}:${e.type}:${e.position}:${e.backend}`)
      .sort()
      .join('|')
  },
  (sig) => {
    if (!sig || sig === lastAutoWireRegistrySig)
      return
    lastAutoWireRegistrySig = sig
    const reg = roleStore.roleInfo.slotRegistryEffective
    if (reg)
      applyAutoWiringLayers(true)
  },
  { immediate: true },
)

function onEdgesChangeBlueprint() {
  /* v2：边由 slot_registry 派生，忽略 Vue Flow 边变更 */
}

watch(builtNodes, syncGraphFromModel, { deep: true })
watch(builtEdges, syncGraphFromModel, { deep: true })
watch(
  () => roleStore.roleInfo.pluginBackendsEffective,
  syncGraphFromModel,
  { deep: true },
)
watch(
  () => roleStore.roleInfo.slotRegistryEffective,
  syncGraphFromModel,
  { deep: true },
)
watch(
  () => roleStore.roleInfo.slotSessionOverriddenKeys,
  syncGraphFromModel,
  { deep: true },
)
watch(
  () => roleStore.roleInfo.blueprintGroupsPack,
  syncGraphFromModel,
  { deep: true },
)

const nodeTypes = {
  archKernel: markRaw(ArchKernelNode),
  archBus: markRaw(ArchBusNode),
  archGroup: markRaw(ArchGroupNode),
  archModule: markRaw(ArchModuleNode),
  archPlugin: markRaw(ArchPluginNode),
  archComplex: markRaw(ArchComplexNode),
}

const edgeTypes = {
  archBackend: markRaw(ArchBackendEdge),
}

provide(archGraphActionsKey, {
  busy: () => busy.value,
  usesBlueprint: () => usesBlueprint.value,
  onBackendChange: async (targetKey: string, value: string) => {
    if (usesBlueprint.value)
      return
    busy.value = true
    try {
      const backend = value === '__pack_default__' ? null : value
      const info = await setSessionPluginBackend(
        roleStore.currentRoleId,
        targetKey as CoreModule,
        backend,
      )
      roleStore.applyRoleInfo(info)
    }
    catch (e) {
      showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      busy.value = false
    }
  },
  onApplySessionOverride: async (slotKey: string, backend: string) => {
    busy.value = true
    try {
      const info = await setSessionSlotOverride(roleStore.currentRoleId, slotKey, {
        backend,
      })
      roleStore.applyRoleInfo(info)
      showToast('success', t('pluginWorkbench.graph.applySessionDone'))
    }
    catch (e) {
      showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      busy.value = false
    }
  },
  onApplyPackDefault: async (slotKey: string, backend: string) => {
    const pack = roleStore.roleInfo.slotRegistryPack
    if (!pack?.[slotKey]) {
      showToast('error', t('pluginWorkbench.graph.connectUnknownPort'))
      return
    }
    busy.value = true
    try {
      const next = patchSlotRegistryBackend(pack, slotKey, backend)
      let info = await saveRoleSlotRegistry(roleStore.currentRoleId, next)
      roleStore.applyRoleInfo(info)
      info = await clearSessionSlotOverride(roleStore.currentRoleId, slotKey)
      roleStore.applyRoleInfo(info)
      showToast('success', t('pluginWorkbench.graph.applyPackDone'))
    }
    catch (e) {
      showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      busy.value = false
    }
  },
  onClearSlotOverride: async (slotKey: string) => {
    busy.value = true
    try {
      const info = await clearSessionSlotOverride(roleStore.currentRoleId, slotKey)
      roleStore.applyRoleInfo(info)
      showToast('success', t('pluginWorkbench.graph.resetSlotDefaultDone'))
    }
    catch (e) {
      showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      busy.value = false
    }
  },
  onFocusPlugin: (id: string) => emit('focus-plugin', id),
  onToggleExpand: (targetKey: string) => {
    if (hiddenPluginCount(targetKey) > 0)
      togglePluginExpand(targetKey)
  },
  onToggleGroupCollapse: (groupId: string) => {
    toggleGroupCollapse(groupId)
  },
  onTogglePluginDisabled: (id: string) => {
    try {
      pluginStore.setPluginDisabled(id, !pluginStore.isPluginDisabled(id))
      showToast('success', t('pluginWorkbench.graph.ctxToggled'))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  },
  onUninstallPlugin: async (id: string) => {
    try {
      await pluginStore.uninstallPluginFromGitIndex(id)
      showToast('success', t('pluginWorkbench.graph.ctxUninstalled', { id }))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  },
})

async function persistPackRegistry(next: SlotRegistryMap) {
  busy.value = true
  try {
    const info = await saveRoleSlotRegistry(roleStore.currentRoleId, next)
    roleStore.applyRoleInfo(info)
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
    throw e
  }
  finally {
    busy.value = false
  }
}

function openAddSlotWizard() {
  showAddSlotWizard.value = true
}

async function onAddSlotConfirm(slotType: string, label: string) {
  const pack = roleStore.roleInfo.slotRegistryPack
  if (!pack)
    return
  try {
    const { registry, key } = addSlotToRegistry(pack, slotType, label)
    await persistPackRegistry(registry)
    removeSlotKey.value = key
    showAddSlotWizard.value = false
    showToast('success', t('pluginWorkbench.graph.addSlotDone'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

function openRemoveSlotDialog() {
  const pack = roleStore.roleInfo.slotRegistryPack
  const key = removeSlotKey.value.trim()
  if (!pack || !key || !pack[key])
    return
  if (!canRemoveSlotKey(pack, key)) {
    showToast('error', t('pluginWorkbench.graph.removeSlotLastLlm'))
    return
  }
  showRemoveSlotDialog.value = true
}

async function onRemoveSlotConfirm() {
  const pack = roleStore.roleInfo.slotRegistryPack
  const key = removeSlotKey.value.trim()
  if (!pack || !key || !pack[key])
    return
  showRemoveSlotDialog.value = false
  try {
    const next = removeSlotFromRegistry(pack, key)
    await persistPackRegistry(next)
    await clearSessionSlotOverride(roleStore.currentRoleId, key).catch(() => undefined)
    showToast('success', t('pluginWorkbench.graph.removeSlotDone'))
  }
  catch (e) {
    if (e instanceof Error && e.message === SLOT_REGISTRY_LAST_LLM) {
      showToast('error', t('pluginWorkbench.graph.removeSlotLastLlm'))
    }
    else {
      showToast('error', e instanceof Error ? e.message : String(e))
    }
  }
}

watch(
  () => traceStore.focusSlotKey,
  (key) => {
    if (!key || !usesBlueprint.value)
      return
    nodes.value = nodes.value.map(n => ({
      ...n,
      selected: n.id === key,
    }))
    traceStore.clearFocusArchSlot()
  },
)

onMounted(() => {
  syncGraphFromModel()
  void refreshExpertHighlights()
})

watch(
  () => roleStore.currentRoleId,
  () => void refreshExpertHighlights(),
)
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

    <div v-if="dualCoreEnabled" class="agf-dual-core-banner" role="status">
      <span class="agf-dual-core-badge">{{ t("pluginWorkbench.graph.dualCoreBadge") }}</span>
      <div class="agf-dual-core-lines">
        <p>
          <strong>{{ t("pluginWorkbench.graph.dualCoreStable") }}</strong>
          {{ t("pluginWorkbench.graph.dualCoreStableHint") }}
        </p>
        <p v-if="pipelineExperimentalActions.length">
          <strong>{{ t("pluginWorkbench.graph.dualCoreExperimental") }}</strong>
          {{ pipelineExperimentalActions.join(" → ") }}
        </p>
        <p v-else class="agf-dual-core-muted">
          {{ t("pluginWorkbench.graph.dualCoreExperimentalEmpty") }}
        </p>
      </div>
    </div>

    <div class="agf-toolbar">
      <button type="button" class="agf-tb-btn" @click="onResetLayout">
        {{ t("pluginWorkbench.graph.resetLayout") }}
      </button>
      <template v-if="usesBlueprint">
        <button
          type="button"
          class="agf-tb-btn"
          :disabled="busy"
          :title="t('pluginWorkbench.graph.autoWireLayer1Hint')"
          @click="onAutoWireLayer(1)"
        >
          {{ t("pluginWorkbench.graph.autoWireLayer1") }}
        </button>
        <button
          type="button"
          class="agf-tb-btn"
          :disabled="busy"
          :title="t('pluginWorkbench.graph.autoWireLayer2Hint')"
          @click="onAutoWireLayer(2)"
        >
          {{ t("pluginWorkbench.graph.autoWireLayer2") }}
        </button>
        <button
          type="button"
          class="agf-tb-btn"
          :disabled="busy"
          :title="t('pluginWorkbench.graph.autoWireLayer3Hint')"
          @click="onAutoWireLayer(3)"
        >
          {{ t("pluginWorkbench.graph.autoWireLayer3") }}
        </button>
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
          @click="openRemoveSlotDialog"
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
    <ArchRemoveSlotDialog
      :open="showRemoveSlotDialog"
      :slot-key="removeSlotKey"
      :busy="busy"
      @close="showRemoveSlotDialog = false"
      @confirm="onRemoveSlotConfirm"
    />

    <div
      class="agf-vf"
      role="application"
      :aria-label="t('pluginWorkbench.graph.canvasAria')"
    >
      <ExpertFlowPanel
        :doc="expertRoutingDoc"
        :scene-id="roleStore.roleInfo.current_scene ?? ''"
        :user-message="previewUserMessage"
      />
      <VueFlow
        v-model:nodes="nodes"
        v-model:edges="edges"
        :node-types="nodeTypes"
        :edge-types="edgeTypes"
        :min-zoom="0.35"
        :max-zoom="1.8"
        :nodes-draggable="true"
        :nodes-connectable="false"
        :edges-updatable="false"
        :elements-selectable="true"
        :fit-view-on-init="false"
        :default-viewport="{ zoom: 0.85 }"
        @nodes-change="onNodesChange"
        @edges-change="onEdgesChangeBlueprint"
        @connect="onEdgesChangeBlueprint"
        @edge-update="onEdgesChangeBlueprint"
        @connect-start="onEdgesChangeBlueprint"
        @connect-end="onEdgesChangeBlueprint"
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
.agf-dual-core-banner {
  margin: 8px 0 10px;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--accent) 45%, transparent);
  background: color-mix(in srgb, var(--accent) 12%, var(--surface-elevated, #1a1a1a));
}
.agf-dual-core-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--accent);
  margin-bottom: 6px;
}
.agf-dual-core-lines p {
  margin: 4px 0;
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
}
.agf-dual-core-muted {
  opacity: 0.85;
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
  position: relative;
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
