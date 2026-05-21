<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";

const props = withDefaults(
  defineProps<{
    slotKey: string;
    labelKey: string;
    hintKey?: string;
    /** chat_toolbar 使用独立禁用开关 */
    toolbarSlot?: boolean;
    active?: boolean;
    /** 与真实 UI 区域形态对齐 */
    variant?: "default" | "headerStrip" | "toolbar" | "sidebar" | "overlay";
  }>(),
  { variant: "default" },
);

const emit = defineEmits<{
  select: [];
}>();

const pluginStore = usePluginStore();
const { t } = useI18n();

function isContributionOff(pluginId: string): boolean {
  if (props.toolbarSlot) {
    return pluginStore.isToolbarContributionDisabled(pluginId);
  }
  return pluginStore.isSlotContributionDisabled(props.slotKey, pluginId);
}

const candidates = computed(() =>
  (pluginStore.catalogCandidatesBySlot[props.slotKey] ?? []).filter(
    (id) => !pluginStore.isPluginDisabled(id) && !isContributionOff(id),
  ),
);

const boundIds = computed(() =>
  pluginStore.pluginsOrderedForSlot(props.slotKey).filter((id) => candidates.value.includes(id)),
);

const primaryId = computed(() => boundIds.value[0] ?? "");

const isEmpty = computed(() => boundIds.value.length === 0);

function onSelect(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value;
  if (!v) {
    pluginStore.setSlotPluginIds(props.slotKey, []);
    return;
  }
  const rest = boundIds.value.filter((id) => id !== v);
  pluginStore.setSlotPluginIds(props.slotKey, [v, ...rest]);
}
</script>

<template>
  <div
    class="usmr"
    :class="[
      `usmr--${variant}`,
      {
        'usmr--empty': isEmpty,
        'usmr--active': active,
        'usmr--filled': !isEmpty,
      },
    ]"
    role="group"
    :aria-label="t('pluginWorkbench.layout.selectAria', { slot: t(labelKey) })"
    @click.stop="emit('select')"
  >
    <div class="usmr-badge" :title="slotKey">
      <span class="usmr-badge-key">{{ slotKey }}</span>
      <span class="usmr-badge-zh">{{ t(labelKey) }}</span>
    </div>
    <p v-if="hintKey && variant !== 'headerStrip' && variant !== 'toolbar'" class="usmr-hint">
      {{ t(hintKey) }}
    </p>
    <div v-if="!isEmpty" class="usmr-plugin">
      <span class="usmr-plugin-ico" aria-hidden="true">🧩</span>
      <span class="usmr-plugin-name">{{ primaryId }}</span>
      <span v-if="boundIds.length > 1" class="usmr-more">+{{ boundIds.length - 1 }}</span>
    </div>
    <div v-else-if="variant !== 'toolbar'" class="usmr-empty">{{ t("pluginWorkbench.layout.emptySlot") }}</div>
    <div v-if="variant === 'headerStrip' || variant === 'toolbar'" class="usmr-chrome-placeholder" aria-hidden="true" />
    <select
      class="usmr-select"
      :value="primaryId"
      @click.stop
      @change="onSelect"
    >
      <option value="">{{ t("pluginWorkbench.layout.none") }}</option>
      <option v-for="id in candidates" :key="id" :value="id">{{ id }}</option>
    </select>
  </div>
</template>

<style scoped>
.usmr {
  border-radius: 6px;
  padding: 6px 8px;
  border: 2px solid transparent;
  background: color-mix(in srgb, var(--accent) 6%, var(--bg-primary));
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    background 0.15s ease;
  cursor: pointer;
  position: relative;
}
.usmr--empty {
  border: 2px dashed color-mix(in srgb, var(--text-secondary) 35%, var(--border-light));
  background: color-mix(in srgb, var(--bg-elevated) 70%, transparent);
}
.usmr--active {
  border-color: #2196f3;
  box-shadow: 0 0 0 2px color-mix(in srgb, #2196f3 20%, transparent);
  z-index: 2;
}
.usmr--filled {
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border-light));
}
.usmr--headerStrip {
  width: 100%;
  padding: 6px 10px;
  border-radius: var(--radius-btn);
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 80%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, var(--bg-primary));
}
.usmr--toolbar {
  flex: 1 1 auto;
  min-width: 0;
  max-width: 100%;
  padding: 4px 8px;
}
.usmr--sidebar {
  margin-top: 2px;
}
.usmr--overlay {
  width: 100%;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
}
.usmr-badge {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 8px;
  margin-bottom: 4px;
}
.usmr--headerStrip .usmr-badge,
.usmr--toolbar .usmr-badge {
  margin-bottom: 2px;
}
.usmr-badge-key {
  font-size: 10px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-weight: 700;
  color: var(--text-accent, var(--accent));
}
.usmr-badge-zh {
  font-size: 10px;
  color: var(--text-secondary);
}
.usmr-hint {
  margin: 0 0 4px;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.35;
}
.usmr-plugin {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--accent) 12%, var(--bg-elevated));
  font-size: 11px;
  font-family: ui-monospace, monospace;
}
.usmr--headerStrip .usmr-plugin,
.usmr--toolbar .usmr-plugin {
  margin-bottom: 4px;
}
.usmr-plugin-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.usmr-more {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-secondary);
}
.usmr-empty {
  font-size: 11px;
  color: var(--text-secondary);
  font-style: italic;
  margin-bottom: 6px;
}
.usmr-chrome-placeholder {
  height: 28px;
  margin-bottom: 4px;
  border-radius: var(--radius-btn);
  border: 1px dashed color-mix(in srgb, var(--border-light) 90%, transparent);
  background: var(--bg-elevated);
  opacity: 0.65;
}
.usmr--toolbar .usmr-chrome-placeholder {
  height: 24px;
}
.usmr-select {
  width: 100%;
  font-size: 11px;
  padding: 4px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
</style>
