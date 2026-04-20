<script setup lang="ts">
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
</script>

<template>
  <section class="pm2-mid">
    <input
      class="pm2-search"
      type="search"
      :value="keyword"
      placeholder="搜索：例如 远程、情绪、目录插件"
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
      <p v-if="!items.length" class="pm2-empty">没有匹配项，试试更短的关键词。</p>
    </div>
  </section>
</template>

<style scoped>
.pm2-mid {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}
.pm2-search {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.pm2-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 62vh;
  overflow: auto;
  padding-right: 4px;
}
.pm2-empty {
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
