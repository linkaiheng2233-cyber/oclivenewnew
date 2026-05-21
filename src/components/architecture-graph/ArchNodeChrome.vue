<script setup lang="ts">
import { NodeResizer } from "@vue-flow/node-resizer";

defineProps({
  selected: { type: Boolean, default: false },
  variant: { type: String, default: "default" },
  minWidth: { type: Number, default: 160 },
  minHeight: { type: Number, default: 72 },
  maxWidth: { type: Number, default: 400 },
  maxHeight: { type: Number, default: 320 },
});
</script>

<template>
  <NodeResizer
    :is-visible="selected"
    :min-width="minWidth"
    :min-height="minHeight"
    :max-width="maxWidth"
    :max-height="maxHeight"
    color="var(--arch-accent, var(--border-light))"
    line-class-name="agn-resize-line"
    handle-class-name="agn-resize-handle"
  />
  <div class="agn-shell" :class="[`agn-shell--${variant}`, { 'agn--selected': selected }]">
    <slot />
  </div>
</template>

<style scoped>
.agn-shell--kernel {
  background: transparent;
  border: none;
  box-shadow: none;
}
</style>
