<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { inject } from "vue";
import { useI18n } from "vue-i18n";
import { archGraphActionsKey } from "./archGraphContext";

defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
});
const { t } = useI18n();
const actions = inject(archGraphActionsKey);
</script>

<template>
  <div
    class="agn agn-plugin"
    :class="{ 'agn--selected': selected, 'agn-plugin--off': data?.disabled }"
    @contextmenu.prevent="actions?.onUninstallPlugin(String(data?.pluginId))"
  >
    <Handle id="plugin-in" type="target" :position="Position.Left" class="agn-handle agn-handle--in agn-handle--plugin" />
    <div class="agn-plugin-bar" />
    <div class="agn-plugin-head">{{ data?.pluginId }}</div>
    <div class="agn-plugin-meta">
      <span>{{ data?.moduleKey }}</span>
      <span>v{{ data?.version }}</span>
    </div>
    <span class="agn-plugin-state">
      {{
        data?.disabled
          ? t("pluginWorkbench.graph.pluginDisabled")
          : t("pluginWorkbench.graph.pluginEnabled")
      }}
    </span>
    <button
      type="button"
      class="agn-btn nodrag nopan"
      @click="actions?.onFocusPlugin(String(data?.pluginId))"
    >
      {{ t("pluginWorkbench.graph.detail") }}
    </button>
  </div>
</template>

<style scoped>
.agn-plugin {
  width: 180px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid color-mix(in srgb, #9c27b0 40%, var(--border-light));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  padding: 0 10px 10px;
  position: relative;
}
.agn-plugin--off {
  opacity: 0.55;
  border-style: dashed;
}
.agn-plugin-bar {
  height: 3px;
  background: #9c27b0;
  margin: 0 -10px;
  border-radius: 8px 8px 0 0;
}
.agn-plugin-head {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  padding-top: 8px;
}
.agn-plugin-meta {
  font-size: 10px;
  color: var(--text-secondary);
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
}
.agn-plugin-state {
  display: block;
  font-size: 10px;
  color: var(--text-secondary);
  margin: 4px 0;
}
</style>
