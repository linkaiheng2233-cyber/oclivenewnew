<script setup lang="ts">
import { useVueFlow } from '@vue-flow/core'
import { onMounted, watch } from 'vue'
import { usePluginTraceStore } from '../../stores/pluginTraceStore'

const { fitView } = useVueFlow()
const traceStore = usePluginTraceStore()

function fit() {
  void fitView({ padding: 0.16, duration: 180 })
}

onMounted(() => {
  if (traceStore.panelVisible)
    fit()
})

watch(
  () => [traceStore.panelVisible, traceStore.panelMainTab] as const,
  ([open, tab]) => {
    if (open && tab === 'graph')
      setTimeout(fit, 60)
  },
)
</script>

<template />
