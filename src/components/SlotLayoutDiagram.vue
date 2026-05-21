<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import UiSlotMockRegion from "./UiSlotMockRegion.vue";
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

const activeSlot = ref<string | null>(SLOT_CHAT_HEADER);

const slotRows = [
  { key: SLOT_CHAT_HEADER, labelKey: "pluginWorkbench.layout.chatHeader", hintKey: "pluginWorkbench.layout.descHeader" },
  { key: SLOT_SIDEBAR, labelKey: "pluginWorkbench.layout.sidebar", hintKey: "pluginWorkbench.layout.descSidebar" },
  { key: SLOT_CHAT_TOOLBAR, labelKey: "pluginWorkbench.layout.chatToolbar", hintKey: "pluginWorkbench.layout.descToolbar", toolbar: true },
  { key: SLOT_ROLE_DETAIL, labelKey: "pluginWorkbench.layout.roleDetail", hintKey: "pluginWorkbench.layout.descRoleDetail" },
  { key: SLOT_OVERLAY_FLOATING, labelKey: "pluginWorkbench.layout.overlay", hintKey: "pluginWorkbench.layout.descOverlay" },
] as const;

function isContributionOff(slot: string, pluginId: string): boolean {
  if (slot === SLOT_CHAT_TOOLBAR) {
    return pluginStore.isToolbarContributionDisabled(pluginId);
  }
  return pluginStore.isSlotContributionDisabled(slot, pluginId);
}

function boundLabel(slot: string): string {
  const ids = pluginStore
    .pluginsOrderedForSlot(slot)
    .filter((id) => !pluginStore.isPluginDisabled(id) && !isContributionOff(slot, id));
  if (!ids.length) return t("pluginWorkbench.layout.none");
  return ids.join(", ");
}

const mapRows = computed(() =>
  slotRows.map((r) => ({
    key: r.key,
    label: t(r.labelKey),
    bound: boundLabel(r.key),
    empty: boundLabel(r.key) === t("pluginWorkbench.layout.none"),
  })),
);
</script>

<template>
  <div class="sld-root">
    <p class="sld-lead">{{ t("pluginWorkbench.layout.lead") }}</p>

    <div class="sld-workspace">
      <div class="sld-mock-wrap" :aria-label="t('pluginWorkbench.layout.frameAria')">
        <p class="sld-mock-caption">{{ t("pluginWorkbench.layout.mockCaption") }}</p>

        <div class="sld-mock-app">
          <header class="sld-mock-chrome">
            <span class="sld-mock-dot" />
            <span class="sld-mock-dot" />
            <span class="sld-mock-dot" />
            <span class="sld-mock-chrome-title">{{ t("pluginWorkbench.layout.mockAppTitle") }}</span>
          </header>

          <div class="sld-mock-main">
            <aside class="sld-mock-aside">
              <div class="sld-mock-role">
                <div class="sld-mock-avatar" aria-hidden="true" />
                <div class="sld-mock-role-lines">
                  <span class="sld-mock-line sld-mock-line--wide" />
                  <span class="sld-mock-line" />
                </div>
              </div>

              <UiSlotMockRegion
                :slot-key="SLOT_SIDEBAR"
                label-key="pluginWorkbench.layout.sidebar"
                hint-key="pluginWorkbench.layout.descSidebar"
                :active="activeSlot === SLOT_SIDEBAR"
                @select="activeSlot = SLOT_SIDEBAR"
              />

              <UiSlotMockRegion
                :slot-key="SLOT_ROLE_DETAIL"
                label-key="pluginWorkbench.layout.roleDetail"
                hint-key="pluginWorkbench.layout.descRoleDetail"
                :active="activeSlot === SLOT_ROLE_DETAIL"
                @select="activeSlot = SLOT_ROLE_DETAIL"
              />
            </aside>

            <section class="sld-mock-chat">
              <UiSlotMockRegion
                :slot-key="SLOT_CHAT_HEADER"
                label-key="pluginWorkbench.layout.chatHeader"
                hint-key="pluginWorkbench.layout.descHeader"
                :active="activeSlot === SLOT_CHAT_HEADER"
                @select="activeSlot = SLOT_CHAT_HEADER"
              />

              <UiSlotMockRegion
                :slot-key="SLOT_CHAT_TOOLBAR"
                label-key="pluginWorkbench.layout.chatToolbar"
                hint-key="pluginWorkbench.layout.descToolbar"
                toolbar-slot
                :active="activeSlot === SLOT_CHAT_TOOLBAR"
                @select="activeSlot = SLOT_CHAT_TOOLBAR"
              />

              <div class="sld-mock-messages" aria-hidden="true">
                <div class="sld-bubble sld-bubble--user" />
                <div class="sld-bubble sld-bubble--role" />
                <div class="sld-bubble sld-bubble--user sld-bubble--short" />
              </div>

              <div class="sld-mock-input" aria-hidden="true">
                <span class="sld-mock-input-ph">{{ t("pluginWorkbench.layout.mockInput") }}</span>
                <span class="sld-mock-send" />
              </div>
            </section>
          </div>

          <div class="sld-mock-overlay-wrap">
            <UiSlotMockRegion
              :slot-key="SLOT_OVERLAY_FLOATING"
              label-key="pluginWorkbench.layout.overlay"
              hint-key="pluginWorkbench.layout.descOverlay"
              :active="activeSlot === SLOT_OVERLAY_FLOATING"
              @select="activeSlot = SLOT_OVERLAY_FLOATING"
            />
          </div>
        </div>
      </div>

      <aside class="sld-map" :aria-label="t('pluginWorkbench.layout.mapAria')">
        <h4 class="sld-map-title">{{ t("pluginWorkbench.layout.mapTitle") }}</h4>
        <p class="sld-map-hint">{{ t("pluginWorkbench.layout.mapHint") }}</p>
        <table class="sld-map-table">
          <thead>
            <tr>
              <th>{{ t("pluginWorkbench.layout.mapColSlot") }}</th>
              <th>{{ t("pluginWorkbench.layout.mapColPlugin") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in mapRows"
              :key="row.key"
              :class="{ 'sld-map-row--active': activeSlot === row.key, 'sld-map-row--empty': row.empty }"
              @click="activeSlot = row.key"
            >
              <td>
                <code class="sld-map-code">{{ row.key }}</code>
                <span class="sld-map-zh">{{ row.label }}</span>
              </td>
              <td>{{ row.bound }}</td>
            </tr>
          </tbody>
        </table>
      </aside>
    </div>

    <details class="sld-contrib">
      <summary>{{ t("pluginWorkbench.layout.contribSummary") }}</summary>
      <ul class="sld-contrib-list">
        <li v-for="c in pluginStore.catalog" :key="c.id">
          <strong>{{ c.id }}</strong>
          <span class="sld-muted">
            · {{ (c.uiSlotNames ?? []).join(", ") || t("pluginWorkbench.layout.noSlots") }}
          </span>
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
  line-height: 1.45;
}
.sld-workspace {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(200px, 280px);
  gap: 14px;
  align-items: start;
}
@media (max-width: 860px) {
  .sld-workspace {
    grid-template-columns: 1fr;
  }
}
.sld-mock-wrap {
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 50%, var(--bg-primary));
  padding: 12px;
}
.sld-mock-caption {
  margin: 0 0 8px;
  font-size: 11px;
  color: var(--text-secondary);
}
.sld-mock-app {
  position: relative;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  min-height: 340px;
}
.sld-mock-chrome {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-secondary, var(--bg-elevated));
}
.sld-mock-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--text-secondary) 30%, transparent);
}
.sld-mock-chrome-title {
  margin-left: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}
