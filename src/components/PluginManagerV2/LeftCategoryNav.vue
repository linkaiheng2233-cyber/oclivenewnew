<script setup lang="ts">
import { computed, ref } from "vue";
import type { PluginV2CategoryItem } from "../../composables/usePluginManagerV2";

const props = defineProps<{
  categories: PluginV2CategoryItem[];
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const openFolders = ref({
  module: true,
  type: true,
  status: true,
});

const allRow = computed(() => props.categories.find((c) => c.id === "all") ?? null);

const moduleRows = computed(() =>
  props.categories.filter((c) => c.id.startsWith("module:")),
);
const typeRows = computed(() => props.categories.filter((c) => c.id.startsWith("type:")));
const statusRows = computed(() =>
  props.categories.filter((c) => c.id.startsWith("status:")),
);

function filterFileName(row: PluginV2CategoryItem): string {
  const i = row.id.indexOf(":");
  if (i < 0) return `${row.id}.filter`;
  return `${row.id.slice(i + 1)}.filter`;
}

function toggleFolder(key: keyof typeof openFolders.value) {
  openFolders.value[key] = !openFolders.value[key];
}
</script>

<template>
  <aside class="ws-explorer" aria-label="筛选（工作区风格）">
    <header class="ws-head">
      <div class="ws-head-title">资源管理器</div>
      <div class="ws-head-sub">筛选视图</div>
    </header>

    <div class="ws-root" title="仅用于 UI 层级展示，不代表磁盘路径">
      <span class="ws-root-label">oclivenewnew</span>
      <span class="ws-root-sep">/</span>
      <span class="ws-root-label">plugin-manager</span>
      <span class="ws-root-sep">/</span>
      <span class="ws-root-label ws-root-label--accent">filters</span>
    </div>

    <nav class="ws-tree" aria-label="筛选树">
      <button
        v-if="allRow"
        type="button"
        class="ws-row ws-row--file"
        :class="{ 'is-active': modelValue === allRow.id }"
        @click="emit('update:modelValue', allRow.id)"
      >
        <span class="ws-gutter" aria-hidden="true" />
        <span class="ws-file-icon" aria-hidden="true">#</span>
        <span class="ws-main">
          <span class="ws-name">all.filter</span>
          <span class="ws-hint">{{ allRow.label }}</span>
        </span>
        <span class="ws-count">{{ allRow.count }}</span>
      </button>

      <div class="ws-folder">
        <button type="button" class="ws-row ws-row--folder" @click="toggleFolder('module')">
          <span class="ws-chevron" aria-hidden="true">{{
            openFolders.module ? "▾" : "▸"
          }}</span>
          <span class="ws-folder-icon" aria-hidden="true">[+]</span>
          <span class="ws-name">by-module</span>
        </button>
        <div v-show="openFolders.module" class="ws-children">
          <button
            v-for="row in moduleRows"
            :key="row.id"
            type="button"
            class="ws-row ws-row--file ws-row--child"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span class="ws-gutter ws-gutter--guide" aria-hidden="true" />
            <span class="ws-file-icon" aria-hidden="true">#</span>
            <span class="ws-main">
              <span class="ws-name">{{ filterFileName(row) }}</span>
              <span class="ws-hint">{{ row.label }}</span>
            </span>
            <span class="ws-count">{{ row.count }}</span>
          </button>
        </div>
      </div>

      <div class="ws-folder">
        <button type="button" class="ws-row ws-row--folder" @click="toggleFolder('type')">
          <span class="ws-chevron" aria-hidden="true">{{ openFolders.type ? "▾" : "▸" }}</span>
          <span class="ws-folder-icon" aria-hidden="true">[+]</span>
          <span class="ws-name">by-backend</span>
        </button>
        <div v-show="openFolders.type" class="ws-children">
          <button
            v-for="row in typeRows"
            :key="row.id"
            type="button"
            class="ws-row ws-row--file ws-row--child"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span class="ws-gutter ws-gutter--guide" aria-hidden="true" />
            <span class="ws-file-icon" aria-hidden="true">#</span>
            <span class="ws-main">
              <span class="ws-name">{{ filterFileName(row) }}</span>
              <span class="ws-hint">{{ row.label }}</span>
            </span>
            <span class="ws-count">{{ row.count }}</span>
          </button>
        </div>
      </div>

      <div class="ws-folder">
        <button type="button" class="ws-row ws-row--folder" @click="toggleFolder('status')">
          <span class="ws-chevron" aria-hidden="true">{{
            openFolders.status ? "▾" : "▸"
          }}</span>
          <span class="ws-folder-icon" aria-hidden="true">[+]</span>
          <span class="ws-name">by-status</span>
        </button>
        <div v-show="openFolders.status" class="ws-children">
          <button
            v-for="row in statusRows"
            :key="row.id"
            type="button"
            class="ws-row ws-row--file ws-row--child"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span class="ws-gutter ws-gutter--guide" aria-hidden="true" />
            <span class="ws-file-icon" aria-hidden="true">#</span>
            <span class="ws-main">
              <span class="ws-name">{{ filterFileName(row) }}</span>
              <span class="ws-hint">{{ row.label }}</span>
            </span>
            <span class="ws-count">{{ row.count }}</span>
          </button>
        </div>
      </div>
    </nav>
  </aside>
</template>

<style scoped>
.ws-explorer {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  height: 100%;
  border-right: 1px solid var(--border-light);
  padding: 8px 10px 10px 6px;
  overflow-x: hidden;
  overflow-y: auto;
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-elevated) 45%, var(--bg-primary));
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
  font-size: 12px;
  line-height: 1.35;
  color: var(--text-primary);
  box-shadow:
    inset 0 1px 0 color-mix(in srgb, #fff 7%, transparent),
    inset 0 -1px 0 color-mix(in srgb, #000 10%, transparent);
}

.ws-head {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
  padding: 6px 8px;
  border: 1px solid color-mix(in srgb, var(--border-light) 85%, #000 15%);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-primary) 88%, transparent);
}
.ws-head-title {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-secondary);
}
.ws-head-sub {
  font-size: 11px;
  color: var(--text-secondary);
}

.ws-root {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: baseline;
  margin: 0 0 8px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px dashed color-mix(in srgb, var(--border-light) 80%, var(--accent) 20%);
  color: var(--text-secondary);
  word-break: break-all;
}
.ws-root-label {
  color: var(--text-secondary);
}
.ws-root-label--accent {
  color: var(--text-primary);
  font-weight: 800;
}
.ws-root-sep {
  opacity: 0.55;
}

.ws-tree {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ws-folder {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ws-children {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: 2px;
}

.ws-row {
  width: 100%;
  display: grid;
  grid-template-columns: 14px 14px minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
  padding: 5px 6px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.ws-row:hover {
  border-color: color-mix(in srgb, var(--border-light) 80%, var(--accent) 20%);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
}
.ws-row.is-active {
  border-color: color-mix(in srgb, var(--border-light) 55%, var(--accent) 45%);
  background: color-mix(in srgb, var(--bg-elevated) 60%, var(--accent-soft) 40%);
}

.ws-row--folder {
  grid-template-columns: 14px 18px minmax(0, 1fr);
  color: var(--text-secondary);
  font-weight: 700;
}
.ws-row--file {
  font-weight: 500;
}
.ws-row--child .ws-main {
  padding-left: 2px;
}

.ws-gutter {
  width: 14px;
  height: 100%;
}
.ws-gutter--guide {
  border-left: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  margin-left: 3px;
}

.ws-chevron {
  width: 14px;
  text-align: center;
  color: var(--text-secondary);
  user-select: none;
}
.ws-folder-icon {
  width: 18px;
  text-align: center;
  color: color-mix(in srgb, var(--text-secondary) 70%, var(--accent) 30%);
  user-select: none;
}
.ws-file-icon {
  width: 14px;
  text-align: center;
  color: color-mix(in srgb, var(--text-secondary) 65%, var(--accent) 35%);
  user-select: none;
}

.ws-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.ws-name {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ws-hint {
  font-size: 10px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.92;
}

.ws-count {
  font-size: 10px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid color-mix(in srgb, var(--border-light) 80%, #000 20%);
  background: color-mix(in srgb, var(--bg-primary) 78%, transparent);
  color: var(--text-secondary);
}
.ws-row.is-active .ws-count {
  border-color: color-mix(in srgb, var(--border-light) 55%, var(--accent) 45%);
  color: var(--text-primary);
}
</style>
