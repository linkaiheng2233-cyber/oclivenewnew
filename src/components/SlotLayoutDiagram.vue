<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  SLOT_CHAT_HEADER,
  SLOT_CHAT_TOOLBAR,
  SLOT_OVERLAY_FLOATING,
  SLOT_ROLE_DETAIL,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";

const pluginStore = usePluginStore();
const { t } = useI18n();

type DiagramSlot = {
  key: string;
  labelKey: string;
  region: "header" | "sidebar" | "chat-toolbar" | "role-detail" | "overlay";
};

const diagramSlots: DiagramSlot[] = [
  { key: SLOT_CHAT_HEADER, labelKey: "pluginWorkbench.layout.chatHeader", region: "header" },
  { key: SLOT_SIDEBAR, labelKey: "pluginWorkbench.layout.sidebar", region: "sidebar" },
  { key: SLOT_CHAT_TOOLBAR, labelKey: "pluginWorkbench.layout.chatToolbar", region: "chat-toolbar" },
  { key: SLOT_ROLE_DETAIL, labelKey: "pluginWorkbench.layout.roleDetail", region: "role-detail" },
  { key: SLOT_OVERLAY_FLOATING, labelKey: "pluginWorkbench.layout.overlay", region: "overlay" },
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

function primaryPluginId(slot: string): string {
  const order = pluginStore.pluginsOrderedForSlot(slot);
  return order[0] ?? "";
}

function onPrimaryChange(slot: string, ev: Event) {
  const v = (ev.target as HTMLSelectElement).value;
  if (!v) {
    pluginStore.setSlotPluginIds(slot, []);
    return;
  }
  const rest = pluginStore
    .pluginsOrderedForSlot(slot)
    .filter((id) => id !== v);
  pluginStore.setSlotPluginIds(slot, [v, ...rest]);
}

const contributionRows = computed(() =>
  pluginStore.catalog.map((entry) => ({
    id: entry.id,
    slots: entry.uiSlotNames ?? [],
    disabled: pluginStore.isPluginDisabled(entry.id),
  })),
);
</script>

<template>
  <div class="sld-root">
    <p class="sld-lead">{{ t("pluginWorkbench.layout.lead") }}</p>

    <div class="sld-frame" :aria-label="t('pluginWorkbench.layout.frameAria')">
      <div class="sld-region sld-region--header">
        <span class="sld-region-lbl">{{ t("pluginWorkbench.layout.regionHeader") }}</span>
        <div
          v-for="sl in diagramSlots.filter((s) => s.region === 'header')"
          :key="sl.key"
          class="sld-slot"
        >
          <span class="sld-slot-name">{{ t(sl.labelKey) }}</span>
          <select
            class="sld-select"
            :value="primaryPluginId(sl.key)"
            :aria-label="t('pluginWorkbench.layout.selectAria', { slot: t(sl.labelKey) })"
            @change="onPrimaryChange(sl.key, $event)"
          >
            <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
            <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
              {{ id }}
            </option>
          </select>
        </div>
      </div>

      <div class="sld-body">
        <div class="sld-region sld-region--sidebar">
          <span class="sld-region-lbl">{{ t("pluginWorkbench.layout.regionSidebar") }}</span>
          <div
            v-for="sl in diagramSlots.filter((s) => s.region === 'sidebar')"
            :key="sl.key"
            class="sld-slot"
          >
            <span class="sld-slot-name">{{ t(sl.labelKey) }}</span>
            <select
              class="sld-select"
              :value="primaryPluginId(sl.key)"
              @change="onPrimaryChange(sl.key, $event)"
            >
              <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
              <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
                {{ id }}
              </option>
            </select>
          </div>
        </div>

        <div class="sld-region sld-region--chat">
          <span class="sld-region-lbl">{{ t("pluginWorkbench.layout.regionChat") }}</span>
          <div
            v-for="sl in diagramSlots.filter((s) => s.region === 'chat-toolbar')"
            :key="sl.key"
            class="sld-slot"
          >
            <span class="sld-slot-name">{{ t(sl.labelKey) }}</span>
            <select
              class="sld-select"
              :value="primaryPluginId(sl.key)"
              @change="onPrimaryChange(sl.key, $event)"
            >
              <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
              <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
                {{ id }}
              </option>
            </select>
          </div>
          <div
            v-for="sl in diagramSlots.filter((s) => s.region === 'role-detail')"
            :key="sl.key"
            class="sld-slot sld-slot--nested"
          >
            <span class="sld-slot-name">{{ t(sl.labelKey) }}</span>
            <select
              class="sld-select"
              :value="primaryPluginId(sl.key)"
              @change="onPrimaryChange(sl.key, $event)"
            >
              <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
              <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
                {{ id }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <div class="sld-region sld-region--overlay">
        <span class="sld-region-lbl">{{ t("pluginWorkbench.layout.regionOverlay") }}</span>
        <div
          v-for="sl in diagramSlots.filter((s) => s.region === 'overlay')"
          :key="sl.key"
          class="sld-slot"
        >
          <span class="sld-slot-name">{{ t(sl.labelKey) }}</span>
          <select
            class="sld-select"
            :value="primaryPluginId(sl.key)"
            @change="onPrimaryChange(sl.key, $event)"
          >
            <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
            <option v-for="id in candidatesForSlot(sl.key)" :key="id" :value="id">
              {{ id }}
            </option>
          </select>
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
  gap: 12px;
}
.sld-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.sld-frame {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  padding: 12px;
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.sld-region {
  border: 1px dashed color-mix(in srgb, var(--border-light) 90%, transparent);
  border-radius: 8px;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sld-region-lbl {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.sld-body {
  display: grid;
  grid-template-columns: minmax(120px, 28%) 1fr;
  gap: 10px;
  min-height: 140px;
}
.sld-slot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
}
.sld-slot--nested {
  margin-top: 4px;
}
.sld-slot-name {
  font-size: 11px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  color: var(--text-secondary);
}
.sld-select {
  flex: 1 1 120px;
  min-width: 0;
  padding: 4px 8px;
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
.sld-contrib-li {
  margin-bottom: 4px;
}
.sld-muted {
  color: var(--text-secondary);
}
</style>
