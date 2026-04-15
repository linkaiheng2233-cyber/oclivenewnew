<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { computed } from "vue";
import PluginListItem from "../components/PluginListItem.vue";
import { useAppToast } from "../composables/useAppToast";
import {
  SLOT_CHAT_TOOLBAR,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_PANEL,
  usePluginStore,
} from "../stores/pluginStore";

const pluginStore = usePluginStore();
const { showToast } = useAppToast();

const toolbarOrder = computed(() => pluginStore.toolbarPluginsOrdered());
const settingsPanelOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PANEL),
);
const roleDetailOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_ROLE_DETAIL));

let dragToolbarIndex: number | null = null;
let dragSettingsIndex: number | null = null;
let dragRoleDetailIndex: number | null = null;

function onDragToolbarStart(i: number) {
  dragToolbarIndex = i;
}

function onDragSettingsStart(i: number) {
  dragSettingsIndex = i;
}

function onDragRoleDetailStart(i: number) {
  dragRoleDetailIndex = i;
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDropToolbar(i: number) {
  if (dragToolbarIndex === null || dragToolbarIndex === i) {
    return;
  }
  pluginStore.moveToolbarPlugin(dragToolbarIndex, i);
  dragToolbarIndex = null;
}

function onDropSettings(i: number) {
  if (dragSettingsIndex === null || dragSettingsIndex === i) {
    return;
  }
  pluginStore.movePluginInSlotOrder(SLOT_SETTINGS_PANEL, dragSettingsIndex, i);
  dragSettingsIndex = null;
}

function onDropRoleDetail(i: number) {
  if (dragRoleDetailIndex === null || dragRoleDetailIndex === i) {
    return;
  }
  pluginStore.movePluginInSlotOrder(SLOT_ROLE_DETAIL, dragRoleDetailIndex, i);
  dragRoleDetailIndex = null;
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
    await pluginStore.resetToRolePackDefault();
    showToast("success", "已重置为当前角色包 ui.json 的推荐布局。");
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
      aria-label="插件管理"
      @click.self="pluginStore.closePanel()"
    >
      <div class="pm-dialog" @click.stop>
        <header class="pm-head">
          <h2 class="pm-title">插件管理</h2>
          <p class="pm-sub">快捷键 Ctrl+Shift+F · 停用插件与工具栏顺序写入本地配置；停用后建议重启应用以完全释放进程。</p>
          <button type="button" class="pm-close" aria-label="关闭" @click="pluginStore.closePanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted">加载中…</div>
        <p v-else-if="pluginStore.error" class="pm-err">{{ pluginStore.error }}</p>

        <template v-else>
          <section class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">已安装插件</h3>
              <button
                type="button"
                class="pm-btn secondary pm-btn--sm"
                :disabled="pluginStore.pluginUpdatesCheckLoading"
                @click="onCheckUpdates"
              >
                检查更新
              </button>
            </div>
            <ul class="pm-list">
              <li v-for="p in pluginStore.catalog" :key="p.id" class="pm-plugin-row">
                <PluginListItem
                  :entry="p"
                  :plugin-disabled="pluginStore.isPluginDisabled(p.id)"
                  :toolbar-contribution-disabled="pluginStore.isToolbarContributionDisabled(p.id)"
                  :settings-panel-contribution-disabled="
                    pluginStore.isSlotContributionDisabled(SLOT_SETTINGS_PANEL, p.id)
                  "
                  @update:plugin-disabled="pluginStore.setPluginDisabled(p.id, $event)"
                  @update:toolbar-contribution-disabled="
                    pluginStore.setToolbarContributionDisabled(p.id, $event)
                  "
                  @update:settings-panel-contribution-disabled="
                    pluginStore.setSlotContributionDisabled(SLOT_SETTINGS_PANEL, p.id, $event)
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
                    :disabled="pluginStore.extractingPluginId === p.id"
                    @click="onUpdateFromZip(p.id)"
                  >
                    从本地 zip 更新
                  </button>
                </div>
              </li>
            </ul>
            <p v-if="!pluginStore.catalog.length" class="pm-muted">未扫描到目录插件（请将插件放入 roles 同级的 plugins/ 等目录）。</p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">chat_toolbar 顺序</h3>
            <p class="pm-hint">拖拽排序；仅含声明了该插槽的非整壳插件。</p>
            <ol class="pm-order" aria-label="工具栏插件顺序">
              <li
                v-for="(id, i) in toolbarOrder"
                :key="id"
                class="pm-order-item"
                draggable="true"
                @dragstart="onDragToolbarStart(i)"
                @dragover="onDragOver"
                @drop="onDropToolbar(i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                {{ id }}
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
                class="pm-order-item"
                draggable="true"
                @dragstart="onDragSettingsStart(i)"
                @dragover="onDragOver"
                @drop="onDropSettings(i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                {{ id }}
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
                class="pm-order-item"
                draggable="true"
                @dragstart="onDragRoleDetailStart(i)"
                @dragover="onDragOver"
                @drop="onDropRoleDetail(i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                {{ id }}
              </li>
            </ol>
            <p v-if="!roleDetailOrder.length" class="pm-muted">当前无 role.detail 插槽插件。</p>
          </section>

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
  background: color-mix(in srgb, #000 45%, transparent);
}
.pm-dialog {
  position: relative;
  width: min(560px, 100%);
  max-height: min(88vh, 720px);
  overflow: auto;
  padding: 16px 18px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-head {
  padding-right: 32px;
  margin-bottom: 12px;
}
.pm-title {
  margin: 0 0 6px;
  font-size: 18px;
}
.pm-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-close {
  position: absolute;
  top: 10px;
  right: 10px;
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
  margin-bottom: 16px;
}
.pm-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}
.pm-h3 {
  margin: 0;
  font-size: 14px;
}
.pm-plugin-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm-plugin-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding-left: 2px;
}
.pm-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent, #3b82f6) 18%, var(--bg-elevated));
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
  border-radius: 6px;
  cursor: grab;
  background: var(--bg-elevated);
}
.pm-grip {
  color: var(--text-secondary);
  font-size: 12px;
  user-select: none;
}
.pm-muted {
  font-size: 13px;
  color: var(--text-secondary);
}
.pm-err {
  color: var(--text-danger, #c33);
  font-size: 13px;
}
.pm-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
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
  background: var(--accent, #3b82f6);
  color: #fff;
  border-color: transparent;
}
</style>
