<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useGraphCanvas } from "../composables/useGraphCanvas";
import {
  SLOT_CHAT_HEADER,
  SLOT_CHAT_TOOLBAR,
  SLOT_OVERLAY_FLOATING,
  SLOT_ROLE_DETAIL,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";
import type { DirectoryPluginCatalogEntry } from "../utils/tauri-api";

const WORLD_W = 820;
const WORLD_H = 500;

const pluginStore = usePluginStore();
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
} = useGraphCanvas({ worldWidth: WORLD_W, worldHeight: WORLD_H, minScale: 0.55, maxScale: 1.8 });

const selectedSlot = ref<string | null>(null);

type DiagramSlot = {
  key: string;
  labelKey: string;
  descKey: string;
  x: number;
  y: number;
  w: number;
};

const diagramSlots: DiagramSlot[] = [
  {
    key: SLOT_CHAT_HEADER,
    labelKey: "pluginWorkbench.layout.chatHeader",
    descKey: "pluginWorkbench.layout.descHeader",
    x: 300,
    y: 36,
    w: 220,
  },
  {
    key: SLOT_SIDEBAR,
    labelKey: "pluginWorkbench.layout.sidebar",
    descKey: "pluginWorkbench.layout.descSidebar",
    x: 40,
    y: 150,
    w: 200,
  },
  {
    key: SLOT_CHAT_TOOLBAR,
    labelKey: "pluginWorkbench.layout.chatToolbar",
    descKey: "pluginWorkbench.layout.descToolbar",
    x: 300,
    y: 200,
    w: 220,
  },
  {
    key: SLOT_ROLE_DETAIL,
    labelKey: "pluginWorkbench.layout.roleDetail",
    descKey: "pluginWorkbench.layout.descRoleDetail",
    x: 300,
    y: 300,
    w: 220,
  },
  {
    key: SLOT_OVERLAY_FLOATING,
    labelKey: "pluginWorkbench.layout.overlay",
    descKey: "pluginWorkbench.layout.descOverlay",
    x: 220,
    y: 400,
    w: 240,
  },
];

const regions = [
  { id: "header", x: 260, y: 12, w: 520, h: 88, labelKey: "pluginWorkbench.layout.regionHeader" },
  { id: "body", x: 24, y: 108, w: 772, h: 280, labelKey: "pluginWorkbench.layout.regionMain" },
  { id: "overlay", x: 180, y: 378, w: 460, h: 100, labelKey: "pluginWorkbench.layout.regionOverlay" },
];

function isContributionOff(slot: string, pluginId: string): boolean {
  if (slot === SLOT_CHAT_TOOLBAR) {
    return pluginStore.isToolbarContributionDisabled(pluginId);
  }
  return pluginStore.isSlotContributionDisabled(slot, pluginId);
}

function candidatesForSlot(slot: string): string[] {
  return (pluginStore.catalogCandidatesBySlot[slot] ?? []).filter(
    (id) => !pluginStore.isPluginDisabled(id) && !isContributionOff(slot, id),
  );
}

function orderedActive(slot: string): string[] {
  return pluginStore.pluginsOrderedForSlot(slot).filter((id) => candidatesForSlot(slot).includes(id));
}

function catalogEntry(id: string): DirectoryPluginCatalogEntry | undefined {
  return pluginStore.catalog.find((c) => c.id === id);
}

function onSlotSelect(slot: string, ev: Event) {
  const v = (ev.target as HTMLSelectElement).value;
  if (!v) {
    pluginStore.setSlotPluginIds(slot, []);
    return;
  }
  const rest = orderedActive(slot).filter((id) => id !== v);
  pluginStore.setSlotPluginIds(slot, [v, ...rest]);
}

function isEmpty(slot: string): boolean {
  return orderedActive(slot).length === 0;
}

function selectSlot(key: string) {
  selectedSlot.value = key;
}

function onSlotDblClick(sl: DiagramSlot) {
  selectSlot(sl.key);
  focusPoint(sl.x + sl.w / 2, sl.y + 60);
}

