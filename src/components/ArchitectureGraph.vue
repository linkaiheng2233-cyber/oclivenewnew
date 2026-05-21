<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useGraphCanvas } from "../composables/useGraphCanvas";
import { useGraphNodeLayout } from "../composables/useGraphNodeLayout";
import {
  BACKEND_COLORS,
  edgeDash,
  normalizeBackendKind,
  type BackendKind,
} from "../lib/graphEditorTheme";
import {
  layoutOnRing,
  linkBusSlotToModule,
  linkKernelToRect,
  linkModuleToPlugin,
  pointOnRay,
  type RectBox,
} from "../lib/radialGraphLayout";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";
import { useAppToast } from "../composables/useAppToast";
import { setSessionPluginBackend } from "../utils/tauri-api";
import type { DirectoryPluginCatalogEntry } from "../utils/tauri-api";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();

const WORLD_W = 1040;
const WORLD_H = 720;
const NODE_W = 212;
const NODE_H = 108;
const PLUGIN_W = 176;
const PLUGIN_H = 68;
const HUB_CX = WORLD_W / 2;
const HUB_CY = WORLD_H / 2 - 6;
const KERNEL_OUTER_R = 268;
const MODULE_RING = 178;
const PLUGIN_INSET = 80;
const KERNEL_R = 58;
const BUS_W = 236;
const BUS_H = 88;
const BUS_ID = "__facility_bus__";
const KERNEL_ID = "__kernel__";
const COMPLEX_ID = "__complex_emotion__";

const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const { t } = useI18n();

const {
  viewportRef,
  transformStyle,
  gridStyle,
  spaceHeld,
  panning,
  scalePercent,
  zoomIn,
  zoomOut,
  resetView,
  fitWorld,
  focusPoint,
  onWheel,
  scale,
  onPointerDown,
  onPointerMove,
  onPointerUp,
} = useGraphCanvas({ worldWidth: WORLD_W, worldHeight: WORLD_H });

const {
  offsets: nodeOffsets,
  save: persistNodeOffsetsFromStore,
  reset: resetNodeLayoutStore,
  load: loadNodeOffsetsFromStore,
} = useGraphNodeLayout();

const busy = ref(false);
const draggingNode = ref<{
  id: string;
  startX: number;
  startY: number;
  origDx: number;
  origDy: number;
} | null>(null);
const selectedModule = ref<string | null>(null);
const hoveredModule = ref<string | null>(null);
const expandedPlugins = ref<Record<string, boolean>>({});
const ctxMenu = ref<{ x: number; y: number; pluginId: string } | null>(null);

type CoreModule = "memory" | "emotion" | "event" | "prompt" | "llm" | "agent";

const coreModules: {
  key: CoreModule;
  labelKey: string;
  icon: string;
  options: string[];
}[] = [
  { key: "memory", labelKey: "pluginWorkbench.graph.memory", icon: "🧠", options: ["builtin", "builtin_v2", "remote", "local", "directory"] },
  { key: "emotion", labelKey: "pluginWorkbench.graph.emotion", icon: "💭", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "event", labelKey: "pluginWorkbench.graph.event", icon: "⚡", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "prompt", labelKey: "pluginWorkbench.graph.prompt", icon: "📝", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "llm", labelKey: "pluginWorkbench.graph.llm", icon: "🤖", options: ["ollama", "remote", "directory"] },
  { key: "agent", labelKey: "pluginWorkbench.graph.agent", icon: "🛠", options: ["builtin", "remote", "directory"] },
];

const kernelBase = { cx: HUB_CX, cy: HUB_CY - KERNEL_OUTER_R, r: KERNEL_R };

const kernelLayout = computed(() => {
  const o = nodeOffsets.value[KERNEL_ID] ?? { dx: 0, dy: 0 };
  return {
    cx: kernelBase.cx + o.dx,
    cy: kernelBase.cy + o.dy,
    r: KERNEL_R,
    x: kernelBase.cx + o.dx - KERNEL_R,
    y: kernelBase.cy + o.dy - KERNEL_R,
  };
});

const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
const pluginBackendsSessionOverride = computed(
  () => roleStore.roleInfo.pluginBackendsSessionOverride,
);

type SlotLayout = {
  key: CoreModule;
  x: number;
  y: number;
  cx: number;
  cy: number;
  angle: number;
  labelKey: string;
  icon: string;
  options: string[];
};

const facilityBusBase = {
  x: HUB_CX - BUS_W / 2,
  y: HUB_CY - BUS_H / 2,
  cx: HUB_CX,
  cy: HUB_CY,
  w: BUS_W,
  h: BUS_H,
};

const facilityBus = computed(() => {
  const o = nodeOffsets.value[BUS_ID] ?? { dx: 0, dy: 0 };
  return {
    ...facilityBusBase,
    x: facilityBusBase.x + o.dx,
    y: facilityBusBase.y + o.dy,
    cx: facilityBusBase.cx + o.dx,
    cy: facilityBusBase.cy + o.dy,
  };
});

const slotLayouts = computed<SlotLayout[]>(() =>
  coreModules.map((m, i) => {
    const ring = layoutOnRing(
      HUB_CX,
      HUB_CY,
      MODULE_RING,
      i,
      coreModules.length,
      NODE_W,
      NODE_H,
    );
    const o = nodeOffsets.value[m.key] ?? { dx: 0, dy: 0 };
    return {
      ...m,
      x: ring.x + o.dx,
      y: ring.y + o.dy,
      cx: ring.cx + o.dx,
      cy: ring.cy + o.dy,
      angle: ring.angle,
    };
  }),
);

