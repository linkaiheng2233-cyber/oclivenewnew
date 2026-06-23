<script setup lang="ts">
import { defineAsyncComponent, onMounted } from 'vue'

const TheaterShell = defineAsyncComponent(() => import('./shells/theater/TheaterShell.vue'))

onMounted(() => {
  document.documentElement.setAttribute('data-shell', 'theater')
})
</script>

<template>
  <TheaterShell />
</template>
