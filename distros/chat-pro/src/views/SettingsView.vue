<script setup lang="ts">
import { defineAsyncComponent, nextTick, ref, Teleport, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'

const ChatStorageSettingsPanel = defineAsyncComponent(() => import('@oclive/shared/components/settings/ChatStorageSettingsPanel.vue'))
const SettingsGeneralTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsGeneralTab.vue'))
const SettingsPluginsTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsPluginsTab.vue'))

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
    /** When set, switches the active tab (e.g. open from StatusBar identity link). */
    focusTab?: SettingsTab | null
  }>(),
  { embedded: false, focusTab: null },
)

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()
const roleStore = useRoleStore()

type SettingsTab = 'general' | 'plugins' | 'storage'
type GeneralSubTab = 'simple' | 'advanced'

const tab = ref<SettingsTab>('general')
const generalSubTab = ref<GeneralSubTab>('simple')

watch(
  () => props.focusTab,
  (next) => {
    if (next)
      tab.value = next
  },
)

watch(
  () => props.visible,
  (open) => {
    if (open && props.focusTab)
      tab.value = props.focusTab
  },
)

const settingsDialogRef = ref<HTMLElement | null>(null)

watch(
  () => roleStore.roleInfo.interactionMode,
  (mode) => {
    if (mode === 'pure_chat' && tab.value === 'plugins')
      tab.value = 'general'
  },
)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void nextTick(() => {
        settingsDialogRef.value?.focus({ preventScroll: true })
      })
    }
  },
)
</script>

<template>
  <component
    :is="embedded ? 'div' : Teleport"
    v-bind="embedded ? {} : { to: 'body' }"
  >
    <div
      v-if="visible"
      :class="embedded ? 'sv-embedded-root' : 'sv-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('settings.ariaDialog')"
      @click.self="!embedded && emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="settingsDialogRef"
        :class="embedded ? 'sv-embedded' : 'sv-dialog'"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <header v-if="!embedded" class="sv-head">
          <h2 class="sv-title">
            {{ t("settings.title") }}
          </h2>
          <button type="button" class="sv-close" :aria-label="t('settings.closeAria')" @click="emit('close')">
            ×
          </button>
        </header>

        <nav class="sv-nav" :class="{ 'sv-nav--vertical': embedded }" :aria-label="t('settings.ariaNav')">
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'general' ? 'page' : undefined"
            @click="tab = 'general'"
          >
            {{ t("settings.tabGeneral") }}
          </button>
          <button
            v-if="roleStore.interactionImmersive"
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'plugins' ? 'page' : undefined"
            @click="tab = 'plugins'"
          >
            {{ t("settings.tabPlugins") }}
          </button>
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'storage' ? 'page' : undefined"
            @click="tab = 'storage'"
          >
            {{ t("settings.tabStorage") }}
          </button>
        </nav>

        <SettingsGeneralTab
          v-show="tab === 'general'"
          v-model:general-sub-tab="generalSubTab"
          :visible="visible"
          :embedded="embedded"
        />

        <SettingsPluginsTab
          v-show="tab === 'plugins'"
          :bootstrap-epoch="pluginStore.bootstrapEpoch"
        />

        <div v-show="tab === 'storage'" class="sv-body">
          <ChatStorageSettingsPanel />
        </div>
      </div>
    </div>
  </component>
</template>

<style scoped>
.sv-embedded-root {
  flex: 1;
  width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.sv-embedded {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 0;
  padding: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.sv-embedded :deep(.sv-body) {
  flex: 1;
  min-width: 0;
  overflow: visible;
  padding: var(--tool-space-4, 16px);
  max-width: none;
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.sv-embedded :deep(.sv-section) {
  gap: var(--tool-space-3, 12px);
  padding-top: var(--tool-space-6, 24px);
  margin-bottom: 0;
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}

.sv-embedded :deep(.ui-section) {
  margin-bottom: 0;
}

.sv-embedded :deep(.sv-body > .sv-lead + .sv-section),
.sv-embedded :deep(.sv-body > .sv-section:first-child) {
  border-top: none;
  padding-top: 0;
}

.sv-embedded :deep(.sv-label) {
  font-size: var(--tool-fs-md, 13px);
}

.sv-nav--vertical {
  position: sticky;
  top: 0;
  align-self: flex-start;
  flex: 0 0 132px;
  flex-shrink: 0;
  flex-direction: column;
  align-items: stretch;
  gap: var(--tool-space-1, 4px);
  padding: var(--tool-space-3, 12px);
  border-bottom: none;
  border-right: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-sidebar, var(--tool-bg, var(--bg-secondary)));
}

.sv-nav--vertical .sv-nav-btn {
  width: 100%;
  text-align: left;
  border-radius: var(--tool-radius, 4px);
  font-size: var(--tool-fs-md, 13px);
}

.sv-nav--vertical .sv-nav-btn[aria-current="page"] {
  border-color: transparent;
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 12%, transparent);
  color: var(--tool-text, var(--text-primary));
}

.sv-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10040;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sv-dialog {
  position: relative;
  width: min(640px, 100%);
  max-height: min(90vh, 800px);
  overflow: auto;
  padding: 16px 18px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.sv-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
}
.sv-title {
  margin: 0;
  font-size: 18px;
}
.sv-close {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sv-nav {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid var(--border-light);
  padding-bottom: 8px;
}
.sv-nav-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-nav-btn[aria-current="page"] {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.sv-body {
  flex: 1;
  min-height: 0;
}
</style>

<style>
@import '@oclive/shared/styles/win98/panel-settings.css';
</style>
