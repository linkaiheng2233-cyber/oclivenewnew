<script setup lang="ts">
import type { DirectoryPluginCatalogEntry } from "../utils/tauri-api";

defineProps<{
  entry: DirectoryPluginCatalogEntry;
  /** 全局禁用（停用插件） */
  pluginDisabled: boolean;
  /** 仅隐藏 chat_toolbar 嵌入（整壳插件此项不适用） */
  toolbarContributionDisabled: boolean;
  /** 仅隐藏 settings.panel 嵌入 */
  settingsPanelContributionDisabled: boolean;
  /** 仅隐藏 role.detail 嵌入 */
  roleDetailContributionDisabled: boolean;
}>();

const emit = defineEmits<{
  "update:pluginDisabled": [value: boolean];
  "update:toolbarContributionDisabled": [value: boolean];
  "update:settingsPanelContributionDisabled": [value: boolean];
  "update:roleDetailContributionDisabled": [value: boolean];
}>();
</script>

<template>
  <div class="pli" role="group" :aria-label="`插件 ${entry.id}`">
    <div class="pli-main">
      <div class="pli-title">
        <span class="pli-id">{{ entry.id }}</span>
        <span class="pli-ver">v{{ entry.version }}</span>
        <span class="pli-kind" :data-shell="entry.isShell">{{
          entry.isShell ? "整壳" : "插槽"
        }}</span>
      </div>
      <p v-if="entry.provides.length" class="pli-meta">
        provides: {{ entry.provides.join(", ") }}
      </p>
      <p v-if="entry.uiSlotNames.length && !entry.isShell" class="pli-meta">
        UI 插槽: {{ entry.uiSlotNames.join(", ") }}
      </p>
    </div>
    <div class="pli-actions">
      <label class="chk">
        <input
          type="checkbox"
          :checked="pluginDisabled"
          @change="emit('update:pluginDisabled', ($event.target as HTMLInputElement).checked)"
        />
        停用插件
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('chat_toolbar')" class="chk">
        <input
          type="checkbox"
          :checked="toolbarContributionDisabled"
          @change="
            emit(
              'update:toolbarContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        />
        隐藏工具栏嵌入
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('settings.panel')" class="chk">
        <input
          type="checkbox"
          :checked="settingsPanelContributionDisabled"
          @change="
            emit(
              'update:settingsPanelContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        />
        隐藏设置页嵌入
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('role.detail')" class="chk">
        <input
          type="checkbox"
          :checked="roleDetailContributionDisabled"
          @change="
            emit(
              'update:roleDetailContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        />
        隐藏角色详情嵌入
      </label>
    </div>
  </div>
</template>

<style scoped>
.pli {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.pli-title {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
}
.pli-id {
  font-weight: 600;
  font-size: 14px;
}
.pli-ver {
  font-size: 12px;
  color: var(--text-secondary);
}
.pli-kind {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--text-secondary);
}
.pli-kind[data-shell="true"] {
  background: color-mix(in srgb, var(--fluent-warning-text, #a60) 15%, transparent);
}
.pli-meta {
  margin: 4px 0 0;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.35;
}
.pli-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
  min-width: 140px;
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  cursor: pointer;
  user-select: none;
}
</style>
