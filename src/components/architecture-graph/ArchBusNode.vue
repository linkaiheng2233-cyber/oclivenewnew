<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ARCH_NODE_DEFAULT_SIZE } from '../../composables/useArchitectureGraphLayout'
import { GRAPH_SURFACE } from '../../lib/graphEditorTheme'
import ArchNodeChrome from './ArchNodeChrome.vue'

const props = defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
})
const { t } = useI18n()
const size = ARCH_NODE_DEFAULT_SIZE.archBus!

const moduleKeys = computed(() => (props.data?.moduleKeys as string[]) ?? [])

function outTop(i: number, n: number): string {
  return `${((i + 1) / (n + 1)) * 100}%`
}
</script>

<template>
  <ArchNodeChrome
    :selected="selected"
    :min-width="size.minWidth"
    :min-height="size.minHeight"
    :max-width="size.maxWidth"
    :max-height="size.maxHeight"
  >
    <div
      class="agn-bus agn-shell-inner"
      :class="{ 'agn--selected': selected }"
      :style="{ '--arch-accent': GRAPH_SURFACE.busAccent, '--arch-node-border': GRAPH_SURFACE.busAccent }"
    >
      <Handle
        id="pipeline-in"
        type="target"
        :position="Position.Top"
        :connectable-start="false"
        :connectable-end="true"
        connectable="single"
        class="agn-handle agn-handle--in"
      />
      <div class="agn-accent-bar" />
      <div class="agn-head">
        <span class="agn-head-title">{{ t("pluginWorkbench.graph.facilityBus") }}</span>
      </div>
      <p class="agn-mono agn-bus-type">
        {{ data?.blueprintV2 ? "slot_registry" : "plugin_backends" }}
      </p>
      <p class="agn-hint agn-bus-hint">
        {{ t("pluginWorkbench.graph.facilityBusHint") }}
      </p>
      <Handle
        v-for="(key, i) in moduleKeys"
        :id="`fac-${key}`"
        :key="key"
        type="source"
        :position="Position.Right"
        :connectable-start="true"
        :connectable-end="false"
        connectable="single"
        class="agn-handle agn-handle--out"
        :style="{ top: outTop(i, moduleKeys.length) }"
      />
      <Handle
        id="fac-complex"
        type="source"
        :position="Position.Right"
        :connectable-start="true"
        :connectable-end="false"
        connectable="single"
        class="agn-handle agn-handle--out"
        style="top: 92%"
      />
    </div>
  </ArchNodeChrome>
</template>

<style scoped>
.agn-shell-inner {
  border-style: dashed;
  padding: 8px 12px 10px;
  min-height: 100%;
  box-sizing: border-box;
}
.agn-head-title {
  font-size: 13px;
}
.agn-bus-type {
  margin: 2px 12px 0;
  font-size: 10px;
}
.agn-bus-hint {
  padding: 0 12px 4px;
}
</style>
