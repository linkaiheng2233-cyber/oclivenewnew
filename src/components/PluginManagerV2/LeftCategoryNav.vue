<script setup lang="ts">
import { ref } from "vue";
import type { PluginV2CategoryItem } from "../../composables/usePluginManagerV2";

defineProps<{
  categories: PluginV2CategoryItem[];
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const openGroups = ref({
  module: true,
  type: true,
  status: true,
});
</script>

<template>
  <aside class="pm2-left">
    <h3 class="pm2-left-title">筛选</h3>
    <ul class="pm2-left-list">
      <li v-for="row in categories.filter((x) => x.id === 'all')" :key="row.id">
        <button
          type="button"
          class="pm2-left-btn"
          :class="{ 'is-active': modelValue === row.id }"
          @click="emit('update:modelValue', row.id)"
        >
          <span>{{ row.label }}</span>
          <span class="pm2-badge">{{ row.count }}</span>
        </button>
      </li>
    </ul>

    <section class="pm2-group">
      <button type="button" class="pm2-group-title" @click="openGroups.module = !openGroups.module">
        <span>按模块</span>
        <span class="pm2-arrow">{{ openGroups.module ? "▾" : "▸" }}</span>
      </button>
      <ul v-if="openGroups.module" class="pm2-left-list">
        <li v-for="row in categories.filter((x) => x.id.startsWith('module:'))" :key="row.id">
          <button
            type="button"
            class="pm2-left-btn"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span>{{ row.label }}</span>
            <span class="pm2-badge">{{ row.count }}</span>
          </button>
        </li>
      </ul>
    </section>

    <section class="pm2-group">
      <button type="button" class="pm2-group-title" @click="openGroups.type = !openGroups.type">
        <span>按实现方式</span>
        <span class="pm2-arrow">{{ openGroups.type ? "▾" : "▸" }}</span>
      </button>
      <ul v-if="openGroups.type" class="pm2-left-list">
        <li v-for="row in categories.filter((x) => x.id.startsWith('type:'))" :key="row.id">
          <button
            type="button"
            class="pm2-left-btn"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span>{{ row.label }}</span>
            <span class="pm2-badge">{{ row.count }}</span>
          </button>
        </li>
      </ul>
    </section>

    <section class="pm2-group">
      <button type="button" class="pm2-group-title" @click="openGroups.status = !openGroups.status">
        <span>按状态</span>
        <span class="pm2-arrow">{{ openGroups.status ? "▾" : "▸" }}</span>
      </button>
      <ul v-if="openGroups.status" class="pm2-left-list">
        <li v-for="row in categories.filter((x) => x.id.startsWith('status:'))" :key="row.id">
          <button
            type="button"
            class="pm2-left-btn"
            :class="{ 'is-active': modelValue === row.id }"
            @click="emit('update:modelValue', row.id)"
          >
            <span>{{ row.label }}</span>
            <span class="pm2-badge">{{ row.count }}</span>
          </button>
        </li>
      </ul>
    </section>
  </aside>
</template>

<style scoped>
.pm2-left {
  border-right: 1px solid var(--border-light);
  padding: 10px 12px 10px 4px;
  min-width: 200px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-elevated) 55%, var(--bg-primary));
  box-shadow:
    inset 0 1px 0 color-mix(in srgb, #fff 8%, transparent),
    inset 0 -1px 0 color-mix(in srgb, #000 12%, transparent);
}
.pm2-left-title {
  margin: 0 0 8px;
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-secondary);
}
.pm2-left-list {
  margin: 0;
  padding: 6px;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--border-light) 90%, #000 10%);
  background: color-mix(in srgb, var(--bg-primary) 88%, transparent);
}
.pm2-group {
  margin-top: 10px;
  padding: 8px;
  border-radius: 10px;
  border: 2px solid color-mix(in srgb, var(--border-light) 75%, #000 25%);
  background: color-mix(in srgb, var(--bg-primary) 92%, var(--bg-elevated));
  box-shadow:
    inset 2px 2px 0 color-mix(in srgb, #fff 6%, transparent),
    inset -2px -2px 0 color-mix(in srgb, #000 10%, transparent);
}
.pm2-group-title {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  padding: 6px 8px;
  border: 1px solid color-mix(in srgb, var(--border-light) 80%, #000 20%);
  border-radius: 6px;
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--bg-elevated) 88%, #fff 12%) 0%,
    color-mix(in srgb, var(--bg-elevated) 55%, #000 8%) 100%
  );
  color: var(--text-primary);
  cursor: pointer;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.pm2-arrow {
  font-size: 11px;
  color: var(--text-secondary);
}
.pm2-left-btn {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 7px 8px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}
.pm2-left-btn:hover {
  border-color: color-mix(in srgb, var(--border-light) 85%, var(--accent) 15%);
  color: var(--text-primary);
}
.pm2-left-btn.is-active {
  border-color: color-mix(in srgb, var(--border-light) 60%, var(--accent) 40%);
  background: color-mix(in srgb, var(--bg-elevated) 65%, var(--accent-soft) 35%);
  color: var(--text-primary);
  box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--accent) 25%, transparent);
}
.pm2-badge {
  font-size: 11px;
  border-radius: 4px;
  border: 1px solid color-mix(in srgb, var(--border-light) 80%, #000 20%);
  padding: 0 6px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  background: color-mix(in srgb, var(--bg-primary) 75%, transparent);
}
</style>
