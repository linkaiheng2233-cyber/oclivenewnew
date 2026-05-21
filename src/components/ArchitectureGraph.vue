<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useGraphCanvas } from "../composables/useGraphCanvas";
import {
  BACKEND_COLORS,
  bezierPath,
  edgeDash,
  normalizeBackendKind,
  type BackendKind,
} from "../lib/graphEditorTheme";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";
import { useAppToast } from "../composables/useAppToast";
import { setSessionPluginBackend } from "../utils/tauri-api";
import type { DirectoryPluginCatalogEntry } from "../utils/tauri-api";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();

const WORLD_W = 1180;
const WORLD_H = 560;
const NODE_W = 228;
const NODE_H = 112;
const PLUGIN_W = 188;
const PLUGIN_H = 72;

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
  onPointerDown,
  onPointerMove,
  onPointerUp,
} = useGraphCanvas({ worldWidth: WORLD_W, worldHeight: WORLD_H });

const busy = ref(false);
const selectedModule = ref<string | null>(null);
const hoveredModule = ref<string | null>(null);
const expandedPlugins = ref<Record<string, boolean>>({});
const switchOpen = ref<string | null>(null);
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

const KERNEL = { cx: 150, cy: 280, r: 56 };

const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
const pluginBackendsSessionOverride = computed(
  () => roleStore.roleInfo.pluginBackendsSessionOverride,
);

type SlotLayout = {
  key: CoreModule;
  x: number;
  y: number;
  labelKey: string;
  icon: string;
  options: string[];
};

