<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginTraceStore } from '../../stores/pluginTraceStore'

const firstTabRef = ref<HTMLButtonElement | null>(null)
defineExpose({ firstTabRef })

const traceStore = usePluginTraceStore()
const { t } = useI18n()
</script>

<template>
  <div
    class="pm-tabs"
    role="tablist"
    :aria-label="t('pluginWorkbench.aria.tablist')"
  >
    <button
      ref="firstTabRef"
      type="button"
      role="tab"
      class="pm-tab"
      :class="{ 'pm-tab--active': traceStore.panelMainTab === 'graph' }"
      :aria-selected="traceStore.panelMainTab === 'graph'"
      @click="traceStore.panelMainTab = 'graph'"
    >
      {{ t("pluginWorkbench.tabs.graph") }}
    </button>
    <button
      type="button"
      role="tab"
      class="pm-tab"
      :class="{ 'pm-tab--active': traceStore.panelMainTab === 'layout' }"
      :aria-selected="traceStore.panelMainTab === 'layout'"
      @click="traceStore.panelMainTab = 'layout'"
    >
      {{ t("pluginWorkbench.tabs.layout") }}
    </button>
  </div>
</template>

<style scoped>
.pm-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 18px 10px;
  border-bottom: 1px solid var(--border-light);
}
.pm-tab {
  flex: 1 1 auto;
  min-width: 0;
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
}
.pm-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
</style>
