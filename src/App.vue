<script setup lang="ts">
import { onMounted, provide } from 'vue'
import { MAIN_SHELL_KEY } from './composables/mainShellKey'
import { useMainShell } from './composables/useMainShell'
import { resolveOcliveShell } from './composables/useOcliveShell'
import FluentShell from './shells/fluent/FluentShell.vue'
import ToolShell from './shells/tool/ToolShell.vue'

const shellKind = resolveOcliveShell()
const shellState = useMainShell()

provide(MAIN_SHELL_KEY, shellState)

onMounted(() => {
  document.documentElement.setAttribute('data-shell', shellKind)
})
</script>

<template>
  <FluentShell v-if="shellKind === 'fluent'" />
  <ToolShell v-else />
</template>