const complexLayout = computed(() => {
  const angle = Math.PI / 2 + 0.18;
  const cx0 = HUB_CX + Math.cos(angle) * (MODULE_RING * 0.78);
  const cy0 = HUB_CY + Math.sin(angle) * (MODULE_RING * 0.78);
  const o = nodeOffsets.value[COMPLEX_ID] ?? { dx: 0, dy: 0 };
  return {
    x: cx0 - 100 + o.dx,
    y: cy0 - NODE_H / 2 + o.dy,
    cx: cx0 + o.dx,
    cy: cy0 + o.dy,
    angle,
    w: 200,
  };
});

function effectiveBackend(key: CoreModule): string {
  return String(pluginBackendsEffective.value[key] ?? "");
}

function backendKind(key: CoreModule): BackendKind {
  return normalizeBackendKind(effectiveBackend(key));
}

function catalogEntry(id: string): DirectoryPluginCatalogEntry | undefined {
  return pluginStore.catalog.find((c) => c.id === id);
}

/** directory 槽位关联的插件 id 列表（主绑定优先，其余来自全局 directory_plugins 去重） */
function directoryPluginIds(key: CoreModule): string[] {
  const dp = pluginBackendsEffective.value.directory_plugins;
  const primary = dp?.[key]?.trim() ?? "";
  const set = new Set<string>();
  if (primary) set.add(primary);
  for (const mod of coreModules) {
    const id = dp?.[mod.key]?.trim();
    if (id) set.add(id);
  }
  const list = [...set];
  if (primary) {
    return [primary, ...list.filter((id) => id !== primary)];
  }
  return list;
}

function visiblePluginIds(key: CoreModule): string[] {
  const all = directoryPluginIds(key);
  if (backendKind(key) !== "directory" || all.length === 0) return [];
  if (expandedPlugins.value[key] || all.length <= 1) return all;
  return [all[0]!];
}

function hiddenPluginCount(key: CoreModule): number {
  const all = directoryPluginIds(key);
  return Math.max(0, all.length - 1);
}

type PluginLayout = {
  pid: string;
  moduleKey: CoreModule;
  x: number;
  y: number;
  cx: number;
  cy: number;
  angle: number;
  index: number;
};

const pluginLayouts = computed<PluginLayout[]>(() => {
  const out: PluginLayout[] = [];
  for (const sl of slotLayouts.value) {
    if (backendKind(sl.key) !== "directory") continue;
    const plugins = visiblePluginIds(sl.key);
    plugins.forEach((pid, j) => {
      const dist = PLUGIN_INSET + j * (PLUGIN_H + 12);
      const center = pointOnRay(sl.cx, sl.cy, sl.angle + Math.PI, dist);
      const pidKey = `plugin:${pid}`;
      const o = nodeOffsets.value[pidKey] ?? { dx: 0, dy: 0 };
      out.push({
        pid,
        moduleKey: sl.key,
        x: center.x - PLUGIN_W / 2 + o.dx,
        y: center.y - PLUGIN_H / 2 + o.dy,
        cx: center.x + o.dx,
        cy: center.y + o.dy,
        angle: sl.angle,
        index: j,
      });
    });
  }
  return out;
});

type EdgeDef = {
  id: string;
  d: string;
  kind: BackendKind;
  toModule?: CoreModule;
  animated?: boolean;
};

function moduleBox(sl: SlotLayout): RectBox {
  return { x: sl.x, y: sl.y, w: NODE_W, h: NODE_H };
}

function pluginBox(pl: PluginLayout): RectBox {
  return { x: pl.x, y: pl.y, w: PLUGIN_W, h: PLUGIN_H };
}

const edges = computed<EdgeDef[]>(() => {
  const out: EdgeDef[] = [];
  const k = kernelLayout.value;
  const bus = facilityBus.value;
  const busBox: RectBox = { x: bus.x, y: bus.y, w: bus.w, h: bus.h };
  const modules = slotLayouts.value;
  const n = modules.length;

  out.push({
    id: "kernel-bus",
    d: linkKernelToRect(k.cx, k.cy, k.r, busBox),
    kind: "builtin",
  });

  modules.forEach((sl, i) => {
    const kind = backendKind(sl.key);
    out.push({
      id: `bus-${sl.key}`,
      d: linkBusSlotToModule(busBox, moduleBox(sl), i, n),
      kind,
      toModule: sl.key,
      animated: kind === "remote",
    });
  });

  for (const pl of pluginLayouts.value) {
    const sl = modules.find((s) => s.key === pl.moduleKey);
    if (!sl) continue;
    out.push({
      id: `p-${pl.moduleKey}-${pl.pid}`,
      d: linkModuleToPlugin(moduleBox(sl), pluginBox(pl)),
      kind: "directory",
      animated: false,
    });
  }

  const cx = complexLayout.value;
  const complexBox: RectBox = { x: cx.x, y: cx.y, w: cx.w, h: NODE_H };
  out.push({
    id: "bus-complex",
    d: linkBusSlotToModule(busBox, complexBox, n - 1, n + 1),
    kind: "builtin",
  });

  return out;
});

function edgeStrokeWidth(edge: EdgeDef): number {
  if (edge.toModule && hoveredModule.value === edge.toModule) return 3;
  return 2;
}

async function onBackendChange(module: CoreModule, ev: Event) {
  const selected = (ev.target as HTMLSelectElement).value;
  const backend = selected === "__pack_default__" ? null : selected;
  busy.value = true;
  try {
    const info = await setSessionPluginBackend(roleStore.currentRoleId, module, backend);
    roleStore.applyRoleInfo(info);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busy.value = false;
  }
}

