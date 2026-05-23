<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '../../composables/useAppToast'
import { usePluginManagerV2 } from '../../composables/usePluginManagerV2'
import { usePluginStore } from '../../stores/pluginStore'
import LeftCategoryNav from './LeftCategoryNav.vue'
import PluginCardList from './PluginCardList.vue'
import RightDetailPanel from './RightDetailPanel.vue'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  openV1: []
  focusArchSlot: [slotKey: string]
}>()

const {
  searchKeyword,
  selectedCategory,
  selectedCardId,
  categories,
  filteredCards,
  selectedCard,
  applyCardChange,
  hasBlueprint,
} = usePluginManagerV2()
const { t } = useI18n()
const { showToast } = useAppToast()
const pluginStore = usePluginStore()
const busy = ref(false)
const rightCollapsed = ref(false)

onMounted(async () => {
  if (pluginStore.catalog.length > 0)
    return
  try {
    await pluginStore.refresh()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
})

watch(
  () => props.visible,
  (v) => {
    if (v)
      rightCollapsed.value = false
  },
)

async function onApply(payload: Record<string, unknown>) {
  if (!selectedCard.value)
    return
  busy.value = true
  try {
    const msg = await applyCardChange(selectedCard.value, payload)
    showToast('success', msg)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="pm2-root">
    <header class="pm2-head">
      <div>
        <h2 class="pm2-title">
          {{ t("pluginTerms.title.v2") }}
        </h2>
        <p class="pm2-sub">
          {{ t("pluginTerms.subtitle.v2") }}
        </p>
      </div>
      <div class="pm2-actions">
        <button type="button" class="pm2-btn secondary" @click="emit('openV1')">
          {{ t("pluginTerms.action.open_v1") }}
        </button>
        <button type="button" class="pm2-btn" @click="emit('close')">
          {{ t("pluginTerms.action.close") }}
        </button>
      </div>
    </header>
    <p v-if="hasBlueprint" class="pm2-banner" role="note">
      {{ t("pluginManager.v2.archGraphBanner") }}
      <button
        v-if="selectedCard?.slotKey"
        type="button"
        class="pm2-banner-link"
        @click="emit('focusArchSlot', selectedCard.slotKey)"
      >
        {{ t("pluginManager.v2.openArchGraph") }}
      </button>
    </p>
    <p v-else class="pm2-banner pm2-banner--warn" role="note">
      {{ t("pluginManager.v2.noBlueprintHint") }}
    </p>
    <div class="pm2-legend" :aria-label="t('pluginManager.legendAria')">
      <span class="pm2-legend-item is-enabled">{{ t("pluginManager.legend.enabled") }}</span>
      <span class="pm2-legend-item is-pending">{{ t("pluginManager.legend.pending") }}</span>
      <span class="pm2-legend-item is-disabled">{{ t("pluginManager.legend.disabled") }}</span>
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
.pm2-banner {
  margin: 0;
  padding: 10px 12px;
  font-size: 12px;
  line-height: 1.45;
  border-radius: 8px;
  border: 1px dashed var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.pm2-banner--warn {
  border-color: color-mix(in srgb, #f59e0b 35%, var(--border-light));
}
.pm2-banner-link {
  margin-left: 8px;
  padding: 0;
  border: none;
  background: none;
  color: var(--accent, #3b82f6);
  text-decoration: underline;
  cursor: pointer;
  font-size: inherit;
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