const slotLayouts = computed<SlotLayout[]>(() => {
  const colX = [468, 708];
  const rowY = [32, 132, 232];
  return coreModules.map((m, i) => ({
    ...m,
    x: colX[Math.floor(i / 3)] ?? colX[0]!,
    y: rowY[i % 3] ?? 32,
  }));
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

type EdgeDef = {
  id: string;
  d: string;
  kind: BackendKind;
  toModule?: CoreModule;
  animated?: boolean;
};

const edges = computed<EdgeDef[]>(() => {
  const out: EdgeDef[] = [];
  const kx = KERNEL.cx + KERNEL.r * 0.92;
  const ky = KERNEL.cy;

  for (const sl of slotLayouts.value) {
    const kind = backendKind(sl.key);
    const sx = sl.x + 6;
    const sy = sl.y + NODE_H / 2;
    out.push({
      id: `k-${sl.key}`,
      d: bezierPath(kx, ky, sx, sy, 0.5),
      kind,
      toModule: sl.key,
      animated: kind === "remote",
    });

    if (kind !== "directory") continue;
    const plugins = visiblePluginIds(sl.key);
    plugins.forEach((pid, j) => {
      const ox = sl.x + NODE_W - 4;
      const oy = sl.y + NODE_H / 2;
      const px = ox + 28;
      const py = sl.y + 20 + j * (PLUGIN_H + 12);
      out.push({
        id: `p-${sl.key}-${pid}`,
        d: bezierPath(ox, oy, px, py + PLUGIN_H / 2, 0.4),
        kind: "directory",
        animated: false,
      });
    });
  }

  out.push({
    id: "k-complex",
    d: bezierPath(kx, ky, 520, 468, 0.4),
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
    switchOpen.value = null;
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
  focusPoint(sl.x + NODE_W / 2, sl.y + NODE_H / 2);
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
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
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
        <span class="arch-tb-scale">{{ scalePercent }}</span>
      </div>

      <div class="arch-minimap" aria-hidden="true" @click.stop>
        <svg :viewBox="`0 0 ${WORLD_W} ${WORLD_H}`" class="arch-minimap-svg">
          <rect
            v-for="sl in slotLayouts"
            :key="'mm-' + sl.key"
            :x="sl.x"
            :y="sl.y"
            :width="NODE_W"
            :height="NODE_H"
            class="arch-minimap-node"
          />
          <circle :cx="KERNEL.cx" :cy="KERNEL.cy" :r="KERNEL.r" class="arch-minimap-kernel" />
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
          class="arch-kernel arch-kernel--hex"
          :style="{ left: KERNEL.cx - KERNEL.r + 'px', top: KERNEL.cy - KERNEL.r + 'px', width: KERNEL.r * 2 + 'px', height: KERNEL.r * 2 + 'px' }"
          :title="t('pluginWorkbench.graph.kernelTitle')"
        >
          <span class="arch-kernel-glow" aria-hidden="true" />
          <span class="arch-kernel-ico" aria-hidden="true">⚙️</span>
          <span class="arch-kernel-lbl">{{ t("pluginWorkbench.graph.kernel") }}</span>
          <span class="arch-kernel-sub">process_message</span>
        </div>

        <div
          v-for="sl in slotLayouts"
          :key="sl.key"
          class="ge-node"
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
          <span
            class="ge-port ge-port--in"
            :style="{ borderColor: BACKEND_COLORS[backendKind(sl.key)].bar }"
          />
          <span
            v-if="backendKind(sl.key) === 'directory'"
            class="ge-port ge-port--out"
          />
          <div
            class="ge-node-bar"
            :style="{ background: BACKEND_COLORS[backendKind(sl.key)].bar }"
          />
          <div class="ge-node-body">
            <div class="ge-node-title">
              <span aria-hidden="true">{{ sl.icon }}</span>
              <span class="ge-node-title-text">
                <span class="ge-node-id">{{ sl.key }}</span>
                <span class="ge-node-zh">{{ t(sl.labelKey) }}</span>
              </span>
              <span
                class="ge-status-led"
                :style="{ background: BACKEND_COLORS[backendKind(sl.key)].bar }"
                :title="effectiveBackend(sl.key)"
              />
            </div>
            <span
              class="ge-tag"
              :style="{
                color: BACKEND_COLORS[backendKind(sl.key)].bar,
                background: BACKEND_COLORS[backendKind(sl.key)].tagBg,
              }"
            >
              {{ effectiveBackend(sl.key) }}
            </span>
            <p
              v-if="backendKind(sl.key) === 'directory' && directoryPluginIds(sl.key).length"
              class="ge-dir-line"
            >
              {{ directoryPluginIds(sl.key)[0] }}
              <button
                v-if="hiddenPluginCount(sl.key) > 0"
                type="button"
                class="ge-plus-n"
                @click.stop="togglePluginExpand(sl.key)"
              >
                +{{ hiddenPluginCount(sl.key) }}
              </button>
            </p>
            <div class="ge-node-actions">
              <button
                type="button"
                class="ge-btn"
                @click.stop="switchOpen = switchOpen === sl.key ? null : sl.key"
              >
                {{ t("pluginWorkbench.graph.switchBackend") }}
              </button>
              <button
                v-if="directoryPluginIds(sl.key)[0]"
                type="button"
                class="ge-btn ge-btn--ghost"
                @click.stop="onFocusPlugin(directoryPluginIds(sl.key)[0]!)"
              >
                {{ t("pluginWorkbench.graph.detail") }}
              </button>
            </div>
            <div v-if="switchOpen === sl.key" class="ge-switch-panel" @click.stop>
              <select
                class="ge-select"
                :disabled="busy"
                :value="pluginBackendsSessionOverride?.[sl.key] ?? '__pack_default__'"
                @change="onBackendChange(sl.key, $event)"
              >
                <option value="__pack_default__">
                  {{ t("pluginWorkbench.graph.followPack", { value: pluginBackends[sl.key] }) }}
                </option>
                <option v-for="v in sl.options" :key="v" :value="v">{{ v }}</option>
              </select>
            </div>
          </div>
        </div>

        <template v-for="sl in slotLayouts" :key="'pl-' + sl.key">
          <div
            v-for="(pid, j) in visiblePluginIds(sl.key)"
            :key="pid"
            class="ge-plugin"
            :class="{ 'ge-plugin--off': pluginStore.isPluginDisabled(pid) }"
            :style="{
              left: sl.x + NODE_W + 16 + 'px',
              top: sl.y + 14 + j * (PLUGIN_H + 10) + 'px',
              width: PLUGIN_W + 'px',
            }"
            @click.stop="onFocusPlugin(pid)"
            @contextmenu="openCtx($event, pid)"
          >
            <div class="ge-plugin-bar" />
            <div class="ge-plugin-body">
              <strong class="ge-plugin-id">{{ pid }}</strong>
              <span class="ge-plugin-ver">v{{ catalogEntry(pid)?.version ?? "?" }}</span>
              <span class="ge-plugin-state">
                {{
                  pluginStore.isPluginDisabled(pid)
                    ? t("pluginWorkbench.graph.pluginDisabled")
                    : t("pluginWorkbench.graph.pluginEnabled")
                }}
              </span>
            </div>
          </div>
        </template>

        <div
          class="ge-node ge-node--complex ge-node--builtin"
          :style="{ left: '500px', top: '448px', width: '220px' }"
        >
          <span class="ge-port ge-port--in" :style="{ borderColor: BACKEND_COLORS.builtin.bar }" />
          <div class="ge-node-bar" :style="{ background: BACKEND_COLORS.builtin.bar }" />
          <div class="ge-node-body">
            <div class="ge-node-title">
              <span aria-hidden="true">🎭</span>
              <span class="ge-node-title-text">
                <span class="ge-node-zh">{{ t("pluginWorkbench.graph.complexEmotion") }}</span>
              </span>
            </div>
            <p class="ge-complex-hint">{{ t("pluginWorkbench.graph.complexHint") }}</p>
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
  height: min(540px, 58vh);
  min-height: 380px;
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
.arch-edge--flow {
  animation: arch-flow 1.1s linear infinite;
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
.arch-minimap-kernel {
  fill: color-mix(in srgb, #4caf50 35%, transparent);
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
.ge-port--in {
  left: -5px;
  top: 50%;
  margin-top: -5px;
}
.ge-port--out {
  right: -5px;
  top: 50%;
  margin-top: -5px;
  border-color: #9c27b0;
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
