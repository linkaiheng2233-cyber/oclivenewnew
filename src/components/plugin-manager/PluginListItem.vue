<script setup lang="ts">
import type { DirectoryPluginCatalogEntry } from '../api'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    entry: DirectoryPluginCatalogEntry
    /** Manager panel batch-selection mode */
    batchSelectMode?: boolean
    /** Selected in batch mode */
    batchSelected?: boolean
    /** Globally disabled (plugin turned off) */
    pluginDisabled: boolean
    /** Hide only chat_toolbar embed (not applicable to full-shell plugins) */
    toolbarContributionDisabled: boolean
    /** Hide only settings.panel embed */
    settingsPanelContributionDisabled: boolean
    /** Hide only role.detail embed */
    roleDetailContributionDisabled: boolean
    /** Hide only sidebar embed */
    sidebarContributionDisabled: boolean
    /** Hide only chat.header embed */
    chatHeaderContributionDisabled: boolean
  }>(),
  {
    batchSelectMode: false,
    batchSelected: false,
  },
)

const emit = defineEmits<{
  'update:batchSelected': [value: boolean]
  'update:pluginDisabled': [value: boolean]
  'update:toolbarContributionDisabled': [value: boolean]
  'update:settingsPanelContributionDisabled': [value: boolean]
  'update:roleDetailContributionDisabled': [value: boolean]
  'update:sidebarContributionDisabled': [value: boolean]
  'update:chatHeaderContributionDisabled': [value: boolean]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="pli" role="group" :aria-label="t('pluginManager.v1ListItem.aria', { id: entry.id })">
    <label v-if="batchSelectMode" class="pli-batch chk">
      <input
        type="checkbox"
        :checked="batchSelected"
        @change="
          emit('update:batchSelected', ($event.target as HTMLInputElement).checked)
        "
      >
    </label>
    <div class="pli-main">
      <div class="pli-title">
        <span class="pli-id">{{ entry.id }}</span>
        <span class="pli-ver">v{{ entry.version }}</span>
        <span class="pli-kind" :data-shell="entry.isShell">{{
          entry.isShell ? t("pluginManager.v1ListItem.kindShell") : t("pluginManager.v1ListItem.kindSlot")
        }}</span>
      </div>
      <p v-if="entry.provides.length" class="pli-meta">
        provides: {{ entry.provides.join(", ") }}
      </p>
      <p v-if="entry.uiSlotNames.length && !entry.isShell" class="pli-meta">
        {{ t("pluginManager.v1ListItem.uiSlots", { list: entry.uiSlotNames.join(", ") }) }}
      </p>
      <p
        v-if="entry.dependencyStatus && entry.dependencyStatus !== 'ok'"
        class="pli-deps"
      >
        {{
          t("pluginManager.v1ListItem.depsUnmet", {
            status: entry.dependencyStatus,
            issues: (entry.dependencyIssues ?? []).join("；"),
          })
        }}
      </p>
    </div>
    <div class="pli-actions">
      <label class="chk">
        <input
          type="checkbox"
          :checked="pluginDisabled"
          :disabled="pluginDisabled && entry.dependencyStatus !== 'ok'"
          @change="emit('update:pluginDisabled', ($event.target as HTMLInputElement).checked)"
        >
        {{ t("pluginManager.v1ListItem.disablePlugin") }}
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('chat_toolbar')" class="chk">
        <input
          type="checkbox"
          :checked="toolbarContributionDisabled"
          @change="
            emit(
              'update:toolbarContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        >
        {{ t("pluginManager.v1ListItem.hideToolbarEmbed") }}
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('settings.panel')" class="chk">
        <input
          type="checkbox"
          :checked="settingsPanelContributionDisabled"
          @change="
            emit(
              'update:settingsPanelContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        >
        {{ t("pluginManager.v1ListItem.hideSettingsEmbed") }}
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('role.detail')" class="chk">
        <input
          type="checkbox"
          :checked="roleDetailContributionDisabled"
          @change="
            emit(
              'update:roleDetailContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        >
        {{ t("pluginManager.v1ListItem.hideRoleDetailEmbed") }}
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('sidebar')" class="chk">
        <input
          type="checkbox"
          :checked="sidebarContributionDisabled"
          @change="
            emit(
              'update:sidebarContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        >
        {{ t("pluginManager.v1ListItem.hideSidebarEmbed") }}
      </label>
      <label v-if="!entry.isShell && entry.uiSlotNames.includes('chat.header')" class="chk">
        <input
          type="checkbox"
          :checked="chatHeaderContributionDisabled"
          @change="
            emit(
              'update:chatHeaderContributionDisabled',
              ($event.target as HTMLInputElement).checked,
            )
          "
        >
        {{ t("pluginManager.v1ListItem.hideChatHeaderEmbed") }}
      </label>
    </div>
  </div>
</template>

<style scoped>
.pli {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.pli-batch {
  align-self: flex-start;
  padding-top: 2px;
}
.pli-title {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
}
.pli-id {
  font-weight: 600;
  font-size: 14px;
}
.pli-ver {
  font-size: 12px;
  color: var(--text-secondary);
}
.pli-kind {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--text-secondary);
}
.pli-kind[data-shell="true"] {
  background: color-mix(in srgb, var(--fluent-warning-text, #a60) 15%, transparent);
}
.pli-meta {
  margin: 4px 0 0;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.35;
}
.pli-deps {
  margin: 6px 0 0;
  font-size: 11px;
  color: var(--text-danger, #c33);
  line-height: 1.35;
}
.pli-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
  min-width: 140px;
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  cursor: pointer;
  user-select: none;
}
</style>
