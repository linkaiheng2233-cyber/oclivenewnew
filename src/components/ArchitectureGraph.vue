<script setup lang="ts">
import ArchitectureGraphFlow from "./ArchitectureGraphFlow.vue";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();
</script>

<template>
  <ArchitectureGraphFlow @focus-plugin="emit('focus-plugin', $event)" />
</template>
