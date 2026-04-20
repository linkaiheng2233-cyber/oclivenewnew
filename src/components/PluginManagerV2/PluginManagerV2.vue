<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import LeftCategoryNav from "./LeftCategoryNav.vue";
import PluginCardList from "./PluginCardList.vue";
import RightDetailPanel from "./RightDetailPanel.vue";
import { usePluginManagerV2 } from "../../composables/usePluginManagerV2";
import { usePluginTerm } from "../../composables/usePluginTerm";
import { usePluginStore } from "../../stores/pluginStore";
import { useAppToast } from "../../composables/useAppToast";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openV1: [];
}>();

const {
  searchKeyword,
  selectedCategory,
  selectedCardId,
  categories,
  filteredCards,
  selectedCard,
  applyCardChange,
} = usePluginManagerV2();
const { term } = usePluginTerm();
const { showToast } = useAppToast();
const pluginStore = usePluginStore();
const busy = ref(false);
const rightCollapsed = ref(false);

onMounted(async () => {
  if (pluginStore.catalog.length > 0) return;
  try {
    await pluginStore.refresh();
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
});

watch(
  () => props.visible,
  (v) => {
    if (v) rightCollapsed.value = false;
  },
);

async function onApply(payload: Record<string, unknown>) {
  if (!selectedCard.value) return;
  busy.value = true;
  try {
    const msg = await applyCardChange(selectedCard.value, payload);
    showToast("success", msg);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="pm2-root">
    <header class="pm2-head">
      <div>
        <h2 class="pm2-title">{{ term("title.v2") }}</h2>
        <p class="pm2-sub">{{ term("subtitle.v2") }}</p>
      </div>
      <div class="pm2-actions">
        <button type="button" class="pm2-btn secondary" @click="emit('openV1')">
          {{ term("action.open_v1") }}
        </button>
        <button type="button" class="pm2-btn" @click="emit('close')">{{ term("action.close") }}</button>
      </div>
    </header>
    <div class="pm2-legend" aria-label="状态说明">
      <span class="pm2-legend-item is-enabled">已启用：当前配置可直接生效</span>
      <span class="pm2-legend-item is-pending">还需配置：通常缺少目录插件 ID</span>
      <span class="pm2-legend-item is-disabled">已关闭：当前链路未启用</span>
    </div>

    <div class="pm2-grid">
      <LeftCategoryNav v-model="selectedCategory" :categories="categories" />
      <PluginCardList
        :items="filteredCards"
        :selected-id="selectedCardId"
        :keyword="searchKeyword"
        @update:keyword="searchKeyword = $event"
        @select="selectedCardId = $event"
      />
      <RightDetailPanel
        :item="selectedCard"
        :collapsed="rightCollapsed"
        :busy="busy"
        @toggle="rightCollapsed = !rightCollapsed"
        @apply="onApply"
      />
    </div>
  </div>
</template>

<style scoped>
.pm2-root {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.pm2-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}
.pm2-title {
  margin: 0 0 6px;
  font-size: 18px;
}
.pm2-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.pm2-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px;
  border: 1px dashed var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
}
.pm2-legend-item {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.2;
}
.pm2-legend-item.is-enabled {
  background: color-mix(in srgb, #16a34a 16%, var(--bg-primary));
  color: color-mix(in srgb, #166534 80%, var(--text-primary));
}
.pm2-legend-item.is-pending {
  background: color-mix(in srgb, #f59e0b 20%, var(--bg-primary));
  color: color-mix(in srgb, #92400e 85%, var(--text-primary));
}
.pm2-legend-item.is-disabled {
  background: color-mix(in srgb, #64748b 18%, var(--bg-primary));
  color: var(--text-secondary);
}
.pm2-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.pm2-btn.secondary {
  background: transparent;
}
.pm2-grid {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 248px minmax(0, 1fr) 300px;
  grid-template-rows: minmax(0, 1fr);
  gap: 12px;
  align-items: stretch;
}
.pm2-grid > * {
  min-height: 0;
}
@media (max-width: 1080px) {
  .pm2-grid {
    grid-template-columns: 1fr;
    grid-template-rows: none;
    grid-auto-rows: auto;
    flex: 1 1 auto;
    min-height: 0;
    overflow: auto;
  }
}
</style>
