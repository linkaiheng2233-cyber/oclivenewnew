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
}
.pm2-actions {
  display: flex;
  align-items: center;
  gap: 8px;
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
  display: grid;
  grid-template-columns: 210px minmax(0, 1fr) 300px;
  gap: 12px;
}
@media (max-width: 1080px) {
  .pm2-grid {
    grid-template-columns: 1fr;
  }
}
</style>
