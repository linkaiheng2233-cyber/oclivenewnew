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
      <span
        class="pm2-card-status"
        :class="{
          'is-enabled': item.status === 'enabled',
          'is-pending': item.status === 'needs_config',
          'is-disabled': item.status === 'disabled',
        }"
      >
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
  box-shadow: 0 1px 0 color-mix(in srgb, var(--border-light) 80%, transparent);
}
.pm2-card:hover {
  background: var(--bg-elevated);
}
.pm2-card.is-selected {
  border-color: var(--accent, #3b82f6);
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--accent, #3b82f6) 55%, transparent),
    0 2px 0 color-mix(in srgb, var(--accent, #3b82f6) 45%, transparent);
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
  padding: 2px 7px;
  border-radius: 999px;
  border: 1px solid transparent;
}
.pm2-card-status.is-enabled {
  color: color-mix(in srgb, #166534 80%, var(--text-primary));
  background: color-mix(in srgb, #16a34a 18%, var(--bg-primary));
  border-color: color-mix(in srgb, #16a34a 40%, transparent);
}
.pm2-card-status.is-pending {
  color: color-mix(in srgb, #92400e 85%, var(--text-primary));
  background: color-mix(in srgb, #f59e0b 20%, var(--bg-primary));
  border-color: color-mix(in srgb, #f59e0b 40%, transparent);
}
.pm2-card-status.is-disabled {
  color: var(--text-secondary);
  background: color-mix(in srgb, #64748b 16%, var(--bg-primary));
  border-color: color-mix(in srgb, #64748b 30%, transparent);
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
