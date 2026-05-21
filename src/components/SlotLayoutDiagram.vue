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
    hint: t(r.hintKey),
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

        <!-- 对齐 App.vue：app-frame → top-bar + main-content + app-floating-slot -->
        <div class="sld-app-frame">
          <header class="sld-top-bar">
            <div class="sld-top-bar-row">
              <div class="sld-role-pill">
                <span class="sld-role-pill-avatar" aria-hidden="true" />
                <span class="sld-role-pill-name">{{ t("pluginWorkbench.layout.mockRoleName") }}</span>
              </div>
              <span class="sld-more-btn">{{ t("app.more.more") }}</span>
            </div>
          </header>

          <div class="sld-main-content">
            <div class="sld-split-row">
              <aside class="sld-left-pane">
                <div class="sld-character-block">
                  <div class="sld-portrait" aria-hidden="true" />
                  <div class="sld-char-meta">
                    <span class="sld-char-name">{{ t("pluginWorkbench.layout.mockRoleName") }}</span>
                    <span class="sld-char-emotion">{{ t("pluginWorkbench.layout.mockEmotion") }}</span>
                  </div>
                </div>

                <UiSlotMockRegion
                  variant="default"
                  :slot-key="SLOT_ROLE_DETAIL"
                  label-key="pluginWorkbench.layout.roleDetail"
                  hint-key="pluginWorkbench.layout.descRoleDetail"
                  :active="activeSlot === SLOT_ROLE_DETAIL"
                  @select="activeSlot = SLOT_ROLE_DETAIL"
                />

                <div class="sld-aside-filler" aria-hidden="true">
                  <span class="sld-aside-line" />
                </div>

                <UiSlotMockRegion
                  variant="sidebar"
                  :slot-key="SLOT_SIDEBAR"
                  label-key="pluginWorkbench.layout.sidebar"
                  hint-key="pluginWorkbench.layout.descSidebar"
                  :active="activeSlot === SLOT_SIDEBAR"
                  @select="activeSlot = SLOT_SIDEBAR"
                />

                <div class="sld-fav-footer">
                  {{ t("pluginWorkbench.layout.mockFavorability") }}
                </div>
              </aside>

              <div class="sld-right-pane">
                <UiSlotMockRegion
                  variant="headerStrip"
                  :slot-key="SLOT_CHAT_HEADER"
                  label-key="pluginWorkbench.layout.chatHeader"
                  hint-key="pluginWorkbench.layout.descHeader"
                  :active="activeSlot === SLOT_CHAT_HEADER"
                  @select="activeSlot = SLOT_CHAT_HEADER"
                />

                <div class="sld-chat-scroll" aria-hidden="true">
                  <div class="sld-bubble sld-bubble--role sld-bubble--tall" />
                  <div class="sld-bubble sld-bubble--user" />
                  <div class="sld-bubble sld-bubble--role" />
                  <div class="sld-bubble sld-bubble--user sld-bubble--short" />
                </div>

                <section class="sld-input-area">
                  <div class="sld-toolbar-row">
                    <UiSlotMockRegion
                      variant="toolbar"
                      :slot-key="SLOT_CHAT_TOOLBAR"
                      label-key="pluginWorkbench.layout.chatToolbar"
                      hint-key="pluginWorkbench.layout.descToolbar"
                      toolbar-slot
                      :active="activeSlot === SLOT_CHAT_TOOLBAR"
                      @select="activeSlot = SLOT_CHAT_TOOLBAR"
                    />
                  </div>
                  <div class="sld-chat-input">
                    <span class="sld-chat-input-ph">{{ t("pluginWorkbench.layout.mockInput") }}</span>
                    <span class="sld-chat-send" />
                  </div>
                </section>
              </div>
            </div>
          </div>

          <div class="sld-floating-slot">
            <UiSlotMockRegion
              variant="overlay"
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
                <span class="sld-map-pos">{{ row.hint }}</span>
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
  grid-template-columns: minmax(0, 1fr) minmax(220px, 300px);
  gap: 14px;
  align-items: start;
}
@media (max-width: 900px) {
  .sld-workspace {
    grid-template-columns: 1fr;
  }
}
.sld-mock-wrap {
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--shell-page-bg, color-mix(in srgb, var(--bg-elevated) 50%, var(--bg-primary)));
  padding: 12px;
}
.sld-mock-caption {
  margin: 0 0 8px;
  font-size: 11px;
  color: var(--text-secondary);
}
.sld-app-frame {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 400px;
  border-radius: var(--radius-app, 12px);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app, 0 4px 20px rgba(0, 0, 0, 0.08));
  overflow: hidden;
}
.sld-top-bar {
  flex-shrink: 0;
  padding: 10px 14px 12px;
  background: color-mix(in srgb, var(--bg-secondary) 92%, var(--accent) 8%);
  border-bottom: 1px solid var(--border-light);
  border-left: 3px solid var(--rail-accent-runtime, var(--accent));
  box-shadow: 0 1px 0 color-mix(in srgb, var(--accent) 12%, transparent);
}
.sld-top-bar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.sld-role-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px 4px 4px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sld-role-pill-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent) 30%, var(--bg-primary));
}
.sld-role-pill-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}
.sld-more-btn {
  font-size: 12px;
  font-weight: 600;
  padding: 6px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.sld-main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.sld-split-row {
  flex: 1;
  min-height: 320px;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
}
.sld-left-pane {
  flex: 0 0 clamp(148px, 30%, 200px);
  max-width: 36%;
  min-width: 140px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 10px 0;
  border-right: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-secondary) 96%, var(--accent) 4%);
  box-shadow: inset -1px 0 0 color-mix(in srgb, var(--border-light) 65%, transparent);
  overflow-y: auto;
}
.sld-character-block {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px 4px;
}
.sld-portrait {
  width: 100%;
  max-width: 120px;
  aspect-ratio: 3 / 4;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: linear-gradient(
    165deg,
    color-mix(in srgb, var(--accent) 22%, var(--bg-elevated)),
    var(--bg-elevated)
  );
}
.sld-char-meta {
  width: 100%;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.sld-char-name {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-primary);
}
.sld-char-emotion {
  font-size: 10px;
  color: var(--text-secondary);
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--bg-elevated);
  align-self: center;
}
.sld-aside-filler {
  flex-shrink: 0;
  padding: 4px 0;
  opacity: 0.5;
}
.sld-aside-line {
  display: block;
  height: 4px;
  width: 80%;
  margin: 0 auto;
  border-radius: 2px;
  background: color-mix(in srgb, var(--text-secondary) 15%, var(--border-light));
}
.sld-fav-footer {
  flex-shrink: 0;
  margin-top: auto;
  padding: 10px 8px 12px;
  font-size: 11px;
  color: var(--text-secondary);
  text-align: center;
  border-top: 1px solid var(--border-light);
  background: var(--bg-status, var(--bg-elevated));
}
.sld-right-pane {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
.sld-chat-scroll {
  flex: 1;
  min-height: 120px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 16px 16px;
  overflow: hidden;
  opacity: 0.6;
}
.sld-bubble {
  border-radius: 12px;
  max-width: 78%;
  min-height: 24px;
}
.sld-bubble--tall {
  min-height: 48px;
}
.sld-bubble--role {
  align-self: flex-start;
  background: var(--bg-elevated);
  border: 1px solid var(--border-light);
}
.sld-bubble--user {
  align-self: flex-end;
  background: color-mix(in srgb, var(--accent) 18%, var(--bg-elevated));
}
.sld-bubble--short {
  max-width: 42%;
}
.sld-input-area {
  flex-shrink: 0;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: 0 -2px 14px color-mix(in srgb, var(--text-primary) 8%, transparent);
}
.sld-toolbar-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 6px 12px 0;
}
.sld-chat-input {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 12px 10px;
  padding: 8px 10px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  opacity: 0.75;
}
.sld-chat-input-ph {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
}
.sld-chat-send {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-btn);
  background: color-mix(in srgb, var(--accent) 40%, var(--bg-primary));
  flex-shrink: 0;
}
.sld-floating-slot {
  position: absolute;
  right: 10px;
  bottom: 10px;
  z-index: 3;
  width: min(200px, 44%);
  pointer-events: auto;
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
  display: block;
  font-size: 11px;
  font-weight: 600;
}
.sld-map-pos {
  display: block;
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 2px;
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
