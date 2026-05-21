<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { useI18n } from "vue-i18n";

defineProps({
  selected: { type: Boolean, default: false },
});
const { t } = useI18n();
</script>

<template>
  <div class="agn agn-complex" :class="{ 'agn--selected': selected }">
    <Handle id="backend-in" type="target" :position="Position.Left" class="agn-handle agn-handle--in" />
    <div class="agn-complex-head">🎭 {{ t("pluginWorkbench.graph.complexEmotion") }}</div>
    <p class="agn-complex-hint">{{ t("pluginWorkbench.graph.complexHint") }}</p>
  </div>
</template>

<style scoped>
.agn-complex {
  width: 200px;
  border-radius: 10px;
  background: var(--bg-primary);
  border: 1px solid color-mix(in srgb, #4caf50 40%, var(--border-light));
  padding: 10px 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  position: relative;
}
.agn-complex-head {
  font-size: 12px;
  font-weight: 700;
}
.agn-complex-hint {
  margin: 6px 0 0;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.4;
}
</style>
