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

const changeNotice = computed(() => {
  if (!props.item) return "";
  if (props.item.uiTemplate === "endpoint-config") {
    return "只读说明：此处不会写入任何配置；请在环境变量或角色包中修改后重载应用。";
  }
  return "变更预览：点击下方「应用改动」后写入当前会话（不修改角色包 settings.json；若与环境变量冲突，以环境解析为准）。";
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
        <p class="pm2-change-notice" role="note">{{ changeNotice }}</p>
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
  box-sizing: border-box;
  border-left: 1px solid var(--border-light);
  padding-left: 10px;
  width: 100%;
  min-width: 0;
  min-height: 0;
  height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
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
  font-weight: 600;
}
.pm2-detail {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px;
  border: 1px dashed var(--border-light);
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
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
.pm2-change-notice {
  margin: 0;
  padding: 8px 10px;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--border-light) 85%, var(--accent) 15%);
  background: color-mix(in srgb, var(--bg-primary) 82%, var(--accent-soft) 18%);
}
.pm2-placeholder {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