const contributionRows = computed(() =>
  pluginStore.catalog.map((entry) => ({
    id: entry.id,
    slots: entry.uiSlotNames ?? [],
    disabled: pluginStore.isPluginDisabled(entry.id),
  })),
);

onMounted(() => fitWorld());
</script>

<template>
  <div class="sld-root">
    <p class="sld-lead">{{ t("pluginWorkbench.layout.lead") }}</p>

    <div
      ref="viewportRef"
      class="sld-viewport"
      :class="{ 'sld-viewport--pan': spaceHeld || panning }"
      :aria-label="t('pluginWorkbench.layout.frameAria')"
      @wheel="onWheel"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <div class="sld-toolbar" @click.stop>
        <button type="button" class="sld-tb-btn" @click="zoomIn()">+</button>
        <button type="button" class="sld-tb-btn" @click="zoomOut()">−</button>
        <button type="button" class="sld-tb-btn" @click="resetView()">
          {{ t("pluginWorkbench.graph.resetView") }}
        </button>
        <button type="button" class="sld-tb-btn" @click="fitWorld()">
          {{ t("pluginWorkbench.graph.fitAll") }}
        </button>
        <span class="sld-tb-scale">{{ scalePercent }}</span>
      </div>

      <div class="sld-grid" :style="gridStyle" />

      <div
        class="sld-world"
        :style="{ transform: transformStyle, width: WORLD_W + 'px', height: WORLD_H + 'px' }"
      >
        <div
          v-for="reg in regions"
          :key="reg.id"
          class="sld-zone"
          :style="{ left: reg.x + 'px', top: reg.y + 'px', width: reg.w + 'px', height: reg.h + 'px' }"
        >
          <span class="sld-zone-lbl">{{ t(reg.labelKey) }}</span>
        </div>

        <div
          v-for="sl in diagramSlots"
          :key="sl.key"
          class="ui-node"
          :class="{
            'ui-node--empty': isEmpty(sl.key),
            'ui-node--selected': selectedSlot === sl.key,
            'ui-node--filled': !isEmpty(sl.key),
          }"
          :style="{ left: sl.x + 'px', top: sl.y + 'px', width: sl.w + 'px' }"
          @click.stop="selectSlot(sl.key)"
          @dblclick.stop="onSlotDblClick(sl)"
        >
          <div class="ui-node-bar" :class="{ 'ui-node-bar--empty': isEmpty(sl.key) }" />
          <div class="ui-node-body">
            <div class="ui-node-head">
              <span class="ui-node-key">{{ sl.key }}</span>
              <span
                class="ui-led"
                :class="isEmpty(sl.key) ? 'ui-led--off' : 'ui-led--on'"
              />
            </div>
            <p class="ui-node-desc">{{ t(sl.descKey) }}</p>
            <div v-if="!isEmpty(sl.key)" class="ui-bound">
              <span
                v-for="pid in orderedActive(sl.key)"
                :key="pid"
                class="ui-chip"
                :title="catalogEntry(pid)?.version ? 'v' + catalogEntry(pid)!.version : ''"
              >
                <span class="ui-chip-ico" aria-hidden="true">🧩</span>
                {{ pid }}
              </span>
            </div>
            <p v-else class="ui-empty-lbl">{{ t("pluginWorkbench.layout.emptySlot") }}</p>
            <label class="ui-select-wrap">
              <span class="ui-select-lbl">{{ t("pluginWorkbench.layout.bindPlugin") }}</span>
              <select
                class="ui-select"
                :value="orderedActive(sl.key)[0] ?? ''"
                :aria-label="t('pluginWorkbench.layout.selectAria', { slot: t(sl.labelKey) })"
                @click.stop
                @change="onSlotSelect(sl.key, $event)"
              >
                <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
                <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
                  {{ id }}
                </option>
              </select>
            </label>
          </div>
        </div>
      </div>
    </div>

    <details class="sld-contrib">
      <summary>{{ t("pluginWorkbench.layout.contribSummary") }}</summary>
      <ul class="sld-contrib-list">
        <li v-for="row in contributionRows" :key="row.id" class="sld-contrib-li">
          <strong>{{ row.id }}</strong>
          <span v-if="row.disabled" class="sld-muted"> · {{ t("pluginWorkbench.layout.pluginOff") }}</span>
          <span v-else-if="!row.slots.length" class="sld-muted">
            · {{ t("pluginWorkbench.layout.noSlots") }}
          </span>
          <span v-else class="sld-muted"> · {{ row.slots.join(", ") }}</span>
        </li>
      </ul>
    </details>
  </div>
