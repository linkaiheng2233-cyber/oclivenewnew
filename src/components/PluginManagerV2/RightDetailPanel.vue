<script setup lang="ts">
import { computed } from "vue";
import { pluginUiTemplateMap } from "../PluginUITemplates";
import type { PluginV2CardItem } from "../../composables/usePluginManagerV2";

const props = defineProps<{
  item: PluginV2CardItem | null;
  collapsed: boolean;
  busy?: boolean;
}>();

const emit = defineEmits<{
  toggle: [];
  apply: [payload: Record<string, unknown>];
}>();

const templateComponent = computed(() => {
  if (!props.item) return null;
  return pluginUiTemplateMap[props.item.uiTemplate];
});
</script>

<template>
  <aside class="pm2-right" :class="{ collapsed }">
    <button type="button" class="pm2-collapse" @click="emit('toggle')">
      {{ collapsed ? "展开" : "收起" }}
    </button>
    <template v-if="!collapsed">
      <div v-if="item" class="pm2-detail">
        <h3 class="pm2-detail-title">{{ item.title }}</h3>
        <p class="pm2-detail-desc">{{ item.description }}</p>
        <component
          :is="templateComponent"
          v-if="templateComponent"
          :schema="item.schema"
          :busy="busy"
          @submit="emit('apply', $event)"
        />
      </div>
      <p v-else class="pm2-placeholder">先从中间列表选一个卡片。</p>
    </template>
  </aside>
</template>

<style scoped>
.pm2-right {
  border-left: 1px solid var(--border-light);
  padding-left: 10px;
  width: 300px;
  flex-shrink: 0;
}
.pm2-right.collapsed {
  width: 48px;
  padding-left: 6px;
}
.pm2-collapse {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
  font-size: 12px;
  cursor: pointer;
}
.pm2-detail {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.pm2-detail-title {
  margin: 0;
  font-size: 15px;
}
.pm2-detail-desc {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-placeholder {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
