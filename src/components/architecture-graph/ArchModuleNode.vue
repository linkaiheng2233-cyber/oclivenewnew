<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";
import { BACKEND_COLORS } from "../../lib/graphEditorTheme";
import type { CoreModule } from "../../composables/useArchitectureGraphModel";
import { archGraphActionsKey } from "./archGraphContext";

const props = defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
});
const { t } = useI18n();
const actions = inject(archGraphActionsKey);

const moduleKey = computed(() => props.data?.moduleKey as CoreModule);
const kind = computed(() => props.data?.backendKind as keyof typeof BACKEND_COLORS);
const bar = computed(() => BACKEND_COLORS[kind.value]?.bar ?? BACKEND_COLORS.builtin.bar);

function onSelect(ev: Event) {
  actions?.onBackendChange(moduleKey.value, (ev.target as HTMLSelectElement).value);
}
</script>

<template>
  <div class="agn agn-module" :class="{ 'agn--selected': selected }" :style="{ borderColor: bar }">
    <Handle id="backend-in" type="target" :position="Position.Left" class="agn-handle agn-handle--in" />
    <Handle
      v-if="data?.backendKind === 'directory'"
      id="plugin-out"
      type="source"
      :position="Position.Right"
      class="agn-handle agn-handle--out"
    />
    <div class="agn-module-bar" :style="{ background: bar }" />
    <div class="agn-module-head">
      <span aria-hidden="true">{{ data?.icon }}</span>
      <span class="agn-module-id">{{ data?.moduleKey }}</span>
    </div>
    <div class="agn-module-zh">{{ t(data?.labelKey as string) }}</div>
    <span class="agn-tag" :style="{ color: bar, background: BACKEND_COLORS[kind].tagBg }">{{ data?.backend }}</span>
    <p v-if="data?.primaryPlugin" class="agn-dir">{{ data.primaryPlugin }}</p>
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
    <div class="agn-module-actions nodrag nopan">
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
</template>

<style scoped>
.agn-module {
  width: 220px;
  border-radius: 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.14);
  padding: 0 0 10px;
  position: relative;
}
.agn-module-bar {
  height: 3px;
  border-radius: 10px 10px 0 0;
}
.agn-module-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px 0;
  font-weight: 700;
}
.agn-module-id {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
}
.agn-module-zh {
  padding: 0 12px;
  font-size: 11px;
  color: var(--text-secondary);
}
.agn-tag {
  margin: 6px 12px 0;
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  font-family: ui-monospace, monospace;
}
.agn-dir {
  margin: 6px 12px 0;
  font-size: 11px;
  color: var(--text-secondary);
  word-break: break-all;
}
.agn-widget-lbl {
  display: block;
  margin: 8px 12px 2px;
  font-size: 10px;
  color: var(--text-secondary);
}
.agn-select {
  margin: 0 12px;
  width: calc(100% - 24px);
  font-size: 11px;
  padding: 4px 6px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.agn-module-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 12px 0;
}
.agn-btn {
  font-size: 11px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
</style>
