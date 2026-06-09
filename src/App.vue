<script setup lang="ts">
import { defineAsyncComponent, onMounted, provide } from 'vue'
import { MAIN_SHELL_KEY } from './composables/mainShellKey'
import { useMainShell } from './composables/useMainShell'
import { resolveOcliveShell } from './composables/useOcliveShell'

const FluentShell = defineAsyncComponent(() => import('./shells/fluent/FluentShell.vue'))
const ToolShell = defineAsyncComponent(() => import('./shells/tool/ToolShell.vue'))
const TheaterShell = defineAsyncComponent(() => import('./shells/theater/TheaterShell.vue'))

const shellKind = resolveOcliveShell()
const shellState = useMainShell()

provide(MAIN_SHELL_KEY, shellState)

onMounted(() => {
  document.documentElement.setAttribute('data-shell', shellKind)
})
</script>

<template>
  <TheaterShell v-if="shellKind === 'theater'" />
  <FluentShell v-else-if="shellKind === 'fluent'" />
  <ToolShell v-else />
</template>
