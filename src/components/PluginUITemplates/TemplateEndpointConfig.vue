<script setup lang="ts">
interface EndpointHintField {
  name: string;
  description: string;
}

interface EndpointSchema {
  summary?: string;
  fields: EndpointHintField[];
}

defineProps<{
  schema: EndpointSchema;
}>();
</script>

<template>
  <div class="tpl-root">
    <p class="tpl-summary">{{ schema.summary }}</p>
    <ul class="tpl-list">
      <li v-for="f in schema.fields" :key="f.name" class="tpl-item">
        <code>{{ f.name }}</code>
        <span>{{ f.description }}</span>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.tpl-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.tpl-summary {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}
.tpl-list {
  margin: 0;
  padding-left: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.tpl-item {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.tpl-item code {
  display: inline-block;
  margin-right: 6px;
}
</style>
