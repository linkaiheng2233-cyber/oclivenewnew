<script setup lang="ts">
import { defineAsyncComponent, onMounted, provide } from 'vue'
import { useEasterEggSkin } from '@oclive/shared/composables/useEasterEggSkin'
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { useMainShell } from './composables/useMainShell'
import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'

const FluentShell = defineAsyncComponent(() => import('./shells/fluent/FluentShell.vue'))
const ToolShell = defineAsyncComponent(() => import('./shells/tool/ToolShell.vue'))

const shellKind = resolveOcliveShell()
const shellState = useMainShell()

useEasterEggSkin()

provide(MAIN_SHELL_KEY, shellState)

onMounted(() => {
  document.documentElement.setAttribute('data-shell', shellKind)
})
</script>

<template>
  <ToolShell v-if="shellKind === 'tool'" />
  <FluentShell v-else />
</template>
