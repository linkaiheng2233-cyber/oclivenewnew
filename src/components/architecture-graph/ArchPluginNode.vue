<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'
import { inject } from 'vue'
import { useI18n } from 'vue-i18n'
import { ARCH_NODE_DEFAULT_SIZE } from '../../composables/useArchitectureGraphLayout'
import { backendCssVars } from '../../lib/graphEditorTheme'
import { archGraphActionsKey } from './archGraphContext'
import ArchNodeChrome from './ArchNodeChrome.vue'

defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
})
const { t } = useI18n()
const actions = inject(archGraphActionsKey)
const size = ARCH_NODE_DEFAULT_SIZE.archPlugin!
const themeStyle = backendCssVars('directory')
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
      class="agn-plugin agn-shell-inner"
      :class="{ 'agn--selected': selected, 'agn-plugin--off': data?.disabled }"
      :style="themeStyle"
      @contextmenu.prevent="actions?.onUninstallPlugin(String(data?.pluginId))"
    >
      <Handle
        id="plugin-in"
        type="target"
        :position="Position.Left"
        :connectable-start="false"
        :connectable-end="true"
        connectable="single"
        class="agn-handle agn-handle--in"
      />
      <div class="agn-accent-bar" />
      <div class="agn-head agn-plugin-head">
        {{ data?.pluginId }}
      </div>
      <div class="agn-hint agn-plugin-meta agn-mono">
        <span>{{ data?.moduleKey }}</span>
        <span>v{{ data?.version }}</span>
      </div>
      <span class="agn-hint agn-plugin-state">
        {{
          data?.disabled
            ? t("pluginWorkbench.graph.pluginDisabled")
            : t("pluginWorkbench.graph.pluginEnabled")
        }}
      </span>
      <div class="agn-actions nodrag nopan">
        <button type="button" class="agn-btn" @click="actions?.onFocusPlugin(String(data?.pluginId))">
          {{ t("pluginWorkbench.graph.detail") }}
        </button>
      </div>
    </div>
  </ArchNodeChrome>
</template>

<style scoped>
.agn-shell-inner {
  min-height: 100%;
  box-sizing: border-box;
  padding-bottom: 6px;
}
.agn-plugin--off {
  opacity: 0.58;
}
.agn-plugin-head {
  font-size: 12px;
}
.agn-plugin-meta {
  display: flex;
  justify-content: space-between;
  padding: 0 12px;
}
.agn-plugin-state {
  display: block;
  padding: 0 12px;
}
</style>
