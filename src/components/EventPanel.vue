<script setup>
defineProps({
  items: {
    type: Array,
    default: () => [],
  },
});
</script>

<template>
  <section class="card">
    <h2>Events (latest 10)</h2>
    <pre>{{ items }}</pre>
  </section>
</template>

<style scoped>
.card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}
pre {
  white-space: pre-wrap;
  word-break: break-word;
  background: #f4f4f4;
  padding: 10px;
  border-radius: 8px;
}
</style>
