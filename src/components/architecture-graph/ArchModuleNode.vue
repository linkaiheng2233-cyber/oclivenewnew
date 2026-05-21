<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";
import { BACKEND_COLORS, backendCssVars } from "../../lib/graphEditorTheme";
import type { CoreModule } from "../../composables/useArchitectureGraphModel";
import { ARCH_NODE_DEFAULT_SIZE } from "../../composables/useArchitectureGraphLayout";
import { archGraphActionsKey } from "./archGraphContext";
import ArchNodeChrome from "./ArchNodeChrome.vue";

const props = defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
});
const { t } = useI18n();
const actions = inject(archGraphActionsKey);
const size = ARCH_NODE_DEFAULT_SIZE.archModule!;

const moduleKey = computed(() => props.data?.moduleKey as CoreModule);
const kind = computed(() => props.data?.backendKind as keyof typeof BACKEND_COLORS);
const themeStyle = computed(() => backendCssVars(kind.value));

function onSelect(ev: Event) {
  actions?.onBackendChange(moduleKey.value, (ev.target as HTMLSelectElement).value);
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
      class="agn-module agn-shell-inner"
      :class="{ 'agn--selected': selected }"
      :style="themeStyle"
    >
      <Handle
        id="backend-in"
        type="target"
        :position="Position.Left"
        :connectable-start="false"
        :connectable-end="true"
        connectable="single"
        class="agn-handle agn-handle--in"
      />
      <Handle
        v-if="data?.backendKind === 'directory'"
        id="plugin-out"
        type="source"
        :position="Position.Right"
        :connectable-start="true"
        :connectable-end="false"
        class="agn-handle agn-handle--out"
      />
      <div class="agn-accent-bar" />
      <div class="agn-head">
        <span aria-hidden="true">{{ data?.icon }}</span>
        <span class="agn-mono agn-module-id">{{ data?.moduleKey }}</span>
      </div>
      <p class="agn-hint agn-module-zh">{{ t(data?.labelKey as string) }}</p>
      <span class="agn-tag">{{ data?.backend }}</span>
      <p v-if="data?.primaryPlugin" class="agn-hint agn-dir agn-mono">{{ data.primaryPlugin }}</p>
      <label class="agn-widget-lbl">{{ t("pluginWorkbench.graph.switchBackend") }}</label>
      <select
        class="agn-select nodrag nopan"
        :disabled="actions?.busy()"
        :value="data?.sessionOverride"
        @change="onSelect"
        @pointerdown.stop
      >
        <option value="__pack_default__">
          {{ t("pluginWorkbench.graph.followPack", { value: data?.packDefault }) }}
        </option>
        <option v-for="v in (data?.options as string[])" :key="v" :value="v">{{ v }}</option>
      </select>
      <div class="agn-actions nodrag nopan">
        <button
          v-if="data?.primaryPlugin"
          type="button"
          class="agn-btn"
          @click="actions?.onFocusPlugin(String(data.primaryPlugin))"
        >
          {{ t("pluginWorkbench.graph.detail") }}
        </button>
        <button
          v-if="(data?.hiddenPluginCount as number) > 0"
          type="button"
          class="agn-btn"
          @click="actions?.onToggleExpand(moduleKey)"
        >
          +{{ data.hiddenPluginCount }} {{ t("pluginWorkbench.graph.plugins") }}
        </button>
      </div>
    </div>
  </ArchNodeChrome>
</template>

<style scoped>
.agn-shell-inner {
  padding: 0 0 4px;
  min-height: 100%;
  box-sizing: border-box;
}
.agn-module-id {
  font-size: 12px;
  font-weight: 600;
}
.agn-module-zh,
.agn-dir {
  padding: 0 12px;
}
</style>
