<script setup lang="ts">
defineProps<{
  text: string;
}>();
</script>

<template>
  <section v-if="text.trim().length > 0" class="rap" aria-label="叙事与内心">
    <h3 class="rap-title">叙事与内心</h3>
    <p class="rap-body">{{ text }}</p>
  </section>
</template>

<style scoped>
.rap {
  margin-top: 10px;
  padding: 10px 11px;
  border-radius: var(--radius-card, 10px);
  border: 1px solid color-mix(in srgb, var(--border-light) 85%, transparent);
  background: color-mix(in srgb, var(--bg-elevated) 92%, transparent);
}
.rap-title {
  margin: 0 0 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--text-secondary);
  text-transform: uppercase;
}
.rap-body {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.55;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
