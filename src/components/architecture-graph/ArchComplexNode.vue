<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { useI18n } from "vue-i18n";
import { backendCssVars } from "../../lib/graphEditorTheme";
import { ARCH_NODE_DEFAULT_SIZE } from "../../composables/useArchitectureGraphLayout";
import ArchNodeChrome from "./ArchNodeChrome.vue";

defineProps({
  selected: { type: Boolean, default: false },
});
const { t } = useI18n();
const size = ARCH_NODE_DEFAULT_SIZE.archComplex!;
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
      class="agn-complex agn-shell-inner"
      :class="{ 'agn--selected': selected }"
      :style="backendCssVars('builtin')"
    >
      <Handle id="backend-in" type="target" :position="Position.Left" class="agn-handle agn-handle--in" />
      <div class="agn-accent-bar" />
      <div class="agn-head">🎭 {{ t("pluginWorkbench.graph.complexEmotion") }}</div>
      <p class="agn-hint agn-complex-hint">{{ t("pluginWorkbench.graph.complexHint") }}</p>
    </div>
  </ArchNodeChrome>
</template>

<style scoped>
.agn-shell-inner {
  padding: 8px 12px 10px;
  min-height: 100%;
  box-sizing: border-box;
}
.agn-complex-hint {
  padding-top: 4px;
}
</style>