.sld-mock-main {
  display: grid;
  grid-template-columns: minmax(148px, 32%) 1fr;
  min-height: 280px;
}
.sld-mock-aside {
  border-right: 1px solid var(--border-light);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: color-mix(in srgb, var(--bg-elevated) 40%, var(--bg-primary));
}
.sld-mock-role {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
}
.sld-mock-avatar {
  width: 40px;
  height: 48px;
  border-radius: 6px;
  background: linear-gradient(
    160deg,
    color-mix(in srgb, var(--accent) 25%, var(--bg-elevated)),
    var(--bg-elevated)
  );
  flex-shrink: 0;
}
.sld-mock-role-lines {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.sld-mock-line {
  height: 6px;
  border-radius: 3px;
  background: color-mix(in srgb, var(--text-secondary) 18%, var(--border-light));
  width: 70%;
}
.sld-mock-line--wide {
  width: 90%;
}
.sld-mock-chat {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.sld-mock-messages {
  flex: 1;
  min-height: 100px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px 4px;
  opacity: 0.55;
}
.sld-bubble {
  height: 28px;
  border-radius: 10px;
  max-width: 72%;
}
.sld-bubble--user {
  align-self: flex-end;
  background: color-mix(in srgb, var(--accent) 20%, var(--bg-elevated));
}
.sld-bubble--role {
  align-self: flex-start;
  background: var(--bg-elevated);
  border: 1px solid var(--border-light);
}
.sld-bubble--short {
  max-width: 48%;
}
.sld-mock-input {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  opacity: 0.7;
}
.sld-mock-input-ph {
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
}
.sld-mock-send {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: color-mix(in srgb, var(--accent) 35%, var(--bg-primary));
}
.sld-mock-overlay-wrap {
  position: absolute;
  right: 12px;
  bottom: 12px;
  width: min(220px, 42%);
  z-index: 2;
}
.sld-map {
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary, var(--bg-elevated));
  padding: 12px;
  font-size: 12px;
}
.sld-map-title {
  margin: 0 0 4px;
  font-size: 13px;
  font-weight: 600;
}
.sld-map-hint {
  margin: 0 0 10px;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.sld-map-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 11px;
}
.sld-map-table th {
  text-align: left;
  padding: 6px 4px;
  border-bottom: 1px solid var(--border-light);
  color: var(--text-secondary);
  font-weight: 600;
}
.sld-map-table td {
  padding: 8px 4px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 60%, transparent);
  vertical-align: top;
}
.sld-map-row--active td {
  background: color-mix(in srgb, #2196f3 10%, transparent);
}
.sld-map-row--empty td:last-child {
  color: var(--text-secondary);
  font-style: italic;
}
.sld-map-table tbody tr {
  cursor: pointer;
}
.sld-map-table tbody tr:hover td {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}
.sld-map-code {
  display: block;
  font-size: 10px;
  margin-bottom: 2px;
}
.sld-map-zh {
  font-size: 10px;
  color: var(--text-secondary);
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
