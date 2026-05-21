<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
});
const { t } = useI18n();

const moduleKeys = computed(() => (props.data?.moduleKeys as string[]) ?? []);

function outTop(i: number, n: number): string {
  return `${((i + 1) / (n + 1)) * 100}%`;
}
</script>

<template>
  <div class="agn agn-bus" :class="{ 'agn--selected': selected }">
    <Handle id="pipeline-in" type="target" :position="Position.Top" class="agn-handle agn-handle--in" />
    <div class="agn-bus-head">{{ t("pluginWorkbench.graph.facilityBus") }}</div>
    <div class="agn-bus-type">plugin_backends</div>
    <p class="agn-bus-hint">{{ t("pluginWorkbench.graph.facilityBusHint") }}</p>
    <Handle
      v-for="(key, i) in moduleKeys"
      :id="`fac-${key}`"
      :key="key"
      type="source"
      :position="Position.Right"
      class="agn-handle agn-handle--out"
      :style="{ top: outTop(i, moduleKeys.length) }"
    />
    <Handle
      id="fac-complex"
      type="source"
      :position="Position.Right"
      class="agn-handle agn-handle--out"
      style="top: 92%"
    />
  </div>
</template>

<style scoped>
.agn-bus {
  width: 240px;
  min-height: 100px;
  border-radius: 10px;
  border: 2px dashed color-mix(in srgb, #2196f3 45%, var(--border-light));
  background: color-mix(in srgb, #2196f3 8%, var(--bg-primary));
  padding: 10px 14px 10px 10px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.12);
  position: relative;
}
.agn-bus-head {
  font-size: 13px;
  font-weight: 700;
}
.agn-bus-type {
  font-size: 10px;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
}
.agn-bus-hint {
  margin: 6px 0 0;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.35;
}
</style>
