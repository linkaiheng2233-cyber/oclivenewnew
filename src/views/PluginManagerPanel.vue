<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed, ref, watch } from "vue";
import PluginBackendSessionPanel from "../components/PluginBackendSessionPanel.vue";
import PluginDebugPanel from "../components/PluginDebugPanel.vue";
import PluginListItem from "../components/PluginListItem.vue";
import PmSlotRow from "../components/PmSlotRow.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import {
  SLOT_CHAT_HEADER,
  SLOT_CHAT_TOOLBAR,
  SLOT_DEBUG_DOCK,
  SLOT_LAUNCHER_PALETTE,
  SLOT_OVERLAY_FLOATING,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_ADVANCED,
  SLOT_SETTINGS_PANEL,
  SLOT_SETTINGS_PLUGINS,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { applyAuthorSuggestedPluginBackends } from "../utils/tauri-api";

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const { showToast } = useAppToast();

const batchMode = ref(false);
const batchSelected = ref<Record<string, boolean>>({});
/** 已安装列表中「调试台」折叠（所有目录插件均可展开；无 `process` 时仅无法在此启动子进程） */
const pluginDebugOpen = ref<Record<string, boolean>>({});

function togglePluginDebug(id: string) {
  pluginDebugOpen.value = { ...pluginDebugOpen.value, [id]: !pluginDebugOpen.value[id] };
}

function clearBatchSelection(): void {
  batchSelected.value = {};
}

watch(batchMode, (v) => {
  if (!v) {
    clearBatchSelection();
  }
});

watch(
  () => pluginStore.catalog.map((c) => c.id).join("\n"),
  () => {
    const next: Record<string, boolean> = {};
    for (const p of pluginStore.catalog) {
      if (batchSelected.value[p.id]) {
        next[p.id] = true;
      }
    }
    batchSelected.value = next;
  },
);

const batchSelectedCount = computed(
  () => Object.values(batchSelected.value).filter(Boolean).length,
);

const batchSelectedIds = computed(() =>
  Object.entries(batchSelected.value)
    .filter(([, v]) => v)
    .map(([k]) => k),
);

function setBatchSelected(id: string, v: boolean): void {
  batchSelected.value = { ...batchSelected.value, [id]: v };
}

async function onBatchEnable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    pluginStore.batchEnablePluginIds(ids);
    showToast("success", `已启用 ${ids.length} 个插件；保存后生效，建议重启应用。`);
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onBatchDisable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  pluginStore.batchDisablePluginIds(ids);
  showToast("success", `已停用 ${ids.length} 个插件；保存后生效，建议重启应用。`);
  clearBatchSelection();
}

function onPluginDisabledRow(id: string, disabled: boolean): void {
  try {
    pluginStore.setPluginDisabled(id, disabled);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onBatchUpdate() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    const r = await pluginStore.batchUpdatePluginIds(ids);
    if (r.count === 0) {
      showToast("info", "所选插件当前无在线更新记录；请使用「从本地 zip 更新」。");
    } else {
      showToast(
        "info",
        `检测到 ${r.count} 个插件可能有更新（${r.targets.join("、")}）。请分别使用「从本地 zip 更新」导入包。`,
      );
    }
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const toolbarOrder = computed(() => pluginStore.toolbarPluginsOrdered());
const settingsPanelOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PANEL),
);
const roleDetailOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_ROLE_DETAIL));
const sidebarOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_SIDEBAR));
const chatHeaderOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_CHAT_HEADER));
const settingsPluginsOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PLUGINS),
);
const settingsAdvancedOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_ADVANCED),
);
const overlayFloatingOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_OVERLAY_FLOATING),
);
const launcherPaletteOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_LAUNCHER_PALETTE),
);
const debugDockOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_DEBUG_DOCK));

let dragSlot: { slot: string; index: number } | null = null;

