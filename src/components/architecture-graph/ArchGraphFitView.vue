<script setup lang="ts">
import { useVueFlow } from '@vue-flow/core'
import { onMounted, watch } from 'vue'
import { usePluginStore } from '../../stores/pluginStore'

const { fitView } = useVueFlow()
const pluginStore = usePluginStore()

function fit() {
  void fitView({ padding: 0.16, duration: 180 })
}

onMounted(() => {
  if (pluginStore.panelVisible)
    fit()
})

watch(
  () => [pluginStore.panelVisible, pluginStore.panelMainTab] as const,
  ([open, tab]) => {
    if (open && tab === 'graph')
      setTimeout(fit, 60)
  },
)
</script>

<template />
