<script setup lang="ts">
import type { PluginV2CardItem } from "../../composables/usePluginManagerV2";

defineProps<{
  item: PluginV2CardItem;
  selected?: boolean;
}>();

const emit = defineEmits<{
  select: [];
}>();
</script>

<template>
  <button type="button" class="pm2-card" :class="{ 'is-selected': selected }" @click="emit('select')">
    <div class="pm2-card-head">
      <h4 class="pm2-card-title">{{ item.title }}</h4>
      <span class="pm2-card-status">
        {{ item.status === "enabled" ? "已启用" : item.status === "needs_config" ? "还需配置" : "已关闭" }}
      </span>
    </div>
    <p class="pm2-card-desc">{{ item.description }}</p>
    <div class="pm2-card-meta">
      <span>{{ item.moduleLabel }}</span>
      <span>{{ item.type === "builtin" ? "内置" : item.type === "remote" ? "远程" : "目录插件" }}</span>
      <span>{{ item.sourceLabel }}</span>
    </div>
  </button>
</template>

<style scoped>
.pm2-card {
  width: 100%;
  text-align: left;
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 10px;
  background: var(--bg-primary);
  cursor: pointer;
}
.pm2-card:hover {
  background: var(--bg-elevated);
}
.pm2-card.is-selected {
  border-color: var(--accent, #3b82f6);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent, #3b82f6) 55%, transparent);
}
.pm2-card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
.pm2-card-title {
  margin: 0;
  font-size: 14px;
}
.pm2-card-status {
  font-size: 11px;
  color: var(--text-secondary);
}
.pm2-card-desc {
  margin: 8px 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