function onDragSlotStart(slot: string, index: number) {
  dragSlot = { slot, index };
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDropSlot(slot: string, index: number) {
  if (!dragSlot || dragSlot.slot !== slot) {
    dragSlot = null;
    return;
  }
  if (dragSlot.index === index) {
    dragSlot = null;
    return;
  }
  pluginStore.movePluginInSlotOrder(slot, dragSlot.index, index);
  dragSlot = null;
}

async function onSave() {
  try {
    await pluginStore.persist();
    showToast("success", "已保存插件配置；停用插件建议重启应用后完全生效。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onResetToPackDefault() {
  try {
    if (pluginStore.persistScope === "global") {
      pluginStore.setPersistScope("role");
    }
    await pluginStore.resetToRolePackDefault();
    showToast(
      "success",
      "已重置为当前角色包推荐布局（author.suggested_ui 优先，否则 ui.json）。",
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onApplyAuthorSuggestedBackends() {
  try {
    const info = await applyAuthorSuggestedPluginBackends(roleStore.currentRoleId);
    roleStore.applyRoleInfo(info);
    showToast("success", "已应用 author.json 中的 suggested_plugin_backends（会话级，未改 settings.json）。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onCheckUpdates() {
  try {
    await pluginStore.checkPluginUpdatesFromRegistry();
    if (pluginStore.error) {
      showToast("error", pluginStore.error);
    } else {
      showToast("success", "检查完成（在线版本接口预留中）。");
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateFromZip(pluginId: string) {
  const path = await open({
    multiple: false,
    filters: [{ name: "Zip", extensions: ["zip"] }],
  });
  if (path === null || Array.isArray(path)) {
    return;
  }
  try {
    await pluginStore.installPluginFromLocalZip(pluginId, path);
    showToast("success", "更新完成，请重启应用生效。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="pluginStore.panelVisible"
      class="pm-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件工作台（专业模式）"
      @click.self="pluginStore.closePanel()"
    >
      <div class="pm-dialog pm-dialog--studio" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">插件工作台</h2>
            <span
              class="pm-studio-badge"
              title="面向创作者与排错：目录插件、后端与会话覆盖"
            >专业模式</span>
          </div>
          <p class="pm-sub">
            <kbd class="pm-kbd">Ctrl</kbd>+<kbd class="pm-kbd">Shift</kbd>+<kbd class="pm-kbd">F</kbd>
            开关本窗口 · 保存后插槽/启用状态建议重启应用生效
          </p>
          <button type="button" class="pm-close" aria-label="关闭" @click="pluginStore.closePanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted pm-dialog-pad">加载中…</div>
        <p v-else-if="pluginStore.error" class="pm-err pm-dialog-pad">{{ pluginStore.error }}</p>

        <template v-else>
          <div class="pm-tabs" role="tablist" aria-label="插件工作台分区">
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'plugins' }"
              :aria-selected="pluginStore.panelMainTab === 'plugins'"
              @click="pluginStore.panelMainTab = 'plugins'"
            >
              界面插件
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'backends' }"
              :aria-selected="pluginStore.panelMainTab === 'backends'"
              @click="pluginStore.panelMainTab = 'backends'"
            >
              后端模块
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'slots' }"
              :aria-selected="pluginStore.panelMainTab === 'slots'"
              @click="pluginStore.panelMainTab = 'slots'"
            >
              插槽顺序
            </button>
          </div>

          <div class="pm-scroll">
          <div
            v-show="pluginStore.panelMainTab === 'plugins'"
            class="pm-tab-panel"
            role="tabpanel"
          >
          <section class="pm-section">
            <h3 class="pm-h3">保存目标</h3>
            <p class="pm-hint">
              「全局默认」对所有角色生效并与各角色设置合并（整壳与插槽以当前角色为准；全局禁用插件为并集）。
            </p>
            <div class="pm-scope-row" role="group" aria-label="插件配置保存范围">
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'role'"
                  @change="pluginStore.setPersistScope('role')"
                />
                仅当前角色
              </label>
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'global'"
                  @change="pluginStore.setPersistScope('global')"
                />
                全局默认
              </label>
            </div>
          </section>

          <section
            v-if="roleStore.roleInfo.authorPack?.suggested_plugin_backends"
            class="pm-section"
          >
            <h3 class="pm-h3">作者建议 · 后端</h3>
            <p class="pm-hint">
              将 author.json 中的 suggested_plugin_backends 写入本会话的后端覆盖（与「后端模块」Tab 一致）。
            </p>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="onApplyAuthorSuggestedBackends"
            >
              应用作者建议的后端
            </button>
          </section>

          <section v-if="roleStore.roleInfo.authorPack" class="pm-section">
            <h3 class="pm-h3">作者与推荐</h3>
            <p v-if="roleStore.roleInfo.authorPack.summary" class="pm-author-summary">
              {{ roleStore.roleInfo.authorPack.summary }}
            </p>
            <ul
              v-if="(roleStore.roleInfo.authorPack.recommended_plugins ?? []).length"
              class="pm-rec-list"
            >
              <li
                v-for="(rp, idx) in roleStore.roleInfo.authorPack.recommended_plugins"
                :key="`${rp.id}-${idx}`"
              >
                <strong>{{ rp.id }}</strong>
                <span v-if="rp.version_range" class="pm-muted"> · {{ rp.version_range }}</span>
                <span v-if="rp.optional" class="pm-muted">（可选）</span>
              </li>
            </ul>
            <p v-else class="pm-muted">未列出 recommended_plugins。</p>
          </section>

          <section class="pm-section pm-section--catalog">
            <div class="pm-section-head">
              <h3 class="pm-h3">已安装插件</h3>
              <div class="pm-section-actions">
                <label class="pm-batch-toggle chk">
                  <input v-model="batchMode" type="checkbox" />
                  批量选择
                </label>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginUpdatesCheckLoading"
                  @click="onCheckUpdates"
                >
                  检查更新
                </button>
              </div>
            </div>
            <div
              v-if="batchMode && batchSelectedCount > 0"
              class="pm-batch-bar"
              role="toolbar"
              aria-label="批量操作"
            >
              <span class="pm-batch-count">已选 {{ batchSelectedCount }} 个</span>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                批量启用
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                批量停用
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchUpdate">
                批量检查更新
              </button>
            </div>
            <ul class="pm-list">
              <li v-for="p in pluginStore.catalog" :key="p.id" class="pm-plugin-row">
                <PluginListItem
                  :entry="p"
                  :batch-select-mode="batchMode"
                  :batch-selected="!!batchSelected[p.id]"
                  @update:batch-selected="setBatchSelected(p.id, $event)"
                  :plugin-disabled="pluginStore.isPluginDisabled(p.id)"
                  :toolbar-contribution-disabled="pluginStore.isToolbarContributionDisabled(p.id)"
                  :settings-panel-contribution-disabled="
                    pluginStore.isSlotContributionDisabled(SLOT_SETTINGS_PANEL, p.id)
                  "
                  :role-detail-contribution-disabled="
                    pluginStore.isSlotContributionDisabled(SLOT_ROLE_DETAIL, p.id)
                  "
                  :sidebar-contribution-disabled="
                    pluginStore.isSlotContributionDisabled(SLOT_SIDEBAR, p.id)
                  "
                  :chat-header-contribution-disabled="
                    pluginStore.isSlotContributionDisabled(SLOT_CHAT_HEADER, p.id)
                  "
                  @update:plugin-disabled="onPluginDisabledRow(p.id, $event)"
                  @update:toolbar-contribution-disabled="
                    pluginStore.setToolbarContributionDisabled(p.id, $event)
                  "
                  @update:settings-panel-contribution-disabled="
                    pluginStore.setSlotContributionDisabled(SLOT_SETTINGS_PANEL, p.id, $event)
                  "
                  @update:role-detail-contribution-disabled="
                    pluginStore.setSlotContributionDisabled(SLOT_ROLE_DETAIL, p.id, $event)
                  "
                  @update:sidebar-contribution-disabled="
                    pluginStore.setSlotContributionDisabled(SLOT_SIDEBAR, p.id, $event)
                  "
                  @update:chat-header-contribution-disabled="
                    pluginStore.setSlotContributionDisabled(SLOT_CHAT_HEADER, p.id, $event)
                  "
                />
                <div class="pm-plugin-actions">
                  <span
                    v-if="pluginStore.pluginUpdateById[p.id]?.hasUpdate"
                    class="pm-badge"
                  >有新版本</span>
                  <button
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="togglePluginDebug(p.id)"
                  >
                    {{ pluginDebugOpen[p.id] ? "▼" : "▶" }} 调试台
                  </button>
                  <button
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    :disabled="pluginStore.extractingPluginId === p.id"
                    @click="onUpdateFromZip(p.id)"
                  >
                    从本地 zip 更新
                  </button>
                </div>
                <div v-if="pluginDebugOpen[p.id]" class="pm-plugin-dev-wrap">
                  <PluginDebugPanel
                    :plugin-id="p.id"
                    :expanded="true"
                    :spawn-supported="p.hasRpcProcess"
                  />
                </div>
              </li>
            </ul>
            <p v-if="!pluginStore.catalog.length" class="pm-muted">未扫描到目录插件（请将插件放入 roles 同级的 plugins/ 等目录）。</p>
          </section>
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'backends'"
            class="pm-tab-panel pm-tab-panel--backends"
            role="tabpanel"
          >
            <PluginBackendSessionPanel />
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'slots'"
            class="pm-tab-panel pm-tab-panel--slots"
            role="tabpanel"
          >
          <section class="pm-section pm-embed-slot">
            <h3 class="pm-h3">插件管理页预览（只读）</h3>
            <p class="pm-hint">
              与下方「settings.plugins」为同一插槽；预览不可操作，请在列表中拖拽排序。
            </p>
            <div class="pm-embed-preview" aria-hidden="true">
              <PluginSlotEmbed
                slot-name="settings.plugins"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.plugins 顺序</h3>
            <p class="pm-hint">本页内嵌区；拖拽排序，可选外观。</p>
            <ol class="pm-order" aria-label="插件管理页槽顺序">
              <li
                v-for="(id, i) in settingsPluginsOrder"
                :key="`spl-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PLUGINS, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PLUGINS, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PLUGINS" />
              </li>
            </ol>
            <p v-if="!settingsPluginsOrder.length" class="pm-muted">当前无 settings.plugins 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">chat_toolbar 顺序</h3>
            <p class="pm-hint">拖拽排序；仅含声明了该插槽的非整壳插件。</p>
            <ol class="pm-order" aria-label="工具栏插件顺序">
              <li
                v-for="(id, i) in toolbarOrder"
                :key="id"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_TOOLBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_TOOLBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_TOOLBAR" />
              </li>
            </ol>
            <p v-if="!toolbarOrder.length" class="pm-muted">当前无 chat_toolbar 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.panel 顺序</h3>
            <p class="pm-hint">设置页「插件扩展」中的嵌入顺序；拖拽排序。</p>
            <ol class="pm-order" aria-label="设置页插件顺序">
              <li
                v-for="(id, i) in settingsPanelOrder"
                :key="`sp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PANEL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PANEL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PANEL" />
              </li>
            </ol>
            <p v-if="!settingsPanelOrder.length" class="pm-muted">当前无 settings.panel 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">role.detail 顺序</h3>
            <p class="pm-hint">左侧角色详情区（立绘下方）嵌入顺序。</p>
            <ol class="pm-order" aria-label="角色详情插件顺序">
              <li
                v-for="(id, i) in roleDetailOrder"
                :key="`rd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_ROLE_DETAIL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_ROLE_DETAIL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_ROLE_DETAIL" />
              </li>
            </ol>
            <p v-if="!roleDetailOrder.length" class="pm-muted">当前无 role.detail 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">sidebar 顺序</h3>
            <p class="pm-hint">左侧栏角色块下方扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="侧边栏插件顺序">
              <li
                v-for="(id, i) in sidebarOrder"
                :key="`sb-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SIDEBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SIDEBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SIDEBAR" />
              </li>
            </ol>
            <p v-if="!sidebarOrder.length" class="pm-muted">当前无 sidebar 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">chat.header 顺序</h3>
            <p class="pm-hint">聊天列顶部（消息列表上方）；拖拽排序。</p>
            <ol class="pm-order" aria-label="聊天头部插件顺序">
              <li
                v-for="(id, i) in chatHeaderOrder"
                :key="`ch-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_HEADER, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_HEADER, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_HEADER" />
              </li>
            </ol>
            <p v-if="!chatHeaderOrder.length" class="pm-muted">当前无 chat.header 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">settings.advanced 顺序</h3>
            <p class="pm-hint">设置对话框「常规」扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="settings.advanced 顺序">
              <li
                v-for="(id, i) in settingsAdvancedOrder"
                :key="`sa-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_ADVANCED, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_ADVANCED, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_ADVANCED" />
              </li>
            </ol>
            <p v-if="!settingsAdvancedOrder.length" class="pm-muted">当前无 settings.advanced 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">overlay.floating 顺序</h3>
            <p class="pm-hint">主界面右下角浮层模板区；拖拽排序。</p>
            <ol class="pm-order" aria-label="overlay.floating 顺序">
              <li
                v-for="(id, i) in overlayFloatingOrder"
                :key="`of-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_OVERLAY_FLOATING, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_OVERLAY_FLOATING, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_OVERLAY_FLOATING" />
              </li>
            </ol>
            <p v-if="!overlayFloatingOrder.length" class="pm-muted">当前无 overlay.floating 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">launcher.palette 顺序</h3>
            <p class="pm-hint">快捷键说明浮层内聚合区；拖拽排序。</p>
            <ol class="pm-order" aria-label="launcher.palette 顺序">
              <li
                v-for="(id, i) in launcherPaletteOrder"
                :key="`lp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_LAUNCHER_PALETTE, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_LAUNCHER_PALETTE, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_LAUNCHER_PALETTE" />
              </li>
            </ol>
            <p v-if="!launcherPaletteOrder.length" class="pm-muted">当前无 launcher.palette 插槽插件。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">debug.dock 顺序</h3>
            <p class="pm-hint">调试面板内扩展区；拖拽排序。</p>
            <ol class="pm-order" aria-label="debug.dock 顺序">
              <li
                v-for="(id, i) in debugDockOrder"
                :key="`dd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_DEBUG_DOCK, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_DEBUG_DOCK, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_DEBUG_DOCK" />
              </li>
            </ol>
            <p v-if="!debugDockOrder.length" class="pm-muted">当前无 debug.dock 插槽插件。</p>
          </section>
          </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="pluginStore.closePanel()">关闭</button>
            <button type="button" class="pm-btn secondary" @click="onResetToPackDefault">
              重置为角色包推荐
            </button>
            <button type="button" class="pm-btn primary" @click="onSave">保存</button>
          </footer>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10050;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 45%, transparent));
}
.pm-dialog {
  position: relative;
  width: min(680px, 100%);
  max-height: min(88vh, 760px);
  overflow: auto;
  padding: 16px 18px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog--studio {
  width: min(1080px, 100%);
  max-height: min(92vh, 900px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog-pad {
  padding: 12px 18px;
}
.pm-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px 18px 8px;
}
.pm-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
  padding: 0 18px 10px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-tab {
  flex: 1 1 auto;
  min-width: 0;
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
  transition: var(--control-transition, border-color 0.18s ease, background 0.18s ease);
}
.pm-tab:hover {
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
.pm-tab-panel {
  min-height: 0;
}
.pm-embed-preview {
  pointer-events: none;
  user-select: none;
  opacity: 0.97;
  border-radius: var(--radius-card);
  overflow: hidden;
  border: 1px dashed color-mix(in srgb, var(--border-light) 85%, transparent);
}
.pm-head {
  flex-shrink: 0;
  padding: 16px 40px 12px 18px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-head-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.pm-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}
.pm-studio-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  color: var(--text-accent);
  background: color-mix(in srgb, var(--accent) 12%, var(--bg-elevated));
}
.pm-sub {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-kbd {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 2px;
  font-size: 11px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.pm-close {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.pm-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.pm-section {
  margin-bottom: 18px;
}
.pm-section--catalog {
  padding: 12px 14px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  box-shadow: var(--shadow-sm);
}
.pm-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}
.pm-section-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}
.pm-batch-toggle {
  font-size: 12px;
  user-select: none;
}
.pm-batch-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px dashed var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
}
.pm-batch-count {
  margin-right: 4px;
  color: var(--text-secondary);
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.pm-h3 {
  margin: 0;
  font-size: 14px;
}
.pm-plugin-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-sm);
}
.pm-plugin-row:hover {
  border-color: color-mix(in srgb, var(--accent) 22%, var(--border-light));
}
.pm-plugin-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding-left: 2px;
}
.pm-plugin-dev-wrap {
  width: 100%;
  min-width: 0;
}
.pm-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated));
  color: var(--text-primary);
}
.pm-btn--sm {
  padding: 5px 10px;
  font-size: 12px;
}
.pm-hint {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm-order {
  margin: 0;
  padding-left: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-order-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  font-size: 13px;
  border: 1px dashed var(--border-light);
  border-radius: var(--radius-btn);
  cursor: grab;
  background: var(--bg-elevated);
}
.pm-order-item--row {
  flex-wrap: wrap;
}
.pm-order-id {
  flex: 1;
  min-width: 0;
  word-break: break-all;
}
.pm-embed-slot code {
  font-size: 11px;
}
.pm-grip {
  color: var(--text-secondary);
  font-size: 12px;
  user-select: none;
}
.pm-scope-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 6px;
}
.pm-scope-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}
.pm-author-summary {
  margin: 0 0 8px;
  font-size: 13px;
  line-height: 1.45;
}
.pm-rec-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.5;
}
.pm-muted {
  font-size: 13px;
  color: var(--text-secondary);
}
.pm-err {
  color: var(--error);
  font-size: 13px;
}
.pm-foot {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  margin: 0;
  padding: 12px 18px;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-btn {
  padding: 8px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  font-size: 13px;
  cursor: pointer;
}
.pm-btn.secondary {
  background: transparent;
}
.pm-btn.primary {
  background: var(--accent);
  color: var(--bg-elevated);
  border-color: color-mix(in srgb, var(--accent) 85%, var(--text-primary) 15%);
}
.pm-btn.primary:hover {
  filter: brightness(1.05);
}
</style>
