<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  roleRailOpen: boolean
  settingsActive: boolean
  pluginsActive: boolean
  modelsActive: boolean
  moreOpen: boolean
}>()

const emit = defineEmits<{
  focusChat: []
  toggleRoleRail: []
  openSettings: []
  openPlugins: []
  openModels: []
  toggleMore: []
}>()

const { t } = useI18n()
</script>

<template>
  <nav class="tool-activity-bar" :aria-label="t('toolShell.activityMore')">
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': !settingsActive && !pluginsActive && !modelsActive }"
      :title="t('toolShell.activityChat')"
      :aria-label="t('toolShell.activityChat')"
      @click="emit('focusChat')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M4 4h16a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H8l-4 3V6a2 2 0 0 1 2-2Z" />
      </svg>
    </button>
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': roleRailOpen }"
      :title="t('toolShell.activityRole')"
      :aria-label="t('toolShell.activityRole')"
      @click="emit('toggleRoleRail')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M12 12a4 4 0 1 0-4-4 4 4 0 0 0 4 4Zm0 2c-4.42 0-8 2.24-8 5v1h16v-1c0-2.76-3.58-5-8-5Z" />
      </svg>
    </button>
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': settingsActive }"
      :title="t('toolShell.activitySettings')"
      :aria-label="t('toolShell.activitySettings')"
      @click="emit('openSettings')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M12 8a4 4 0 1 1 4 4 4 4 0 0 1-4-4Zm8.94 3.05a1 1 0 0 0-.26-1.09l-1.42-1.42a1 1 0 0 0-1.09-.26 7.07 7.07 0 0 0-1.62-.94 1 1 0 0 0-1.15.49l-.66 1.14a1 1 0 0 1-1.15.49 7.07 7.07 0 0 0-1.62.94 1 1 0 0 0-1.09.26l-1.42 1.42a1 1 0 0 0-.26 1.09 7.07 7.07 0 0 0 .94 1.62 1 1 0 0 0 .49 1.15l1.14.66a1 1 0 0 1 .49 1.15 7.07 7.07 0 0 0-.94 1.62 1 1 0 0 0 .26 1.09l1.42 1.42a1 1 0 0 0 1.09.26 7.07 7.07 0 0 0 1.62-.94 1 1 0 0 0 1.15-.49l.66-1.14a1 1 0 0 1 1.15-.49 7.07 7.07 0 0 0 1.62-.94 1 1 0 0 0 1.09-.26l1.42-1.42a1 1 0 0 0 .26-1.09 7.07 7.07 0 0 0-.94-1.62 1 1 0 0 0-.49-1.15l-1.14-.66a1 1 0 0 1-.49-1.15 7.07 7.07 0 0 0 .94-1.62Z" />
      </svg>
    </button>
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': pluginsActive }"
      :title="t('toolShell.activityPlugins')"
      :aria-label="t('toolShell.activityPlugins')"
      @click="emit('openPlugins')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M8 4h8l2 2v12l-2 2H8l-2-2V6l2-2Zm1 4v2h6V8H9Zm0 4v2h4v-2H9Z" />
      </svg>
    </button>
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': modelsActive }"
      :title="t('toolShell.activityModels')"
      :aria-label="t('toolShell.activityModels')"
      @click="emit('openModels')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M12 3a5 5 0 0 1 4.9 4H19a2 2 0 0 1 2 2v3h-2v11H5V12H3V9a2 2 0 0 1 2-2h2.1A5 5 0 0 1 12 3Zm0 2a3 3 0 1 0 3 3 3 3 0 0 0-3-3Zm-5 7v9h10v-9H7Z" />
      </svg>
    </button>
    <div class="tool-activity-bar__spacer" />
    <button
      type="button"
      class="tool-activity-bar__btn"
      :class="{ 'tool-activity-bar__btn--active': moreOpen }"
      :title="t('toolShell.activityMore')"
      :aria-label="t('toolShell.activityMore')"
      @click="emit('toggleMore')"
    >
      <svg class="tool-activity-bar__icon" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M6 10a2 2 0 1 1 0 4 2 2 0 0 1 0-4Zm6 0a2 2 0 1 1 0 4 2 2 0 0 1 0-4Zm6 0a2 2 0 1 1 0 4 2 2 0 0 1 0-4Z" />
      </svg>
    </button>
  </nav>
</template>

<style scoped>
.tool-activity-bar {
  flex: 0 0 var(--tool-activitybar-w, 48px);
  width: var(--tool-activitybar-w, 48px);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--tool-space-1, 4px);
  padding: var(--tool-space-2, 8px) 0;
  background: var(--tool-chrome-activity, var(--tool-bg, var(--bg-secondary)));
  border-right: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}

.tool-activity-bar__spacer {
  flex: 1;
  min-height: var(--tool-space-2, 8px);
}

.tool-activity-bar__btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--tool-radius, 4px);
  background: transparent;
  cursor: pointer;
  color: var(--tool-text-muted, var(--text-secondary));
  transition: var(--tool-transition, var(--control-transition));
}

.tool-activity-bar__btn:hover {
  background: color-mix(in srgb, var(--tool-border, var(--border-light)) 40%, transparent);
  color: var(--tool-text, var(--text-primary));
}

.tool-activity-bar__btn--active {
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 14%, transparent);
  color: var(--tool-accent, var(--accent));
}

.tool-activity-bar__icon {
  display: block;
  flex-shrink: 0;
}
</style>
