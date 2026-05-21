<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { useI18n } from "vue-i18n";
import { GRAPH_SURFACE } from "../../lib/graphEditorTheme";
import { ARCH_NODE_DEFAULT_SIZE } from "../../composables/useArchitectureGraphLayout";
import ArchNodeChrome from "./ArchNodeChrome.vue";

defineProps({
  selected: { type: Boolean, default: false },
});

const { t } = useI18n();
const size = ARCH_NODE_DEFAULT_SIZE.archKernel!;
</script>

<template>
  <ArchNodeChrome
    variant="kernel"
    :selected="selected"
    :min-width="size.minWidth"
    :min-height="size.minHeight"
    :max-width="size.maxWidth"
    :max-height="size.maxHeight"
  >
    <div
      class="agn-kernel"
      :class="{ 'agn--selected': selected }"
      :style="{ '--arch-accent': GRAPH_SURFACE.kernelAccent }"
    >
      <div class="agn-kernel-inner">
        <span class="agn-kernel-ico" aria-hidden="true">⚙️</span>
        <span class="agn-kernel-title">{{ t("pluginWorkbench.graph.kernel") }}</span>
        <span class="agn-kernel-sub agn-mono">process_message</span>
      </div>
      <Handle
        id="pipeline"
        type="source"
        :position="Position.Bottom"
        class="agn-handle agn-handle--out"
      />
    </div>
  </ArchNodeChrome>
</template>

<style scoped>
.agn-kernel {
  width: 100%;
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  clip-path: polygon(50% 0%, 93% 25%, 93% 75%, 50% 100%, 7% 75%, 7% 25%);
  background: linear-gradient(
    155deg,
    #353538,
    color-mix(in srgb, var(--arch-accent) 18%, #2d2d30)
  );
  color: var(--arch-text, #d4d4d4);
  border: none;
  box-shadow: none;
  position: relative;
}
.agn-kernel-inner {
  text-align: center;
  padding: 10px;
}
.agn-kernel-ico {
  font-size: 26px;
  display: block;
  opacity: 0.9;
}
.agn-kernel-title {
  display: block;
  font-size: 11px;
  font-weight: 600;
}
.agn-kernel-sub {
  display: block;
  font-size: 9px;
  opacity: 0.85;
}
</style>