function togglePluginExpand(key: CoreModule) {
  expandedPlugins.value = {
    ...expandedPlugins.value,
    [key]: !expandedPlugins.value[key],
  };
}

function selectModule(key: string) {
  selectedModule.value = key;
}

function onNodeDblClick(sl: SlotLayout) {
  selectModule(sl.key);
  focusPoint(sl.cx, sl.cy);
}

function onNodeDragStart(e: PointerEvent, id: string): void {
  if (e.button !== 0 || spaceHeld.value) return;
  e.stopPropagation();
  e.preventDefault();
  const o = nodeOffsets.value[id] ?? { dx: 0, dy: 0 };
  draggingNode.value = {
    id,
    startX: e.clientX,
    startY: e.clientY,
    origDx: o.dx,
    origDy: o.dy,
  };
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onViewportPointerMove(e: PointerEvent): void {
  if (draggingNode.value) {
    const s = scale.value;
    const dx = (e.clientX - draggingNode.value.startX) / s;
    const dy = (e.clientY - draggingNode.value.startY) / s;
    nodeOffsets.value = {
      ...nodeOffsets.value,
      [draggingNode.value.id]: {
        dx: draggingNode.value.origDx + dx,
        dy: draggingNode.value.origDy + dy,
      },
    };
    return;
  }
  onPointerMove(e);
}

function onViewportPointerUp(e: PointerEvent): void {
  if (draggingNode.value) {
    draggingNode.value = null;
    persistNodeOffsetsFromStore();
    return;
  }
  onPointerUp();
}

function onResetNodeLayout(): void {
  resetNodeLayoutStore();
}

function onFocusPlugin(id: string) {
  ctxMenu.value = null;
  emit("focus-plugin", id);
}

function openCtx(e: MouseEvent, pluginId: string) {
  e.preventDefault();
  ctxMenu.value = { x: e.clientX, y: e.clientY, pluginId };
}

function closeCtx() {
  ctxMenu.value = null;
}

function togglePluginDisabled(id: string) {
  const disabled = pluginStore.isPluginDisabled(id);
  try {
    pluginStore.setPluginDisabled(id, !disabled);
    showToast("success", t("pluginWorkbench.graph.ctxToggled"));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
  closeCtx();
}

async function uninstallPlugin(id: string) {
  closeCtx();
  try {
    await pluginStore.uninstallPluginFromGitIndex(id);
    showToast("success", t("pluginWorkbench.graph.ctxUninstalled", { id }));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

onMounted(() => {
  loadNodeOffsetsFromStore();
  fitWorld();
});

watch(
  () => pluginStore.panelVisible,
  (v) => {
    if (v) fitWorld();
  },
);
</script>

<template>
  <div class="arch-root" @click="closeCtx">
    <p class="arch-lead">{{ t("pluginWorkbench.graph.lead") }}</p>

    <div
      ref="viewportRef"
      class="arch-viewport"
      :class="{ 'arch-viewport--pan': spaceHeld || panning }"
      role="application"
      :aria-label="t('pluginWorkbench.graph.canvasAria')"
      @wheel="onWheel"
      @pointerdown="onPointerDown"
      @pointermove="onViewportPointerMove"
      @pointerup="onViewportPointerUp"
      @pointercancel="onViewportPointerUp"
      @click.stop
    >
      <div class="arch-canvas-title" @click.stop>
        {{ t("pluginWorkbench.graph.canvasTitle") }}
      </div>

      <div class="arch-toolbar" @click.stop>
        <button type="button" class="arch-tb-btn" :title="t('pluginWorkbench.graph.zoomIn')" @click="zoomIn()">
          +
        </button>
        <button type="button" class="arch-tb-btn" :title="t('pluginWorkbench.graph.zoomOut')" @click="zoomOut()">
          −
        </button>
        <button type="button" class="arch-tb-btn" @click="resetView()">
          {{ t("pluginWorkbench.graph.resetView") }}
        </button>
        <button type="button" class="arch-tb-btn" @click="fitWorld()">
          {{ t("pluginWorkbench.graph.fitAll") }}
        </button>
        <button type="button" class="arch-tb-btn" :title="t('pluginWorkbench.graph.resetLayout')" @click="onResetNodeLayout">
          {{ t("pluginWorkbench.graph.resetLayout") }}
        </button>
        <span class="arch-tb-scale">{{ scalePercent }}</span>
      </div>

      <div class="arch-layer-legend" @click.stop>
        <span class="arch-layer-pill arch-layer-pill--hub">{{ t("pluginWorkbench.graph.layerHub") }}</span>
        <span class="arch-layer-pill arch-layer-pill--facility">{{ t("pluginWorkbench.graph.layerFacility") }}</span>
        <span class="arch-layer-pill arch-layer-pill--plugin">{{ t("pluginWorkbench.graph.layerPlugin") }}</span>
      </div>

      <div class="arch-minimap" aria-hidden="true" @click.stop>
        <svg :viewBox="`0 0 ${WORLD_W} ${WORLD_H}`" class="arch-minimap-svg">
          <circle
            :cx="HUB_CX"
            :cy="HUB_CY"
            :r="KERNEL_OUTER_R"
            class="arch-minimap-ring arch-minimap-ring--outer"
          />
          <circle
            :cx="HUB_CX"
            :cy="HUB_CY"
            :r="MODULE_RING"
            class="arch-minimap-ring"
          />
          <rect
            :x="facilityBus.x"
            :y="facilityBus.y"
            :width="facilityBus.w"
            :height="facilityBus.h"
            class="arch-minimap-bus"
          />
          <rect
            v-for="sl in slotLayouts"
            :key="'mm-' + sl.key"
            :x="sl.x"
            :y="sl.y"
            :width="NODE_W"
            :height="NODE_H"
            class="arch-minimap-node"
          />
          <circle
            :cx="kernelLayout.cx"
            :cy="kernelLayout.cy"
            :r="kernelLayout.r"
            class="arch-minimap-kernel"
          />
        </svg>
      </div>

      <div class="arch-grid" :style="gridStyle" />

      <div class="arch-world" :style="{ transform: transformStyle, width: WORLD_W + 'px', height: WORLD_H + 'px' }">
        <svg
          class="arch-edges"
          :width="WORLD_W"
          :height="WORLD_H"
          aria-hidden="true"
        >
          <defs>
            <filter id="arch-glow-remote" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="2" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <circle
            :cx="HUB_CX"
            :cy="HUB_CY"
            :r="KERNEL_OUTER_R"
            class="arch-orbit arch-orbit--kernel"
          />
          <circle
            :cx="HUB_CX"
            :cy="HUB_CY"
            :r="MODULE_RING"
            class="arch-orbit arch-orbit--facility"
          />
          <text
            :x="HUB_CX"
            :y="HUB_CY - KERNEL_OUTER_R - 10"
            class="arch-orbit-label"
            text-anchor="middle"
          >
            {{ t("pluginWorkbench.graph.ringKernel") }}
          </text>
          <text
            :x="HUB_CX"
            :y="HUB_CY - MODULE_RING - 8"
            class="arch-orbit-label arch-orbit-label--inner"
            text-anchor="middle"
          >
            {{ t("pluginWorkbench.graph.ringFacility") }}
          </text>
          <path
            v-for="edge in edges"
            :key="edge.id"
            :d="edge.d"
            class="arch-edge"
            :class="[
              `arch-edge--${edge.kind}`,
              {
                'arch-edge--flow': edge.animated,
                'arch-edge--hot': edge.toModule && hoveredModule === edge.toModule,
              },
            ]"
            fill="none"
            :stroke="BACKEND_COLORS[edge.kind].stroke"
            :stroke-width="edgeStrokeWidth(edge)"
            :stroke-dasharray="edgeDash(edge.kind) === 'none' ? undefined : edgeDash(edge.kind)"
            :filter="edge.animated ? 'url(#arch-glow-remote)' : undefined"
          />
        </svg>

        <div
          class="arch-kernel arch-kernel--hex arch-kernel--comfy"
          :style="{
            left: kernelLayout.x + 'px',
            top: kernelLayout.y + 'px',
            width: kernelLayout.r * 2 + 'px',
            height: kernelLayout.r * 2 + 'px',
          }"
          :title="t('pluginWorkbench.graph.kernelTitle')"
        >
          <div
            class="arch-kernel-drag ge-node-drag-handle"
            @pointerdown="onNodeDragStart($event, KERNEL_ID)"
          >
            <span class="arch-kernel-glow" aria-hidden="true" />
            <span class="arch-kernel-ico" aria-hidden="true">⚙️</span>
            <span class="arch-kernel-lbl">{{ t("pluginWorkbench.graph.kernel") }}</span>
            <span class="arch-kernel-sub">process_message</span>
          </div>
          <span
            class="ge-slot-dot ge-slot-dot--out arch-kernel-out-dot"
            :title="t('pluginWorkbench.graph.portPipeline')"
          />
        </div>

        <div
          class="ge-node ge-node--comfy ge-node--bus ge-node--slots"
          :style="{ left: facilityBus.x + 'px', top: facilityBus.y + 'px', width: facilityBus.w + 'px', minHeight: facilityBus.h + 'px' }"
          @click.stop="selectedModule = null"
        >
          <div
            class="ge-node-header ge-node-drag-handle"
            @pointerdown="onNodeDragStart($event, BUS_ID)"
          >
            <span class="ge-node-header-title">{{ t("pluginWorkbench.graph.facilityBus") }}</span>
            <span class="ge-node-header-type">plugin_backends</span>
          </div>
          <div class="ge-node-body-row">
            <div class="ge-slot-col ge-slot-col--in">
              <div
                class="ge-slot-item"
                :style="{ top: `${(1 / 2) * 100}%` }"
              >
                <span class="ge-slot-dot ge-slot-dot--in" />
                <span class="ge-slot-label">{{ t("pluginWorkbench.graph.portIn") }}</span>
              </div>
            </div>
            <div class="ge-slot-col ge-slot-col--mid">
              <p class="ge-bus-hint">{{ t("pluginWorkbench.graph.facilityBusHint") }}</p>
            </div>
            <div class="ge-slot-col ge-slot-col--out">
              <div
                v-for="(sl, i) in slotLayouts"
                :key="'bus-out-' + sl.key"
                class="ge-slot-item ge-slot-item--anchored"
                :style="{ top: `${((i + 1) / (slotLayouts.length + 1)) * 100}%` }"
              >
                <span class="ge-slot-label ge-slot-label--out">{{ sl.key }}</span>
                <span class="ge-slot-dot ge-slot-dot--out" />
              </div>
            </div>
          </div>
        </div>

        <div
          v-for="sl in slotLayouts"
          :key="sl.key"
          class="ge-node ge-node--comfy ge-node--slots"
          :class="{
            'ge-node--selected': selectedModule === sl.key,
            'ge-node--hover': hoveredModule === sl.key,
            [`ge-node--${backendKind(sl.key)}`]: true,
          }"
          :style="{ left: sl.x + 'px', top: sl.y + 'px', width: NODE_W + 'px', minHeight: NODE_H + 'px' }"
          @mouseenter="hoveredModule = sl.key"
          @mouseleave="hoveredModule = null"
          @click.stop="selectModule(sl.key)"
          @dblclick.stop="onNodeDblClick(sl)"
        >
          <div
            class="ge-node-header ge-node-drag-handle"
            :style="{ borderColor: BACKEND_COLORS[backendKind(sl.key)].bar }"
            @pointerdown="onNodeDragStart($event, sl.key)"
          >
            <span class="ge-node-header-ico" aria-hidden="true">{{ sl.icon }}</span>
            <span class="ge-node-header-title">{{ sl.key }}</span>
            <span class="ge-node-header-type">{{ t(sl.labelKey) }}</span>
          </div>
          <div class="ge-node-body-row">
            <div class="ge-slot-col ge-slot-col--in">
              <div class="ge-slot-item ge-slot-item--anchored" :style="{ top: '50%' }">
                <span
                  class="ge-slot-dot ge-slot-dot--in"
                  :style="{ borderColor: BACKEND_COLORS[backendKind(sl.key)].bar }"
                />
                <span class="ge-slot-label">{{ t("pluginWorkbench.graph.portBackend") }}</span>
              </div>
            </div>
            <div class="ge-slot-col ge-slot-col--mid ge-node-widgets">
            <p
              v-if="backendKind(sl.key) === 'directory' && directoryPluginIds(sl.key).length"
              class="ge-dir-line"
            >
              {{ directoryPluginIds(sl.key)[0] }}
            </p>
            <div class="ge-widget-row">
              <label class="ge-widget-lbl">{{ t("pluginWorkbench.graph.switchBackend") }}</label>
              <select
                class="ge-select ge-select--widget"
                :disabled="busy"
                :value="pluginBackendsSessionOverride?.[sl.key] ?? '__pack_default__'"
                @click.stop
                @change="onBackendChange(sl.key, $event)"
              >
                <option value="__pack_default__">
                  {{ t("pluginWorkbench.graph.followPack", { value: pluginBackends[sl.key] }) }}
                </option>
                <option v-for="v in sl.options" :key="v" :value="v">{{ v }}</option>
              </select>
            </div>
            <div class="ge-node-actions">
              <button
                v-if="directoryPluginIds(sl.key)[0]"
                type="button"
                class="ge-btn ge-btn--ghost"
                @click.stop="onFocusPlugin(directoryPluginIds(sl.key)[0]!)"
              >
                {{ t("pluginWorkbench.graph.detail") }}
              </button>
              <button
                v-if="hiddenPluginCount(sl.key) > 0"
                type="button"
                class="ge-plus-n"
                @click.stop="togglePluginExpand(sl.key)"
              >
                +{{ hiddenPluginCount(sl.key) }} {{ t("pluginWorkbench.graph.plugins") }}
              </button>
            </div>
            </div>
            <div v-if="backendKind(sl.key) === 'directory'" class="ge-slot-col ge-slot-col--out">
              <div class="ge-slot-item ge-slot-item--anchored" :style="{ top: '50%' }">
                <span class="ge-slot-label ge-slot-label--out">{{ t("pluginWorkbench.graph.portPlugin") }}</span>
                <span class="ge-slot-dot ge-slot-dot--out" />
              </div>
            </div>
          </div>
        </div>

        <div
          v-for="pl in pluginLayouts"
          :key="pl.pid"
          class="ge-plugin ge-node--comfy ge-node--slots"
          :class="{ 'ge-plugin--off': pluginStore.isPluginDisabled(pl.pid) }"
          :style="{ left: pl.x + 'px', top: pl.y + 'px', width: PLUGIN_W + 'px' }"
          @click.stop="onFocusPlugin(pl.pid)"
          @contextmenu="openCtx($event, pl.pid)"
        >
          <div
            class="ge-node-header ge-node-header--plugin ge-node-drag-handle"
            @pointerdown="onNodeDragStart($event, `plugin:${pl.pid}`)"
          >
            <span class="ge-node-header-title">{{ pl.pid }}</span>
            <span class="ge-node-header-type">directory</span>
          </div>
          <div class="ge-node-body-row ge-node-body-row--plugin">
            <div class="ge-slot-col ge-slot-col--in">
              <div class="ge-slot-item ge-slot-item--anchored" :style="{ top: '50%' }">
                <span class="ge-slot-dot ge-slot-dot--in ge-slot-dot--plugin" />
                <span class="ge-slot-label">{{ pl.moduleKey }}</span>
              </div>
            </div>
            <div class="ge-slot-col ge-slot-col--mid ge-plugin-body">
            <span class="ge-plugin-module">{{ pl.moduleKey }}</span>
            <strong class="ge-plugin-id">{{ pl.pid }}</strong>
            <span class="ge-plugin-ver">v{{ catalogEntry(pl.pid)?.version ?? "?" }}</span>
            <span class="ge-plugin-state">
              {{
                pluginStore.isPluginDisabled(pl.pid)
                  ? t("pluginWorkbench.graph.pluginDisabled")
                  : t("pluginWorkbench.graph.pluginEnabled")
              }}
            </span>
            </div>
          </div>
        </div>

        <div
          class="ge-node ge-node--comfy ge-node--slots ge-node--complex ge-node--builtin"
          :style="{
            left: complexLayout.x + 'px',
            top: complexLayout.y + 'px',
            width: complexLayout.w + 'px',
            minHeight: NODE_H + 'px',
          }"
        >
          <div
            class="ge-node-header ge-node-drag-handle"
            :style="{ borderColor: BACKEND_COLORS.builtin.bar }"
            @pointerdown="onNodeDragStart($event, COMPLEX_ID)"
          >
            <span class="ge-node-header-ico" aria-hidden="true">🎭</span>
            <span class="ge-node-header-title">{{ t("pluginWorkbench.graph.complexEmotion") }}</span>
          </div>
          <div class="ge-node-body-row">
            <div class="ge-slot-col ge-slot-col--in">
              <div class="ge-slot-item ge-slot-item--anchored" :style="{ top: '50%' }">
                <span class="ge-slot-dot ge-slot-dot--in" :style="{ borderColor: BACKEND_COLORS.builtin.bar }" />
                <span class="ge-slot-label">{{ t("pluginWorkbench.graph.portBackend") }}</span>
              </div>
            </div>
            <div class="ge-slot-col ge-slot-col--mid">
              <p class="ge-complex-hint">{{ t("pluginWorkbench.graph.complexHint") }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="arch-legend" role="list" :aria-label="t('pluginWorkbench.graph.legendAria')">
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--builtin" />{{ t("pluginWorkbench.graph.legendBuiltin") }}
      </span>
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--remote" />{{ t("pluginWorkbench.graph.legendRemote") }}
      </span>
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--directory" />{{ t("pluginWorkbench.graph.legendDirectory") }}
      </span>
      <span class="arch-legend-hint">{{ t("pluginWorkbench.graph.panHint") }}</span>
      <span class="arch-legend-hint">{{ t("pluginWorkbench.graph.dragHint") }}</span>
    </div>

    <div
      v-if="ctxMenu"
      class="arch-ctx"
      :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
      role="menu"
      @click.stop
    >
      <button type="button" role="menuitem" @click="onFocusPlugin(ctxMenu.pluginId)">
        {{ t("pluginWorkbench.graph.ctxSettings") }}
      </button>
      <button type="button" role="menuitem" @click="togglePluginDisabled(ctxMenu.pluginId)">
        {{ t("pluginWorkbench.graph.ctxToggle") }}
      </button>
      <button type="button" role="menuitem" class="arch-ctx-danger" @click="uninstallPlugin(ctxMenu.pluginId)">
        {{ t("pluginWorkbench.graph.ctxUninstall") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.arch-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
  position: relative;
}
.arch-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.arch-viewport {
  position: relative;
  height: min(600px, 62vh);
  min-height: 420px;
  overflow: hidden;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--graph-canvas-bg, var(--bg-elevated));
  touch-action: none;
}
.arch-canvas-title {
  position: absolute;
  z-index: 6;
  left: 50%;
  top: 8px;
  transform: translateX(-50%);
  padding: 4px 14px;
  border-radius: var(--radius-pill);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--text-secondary);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 90%, transparent);
  backdrop-filter: blur(4px);
  pointer-events: none;
}
.arch-viewport--pan {
  cursor: grab;
}
.arch-viewport--pan:active {
  cursor: grabbing;
}
.arch-grid {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: radial-gradient(
    circle,
    var(--graph-grid-color, color-mix(in srgb, var(--text-secondary) 35%, transparent)) var(--graph-grid-dot, 1px),
    transparent var(--graph-grid-dot, 1px)
  );
  opacity: 0.85;
}
html[data-theme="dark"] .arch-grid {
  --graph-grid-color: color-mix(in srgb, #fff 22%, transparent);
}
html:not([data-theme="dark"]) .arch-grid {
  --graph-grid-color: color-mix(in srgb, #000 18%, transparent);
}
.arch-world {
  position: absolute;
  left: 0;
  top: 0;
  transform-origin: 0 0;
}
.arch-edges {
  position: absolute;
  left: 0;
  top: 0;
  pointer-events: none;
  overflow: visible;
}
.arch-edge {
  stroke-linecap: round;
  stroke-linejoin: round;
}
.arch-edge--flow {
  animation: arch-flow 1.1s linear infinite;
}
.arch-orbit--kernel {
  stroke: color-mix(in srgb, #4caf50 35%, transparent);
  stroke-dasharray: 6 8;
}
.arch-orbit-label--inner {
  font-size: 9px;
  opacity: 0.6;
}
.arch-minimap-ring--outer {
  stroke: color-mix(in srgb, #4caf50 30%, transparent);
}
.arch-kernel-drag {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  cursor: grab;
}
.arch-kernel-drag:active {
  cursor: grabbing;
}
.arch-kernel-out-dot {
  position: absolute;
  left: 50%;
  bottom: 2px;
  transform: translate(-50%, 50%);
  z-index: 2;
}
.ge-node--slots .ge-node-body-row {
  display: flex;
  flex-direction: row;
  align-items: stretch;
  min-height: 72px;
  position: relative;
}
.ge-node--bus .ge-node-body-row {
  min-height: 120px;
}
.ge-slot-col {
  position: relative;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}
.ge-slot-col--in {
  width: 22px;
  margin-left: -11px;
}
.ge-slot-col--out {
  width: 22px;
  margin-right: -11px;
}
.ge-slot-col--mid {
  flex: 1;
  min-width: 0;
  padding: 6px 10px 8px;
}
.ge-slot-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 9px;
  color: var(--text-secondary);
  white-space: nowrap;
}
.ge-slot-item--anchored {
  position: absolute;
  left: 0;
  right: 0;
  transform: translateY(-50%);
}
.ge-slot-col--out .ge-slot-item--anchored {
  flex-direction: row-reverse;
  justify-content: flex-start;
}
.ge-slot-col--in .ge-slot-item--anchored {
  justify-content: flex-start;
}
.ge-slot-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: 0 0 0 2px var(--bg-primary);
  flex-shrink: 0;
  z-index: 2;
}
.ge-slot-dot--in {
  border-color: #4caf50;
}
.ge-slot-dot--out {
  border-color: #9c27b0;
}
.ge-slot-dot--plugin {
  border-color: #9c27b0;
}
.ge-slot-label {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  max-width: 72px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ge-slot-label--out {
  text-align: right;
}
.ge-node-body-row--plugin {
  min-height: 52px;
}
.ge-node-body-row--plugin .ge-slot-col--mid {
  padding: 4px 8px 6px;
}
@keyframes arch-flow {
  to {
    stroke-dashoffset: -24;
  }
}
.arch-edge--hot {
  filter: drop-shadow(0 0 4px color-mix(in srgb, currentColor 40%, transparent));
}
.arch-toolbar {
  position: absolute;
  z-index: 5;
  left: 10px;
  top: 10px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 92%, transparent);
  backdrop-filter: blur(6px);
}
.arch-tb-btn {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  color: var(--text-primary);
}
.arch-tb-scale {
  font-size: 11px;
  color: var(--text-secondary);
  min-width: 3em;
  text-align: center;
}
.arch-minimap {
  position: absolute;
  z-index: 5;
  right: 10px;
  bottom: 10px;
  width: 120px;
  height: 64px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 88%, transparent);
  padding: 4px;
  opacity: 0.9;
}
.arch-minimap-svg {
  width: 100%;
  height: 100%;
}
.arch-minimap-node {
  fill: color-mix(in srgb, var(--accent) 25%, transparent);
  stroke: none;
}
.arch-minimap-ring {
  fill: none;
  stroke: color-mix(in srgb, var(--text-secondary) 25%, transparent);
  stroke-width: 1;
}
.arch-minimap-ring--outer {
  stroke-dasharray: 2 3;
}
.arch-minimap-kernel {
  fill: color-mix(in srgb, #4caf50 35%, transparent);
}
.arch-layer-legend {
  position: absolute;
  z-index: 5;
  right: 10px;
  top: 44px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 90%, transparent);
  backdrop-filter: blur(6px);
  font-size: 10px;
}
.arch-layer-pill {
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
}
.arch-layer-pill--hub {
  border-color: color-mix(in srgb, #4caf50 45%, var(--border-light));
  color: color-mix(in srgb, #4caf50 80%, var(--text-primary));
}
.arch-layer-pill--facility {
  border-color: color-mix(in srgb, #2196f3 35%, var(--border-light));
}
.arch-layer-pill--plugin {
  border-color: color-mix(in srgb, #9c27b0 35%, var(--border-light));
}
.arch-orbit {
  fill: none;
  stroke: color-mix(in srgb, var(--text-secondary) 18%, transparent);
  stroke-width: 1;
  pointer-events: none;
}
.arch-orbit--facility {
  stroke-dasharray: 4 6;
}
.arch-orbit--plugin {
  stroke-dasharray: 2 5;
  opacity: 0.65;
}
.arch-orbit-label {
  fill: var(--text-secondary);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  opacity: 0.75;
  pointer-events: none;
}
.arch-orbit-label--outer {
  font-size: 9px;
  opacity: 0.55;
}
.arch-kernel {
  position: absolute;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  border: 2px solid color-mix(in srgb, #4caf50 55%, var(--border-light));
  background: linear-gradient(
    145deg,
    var(--bg-primary),
    color-mix(in srgb, #4caf50 8%, var(--bg-elevated))
  );
  box-shadow:
    0 2px 12px rgba(0, 0, 0, 0.18),
    inset 0 1px 0 color-mix(in srgb, #fff 12%, transparent);
  text-align: center;
}
.arch-kernel--hex {
  border-radius: 0;
  clip-path: polygon(50% 0%, 93% 25%, 93% 75%, 50% 100%, 7% 75%, 7% 25%);
}
.arch-kernel-glow {
  position: absolute;
  inset: -8px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(76, 175, 80, 0.35) 0%, transparent 70%);
  animation: arch-breathe 2.8s ease-in-out infinite;
  pointer-events: none;
}
@keyframes arch-breathe {
  0%,
  100% {
    opacity: 0.45;
    transform: scale(0.95);
  }
  50% {
    opacity: 0.9;
    transform: scale(1.05);
  }
}
.arch-kernel-ico {
  font-size: 26px;
  z-index: 1;
}
.arch-kernel-lbl {
  font-size: 11px;
  font-weight: 700;
  z-index: 1;
}
.arch-kernel-sub {
  font-size: 9px;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
  z-index: 1;
}
.ge-node--comfy {
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-primary) 96%, #1a1a2e 4%);
}
.ge-node--bus {
  border: 2px dashed color-mix(in srgb, #2196f3 40%, var(--border-light));
  background: color-mix(in srgb, #2196f3 6%, var(--bg-primary));
  z-index: 3;
}
.ge-node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 9px 9px 0 0;
  border-bottom: 2px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 85%, #000 15%);
  cursor: grab;
  user-select: none;
}
.ge-node-header:active {
  cursor: grabbing;
}
.ge-node-header--plugin {
  padding: 5px 8px;
  border-bottom-width: 1px;
}
.ge-node-header-ico {
  font-size: 14px;
}
.ge-node-header-title {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 700;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ge-node-header-type {
  font-size: 9px;
  color: var(--text-secondary);
  max-width: 42%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ge-node-ports {
  padding: 4px 8px 2px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.ge-port-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  color: var(--text-secondary);
}
.ge-port-row--in {
  justify-content: flex-start;
}
.ge-port-row--out {
  justify-content: flex-end;
  flex-direction: row-reverse;
}
.ge-port-row--plugin {
  padding: 0 8px 4px;
}
.ge-port-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid var(--border-light);
  background: var(--bg-primary);
  flex-shrink: 0;
}
.ge-port-dot--in {
  border-color: #4caf50;
}
.ge-port-dot--out {
  border-color: #9c27b0;
}
.ge-port-dot--plugin {
  border-color: #9c27b0;
}
.ge-port-name {
  font-family: ui-monospace, Menlo, Consolas, monospace;
}
.ge-port-labeled {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  pointer-events: none;
}
.ge-bus-hint {
  margin: 0;
  padding: 4px 10px 8px;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.35;
}
.ge-node-widgets {
  padding: 6px 10px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ge-widget-row {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.ge-widget-lbl {
  font-size: 10px;
  color: var(--text-secondary);
}
.ge-select--widget {
  width: 100%;
}
.arch-minimap-bus {
  fill: color-mix(in srgb, #2196f3 20%, transparent);
  stroke: none;
}
.ge-node {
  position: absolute;
  z-index: 3;
  border-radius: 8px;
  background: var(--bg-primary);
  box-shadow:
    0 2px 8px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 color-mix(in srgb, #fff 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-light) 90%, #000);
  overflow: visible;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    border-color 0.15s ease;
  cursor: default;
}
.ge-node:hover {
  transform: scale(1.02);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.22);
  z-index: 4;
}
.ge-node--selected {
  border: 2px solid #2196f3;
  box-shadow:
    0 0 0 3px color-mix(in srgb, #2196f3 22%, transparent),
    0 6px 18px rgba(0, 0, 0, 0.2);
  z-index: 5;
}
.ge-port {
  position: absolute;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid;
  background: var(--bg-primary);
  box-shadow: 0 0 0 2px var(--bg-primary);
  z-index: 2;
}
.ge-port--radial {
  margin: 0;
}
.ge-port--out {
  border-color: #9c27b0;
}
.ge-port--plugin-in {
  border-color: #9c27b0;
}
.ge-plugin-module {
  width: 100%;
  font-size: 9px;
  font-family: ui-monospace, monospace;
  color: var(--text-secondary);
}
.ge-node-bar {
  height: 3px;
  width: 100%;
}
.ge-node-body {
  padding: 8px 10px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ge-node-title {
  display: flex;
  align-items: flex-start;
  gap: 6px;
}
.ge-node-title-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.ge-node-id {
  font-size: 10px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  color: var(--text-secondary);
}
.ge-node-zh {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.ge-status-led {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 4px;
  box-shadow: 0 0 6px currentColor;
}
.ge-tag {
  align-self: flex-start;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  font-family: ui-monospace, monospace;
}
.ge-dir-line {
  margin: 0;
  font-size: 11px;
  color: var(--text-secondary);
  word-break: break-all;
}
.ge-plus-n {
  margin-left: 6px;
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, #9c27b0 45%, var(--border-light));
  background: color-mix(in srgb, #9c27b0 12%, transparent);
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  color: #9c27b0;
}
.ge-node-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.ge-btn {
  padding: 4px 8px;
  font-size: 11px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  color: var(--text-primary);
}
.ge-btn--ghost {
  background: transparent;
}
.ge-switch-panel {
  margin-top: 2px;
}
.ge-select {
  width: 100%;
  font-size: 11px;
  padding: 4px 6px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.ge-plugin {
  position: absolute;
  z-index: 3;
  border-radius: 8px;
  transform: scale(0.85);
  transform-origin: top left;
  background: var(--bg-primary);
  border: 1px solid color-mix(in srgb, #9c27b0 35%, var(--border-light));
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}
.ge-plugin:hover {
  transform: scale(0.88);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.18);
}
.ge-plugin--off {
  opacity: 0.55;
  border-style: dashed;
}
.ge-plugin-bar {
  height: 3px;
  background: #9c27b0;
}
.ge-plugin-body {
  padding: 6px 8px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 8px;
  font-size: 11px;
}
.ge-plugin-id {
  font-family: ui-monospace, monospace;
}
.ge-plugin-ver {
  color: var(--text-secondary);
}
.ge-plugin-state {
  font-size: 10px;
  color: var(--text-secondary);
}
.ge-node--complex .ge-complex-hint {
  margin: 0;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.arch-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  font-size: 11px;
  color: var(--text-secondary);
}
.arch-legend-hint {
  margin-left: auto;
  font-size: 10px;
  opacity: 0.85;
}
.arch-swatch {
  display: inline-block;
  width: 18px;
  height: 0;
  border-top: 3px solid;
  margin-right: 4px;
  vertical-align: middle;
}
.arch-swatch--builtin {
  border-color: #4caf50;
}
.arch-swatch--remote {
  border-color: #2196f3;
  border-top-style: dashed;
}
.arch-swatch--directory {
  border-color: #9c27b0;
  border-top-style: dotted;
}
.arch-ctx {
  position: fixed;
  z-index: 10100;
  min-width: 140px;
  padding: 4px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
}
.arch-ctx button {
  text-align: left;
  padding: 8px 10px;
  border: none;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  border-radius: 4px;
  color: var(--text-primary);
}
.arch-ctx button:hover {
  background: var(--bg-elevated);
}
.arch-ctx-danger {
  color: var(--error, #c44) !important;
}
</style>