</template>

<style scoped>
.sld-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.sld-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.sld-viewport {
  position: relative;
  height: min(480px, 52vh);
  min-height: 320px;
  overflow: hidden;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  touch-action: none;
}
.sld-viewport--pan {
  cursor: grab;
}
.sld-viewport--pan:active {
  cursor: grabbing;
}
.sld-grid {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: radial-gradient(
    circle,
    var(--graph-grid-color, color-mix(in srgb, var(--text-secondary) 35%, transparent)) var(--graph-grid-dot, 1px),
    transparent var(--graph-grid-dot, 1px)
  );
}
html[data-theme="dark"] .sld-grid {
  --graph-grid-color: color-mix(in srgb, #fff 20%, transparent);
}
html:not([data-theme="dark"]) .sld-grid {
  --graph-grid-color: color-mix(in srgb, #000 16%, transparent);
}
.sld-toolbar {
  position: absolute;
  z-index: 5;
  left: 10px;
  top: 10px;
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 92%, transparent);
}
.sld-tb-btn {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.sld-tb-scale {
  font-size: 11px;
  color: var(--text-secondary);
}
.sld-world {
  position: absolute;
  left: 0;
  top: 0;
  transform-origin: 0 0;
}
.sld-zone {
  position: absolute;
  border: 2px dashed color-mix(in srgb, var(--border-light) 85%, transparent);
  border-radius: 12px;
  pointer-events: none;
  background: color-mix(in srgb, var(--bg-primary) 40%, transparent);
}
.sld-zone-lbl {
  position: absolute;
  top: 6px;
  left: 10px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-secondary);
}
.ui-node {
  position: absolute;
  z-index: 2;
  border-radius: 8px;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.14);
  border: 1px solid var(--border-light);
  overflow: hidden;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    border-color 0.15s ease;
}
.ui-node:hover {
  transform: scale(1.02);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}
.ui-node--selected {
  border: 2px solid #2196f3;
  box-shadow: 0 0 0 2px color-mix(in srgb, #2196f3 22%, transparent);
}
.ui-node--empty {
  border-style: dashed;
  background: color-mix(in srgb, var(--bg-elevated) 80%, var(--bg-primary));
}
.ui-node-bar {
  height: 3px;
  background: var(--accent);
}
.ui-node-bar--empty {
  background: color-mix(in srgb, var(--text-secondary) 40%, transparent);
}
.ui-node-body {
  padding: 8px 10px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ui-node-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.ui-node-key {
  font-size: 11px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-weight: 600;
  color: var(--text-primary);
}
.ui-led {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.ui-led--on {
  background: #4caf50;
  box-shadow: 0 0 6px rgba(76, 175, 80, 0.7);
}
.ui-led--off {
  background: color-mix(in srgb, var(--text-secondary) 50%, transparent);
}
.ui-node-desc {
  margin: 0;
  font-size: 11px;
  color: var(--text-secondary);
}
.ui-bound {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.ui-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated));
  border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border-light));
  font-family: ui-monospace, monospace;
}
.ui-chip-ico {
  font-size: 10px;
}
.ui-empty-lbl {
  margin: 0;
  font-size: 11px;
  color: var(--text-secondary);
  font-style: italic;
}
.ui-select-wrap {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 2px;
}
.ui-select-lbl {
  font-size: 10px;
  color: var(--text-secondary);
}
.ui-select {
  width: 100%;
  padding: 5px 8px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sld-contrib {
  font-size: 12px;
  border-top: 1px dashed var(--border-light);
  padding-top: 8px;
}
.sld-contrib-list {
  margin: 8px 0 0;
  padding-left: 18px;
}
.sld-muted {
  color: var(--text-secondary);
}
</style>
