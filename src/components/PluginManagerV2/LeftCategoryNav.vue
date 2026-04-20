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
  padding-right: 10px;
  min-width: 200px;
}
.pm2-left-title {
  margin: 0 0 8px;
  font-size: 14px;
}
.pm2-left-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm2-group {
  margin-top: 10px;
}
.pm2-group-title {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  padding: 6px 8px;
  border: 1px dashed var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
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
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}
.pm2-left-btn:hover {
  border-color: var(--border-light);
  color: var(--text-primary);
}
.pm2-left-btn.is-active {
  border-color: var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 70%, var(--accent-soft) 30%);
  color: var(--text-primary);
}
.pm2-badge {
  font-size: 11px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  padding: 0 7px;
}
</style>
