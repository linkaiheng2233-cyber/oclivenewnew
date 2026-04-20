<script setup lang="ts">
import { computed } from "vue";
import type { PluginV2CardItem } from "../../composables/usePluginManagerV2";

const props = defineProps<{
  item: PluginV2CardItem;
  selected?: boolean;
}>();

const emit = defineEmits<{
  select: [];
}>();

const typeLabel = computed(() => {
  const t = props.item.type;
  if (t === "builtin") return "内置";
  if (t === "remote") return "远程";
  return "目录插件";
});

const sourceKind = computed(() => {
  const s = props.item.sourceLabel;
  if (s.includes("会话")) return "session";
  if (s.includes("环境")) return "env";
  return "pack";
});

const riskLabel = computed(() => {
  if (props.item.status === "needs_config") return "缺配置";
  if (sourceKind.value === "env") return "环境优先";
  return "";
});
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
      <span class="pm2-chip pm2-chip--module">{{ item.moduleLabel }}</span>
      <span class="pm2-chip pm2-chip--type">{{ typeLabel }}</span>
      <span class="pm2-chip" :class="`pm2-chip--source-${sourceKind}`">{{ item.sourceLabel }}</span>
      <span v-if="riskLabel" class="pm2-chip pm2-chip--risk">{{ riskLabel }}</span>
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
  gap: 6px;
  font-size: 11px;
  align-items: center;
}
.pm2-chip {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  line-height: 1.25;
  font-weight: 600;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm2-chip--module {
  border-style: solid;
  border-color: color-mix(in srgb, var(--border-light) 70%, var(--accent) 30%);
}
.pm2-chip--type {
  font-variant-numeric: tabular-nums;
}
.pm2-chip--source-pack {
  color: var(--text-secondary);
}
.pm2-chip--source-session {
  color: color-mix(in srgb, #1d4ed8 88%, var(--text-primary));
  background: color-mix(in srgb, #3b82f6 14%, var(--bg-primary));
  border-color: color-mix(in srgb, #3b82f6 35%, transparent);
}
.pm2-chip--source-env {
  color: color-mix(in srgb, #6d28d9 90%, var(--text-primary));
  background: color-mix(in srgb, #a855f7 14%, var(--bg-primary));
  border-color: color-mix(in srgb, #a855f7 35%, transparent);
}
.pm2-chip--risk {
  color: color-mix(in srgb, #92400e 90%, var(--text-primary));
  background: color-mix(in srgb, #f59e0b 18%, var(--bg-primary));
  border-color: color-mix(in srgb, #f59e0b 40%, transparent);
}
</style>
