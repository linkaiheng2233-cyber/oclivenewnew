<script setup lang="ts">
import { useI18n } from "vue-i18n";
import PluginCard from "./PluginCard.vue";
import type { PluginV2CardItem } from "../../composables/usePluginManagerV2";

defineProps<{
  items: PluginV2CardItem[];
  selectedId: string;
  keyword: string;
}>();

const emit = defineEmits<{
  "update:keyword": [value: string];
  select: [id: string];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="pm2-mid">
    <input
      class="pm2-search"
      type="search"
      :value="keyword"
      :placeholder="String(t('pluginManagerV2.list.searchPlaceholder'))"
      @input="emit('update:keyword', ($event.target as HTMLInputElement).value)"
    />
    <div class="pm2-list">
      <PluginCard
        v-for="item in items"
        :key="item.id"
        :item="item"
        :selected="item.id === selectedId"
        @select="emit('select', item.id)"
      />
      <p v-if="!items.length" class="pm2-empty">{{ t("pluginManagerV2.list.empty") }}</p>
    </div>
  </section>
</template>

<style scoped>
.pm2-mid {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}
.pm2-search {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.pm2-list {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: auto;
  padding-right: 4px;
}
.pm2-empty {
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
