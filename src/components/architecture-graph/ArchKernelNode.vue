<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { useI18n } from "vue-i18n";

defineProps({
  selected: { type: Boolean, default: false },
});
const { t } = useI18n();
</script>

<template>
  <div class="agn agn-kernel" :class="{ 'agn--selected': selected }">
    <div class="agn-kernel-inner">
      <span class="agn-kernel-ico" aria-hidden="true">⚙️</span>
      <span class="agn-kernel-title">{{ t("pluginWorkbench.graph.kernel") }}</span>
      <span class="agn-kernel-sub">process_message</span>
    </div>
    <Handle id="pipeline" type="source" :position="Position.Bottom" class="agn-handle agn-handle--out" />
  </div>
</template>

<style scoped>
.agn-kernel {
  width: 124px;
  height: 124px;
  clip-path: polygon(50% 0%, 93% 25%, 93% 75%, 50% 100%, 7% 75%, 7% 25%);
  background: linear-gradient(145deg, var(--bg-primary), color-mix(in srgb, #4caf50 10%, var(--bg-elevated)));
  border: 2px solid color-mix(in srgb, #4caf50 50%, var(--border-light));
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}
.agn-kernel-inner {
  text-align: center;
  padding: 8px;
}
.agn-kernel-ico {
  font-size: 28px;
  display: block;
}
.agn-kernel-title {
  display: block;
  font-size: 11px;
  font-weight: 700;
}
.agn-kernel-sub {
  display: block;
  font-size: 9px;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
}
.agn--selected {
  box-shadow: 0 0 0 3px color-mix(in srgb, #2196f3 25%, transparent);
}
</style>
