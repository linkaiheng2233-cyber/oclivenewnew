<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { installPluginFromZip } from '@oclive/shared/api'
import { showPluginInstallReviewHint } from '@oclive/shared/composables/usePluginInstallReviewHint'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { usePluginSlotEnable } from '@oclive/shared/composables/usePluginSlotEnable'
import { usePluginMarketStore } from '@oclive/shared/stores/pluginMarketStore'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import PluginUiSlotSelectorDialog from '../PluginUiSlotSelectorDialog.vue'
import UiButton from '../ui/UiButton.vue'

const props = defineProps<{
  visible: boolean
  embedded?: boolean
}>()

const emit = defineEmits<{
  close: []
  openMarket: []
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()
const marketStore = usePluginMarketStore()
const { showToast } = useAppToast()
const {
  selector,
  closeSelector,
  toggleSlotChoice,
  applySelectorAndEnable,
  setPluginEnabled,
} = usePluginSlotEnable()

const busyId = ref<string | null>(null)
const selectorBusy = ref(false)

const rows = computed(() =>
  pluginStore.catalog.map(c => ({
    id: c.id,
    version: c.version,
    disabled: pluginStore.isPluginDisabled(c.id),
  })),
)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void pluginStore.refresh()
    }
  },
)

async function onToggleEnabled(id: string, enabled: boolean): Promise<void> {
  busyId.value = id
  try {
    const openedSelector = await setPluginEnabled(id, enabled)
    if (!openedSelector && enabled) {
      showToast('success', t('simplePluginManager.enabled', { id }))
    }
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    busyId.value = null
  }
}

async function onConfirmSlotSelector(): Promise<void> {
  selectorBusy.value = true
  try {
    await applySelectorAndEnable()
    showToast('success', t('simplePluginManager.enabled', { id: selector.value.pluginId }))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    selectorBusy.value = false
  }
}

async function onUninstall(id: string): Promise<void> {
  if (!window.confirm(t('simplePluginManager.confirmUninstall', { id })))
    return
  busyId.value = id
  try {
    await marketStore.uninstallPluginFromGitIndex(id)
    showToast('success', t('simplePluginManager.uninstalled', { id }))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    busyId.value = null
  }
}

async function onInstallZip(): Promise<void> {
  const path = await open({
    multiple: false,
    filters: [
      { name: t('pluginWorkbench.localZipFilterName'), extensions: ['zip'] },
    ],
  })
  if (path === null || Array.isArray(path))
    return
  busyId.value = '__install__'
  try {
    const result = await installPluginFromZip(path)
    await pluginStore.refresh()
    showToast('success', t('simplePluginManager.installed', { id: result.installedPluginId }))
    showPluginInstallReviewHint(showToast, result)
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    busyId.value = null
  }
}
</script>

<template>
  <div class="spm-root tool-mgmt-panel">
    <header class="spm-toolbar">
      <UiButton
        size="sm"
        variant="primary"
        :disabled="busyId === '__install__'"
        @click="onInstallZip"
      >
        {{
          busyId === "__install__"
            ? t("simplePluginManager.installingZip")
            : t("simplePluginManager.installZip")
        }}
      </UiButton>
      <UiButton size="sm" variant="secondary" @click="emit('openMarket')">
        {{ t("simplePluginManager.browseMarket") }}
      </UiButton>
      <UiButton
        v-if="!embedded"
        size="sm"
        variant="ghost"
        class="spm-close"
        :aria-label="t('simplePluginManager.close')"
        @click="emit('close')"
      >
        ×
      </UiButton>
    </header>

    <p v-if="pluginStore.error" class="spm-error" role="alert">
      {{ pluginStore.error }}
    </p>
    <p v-if="pluginStore.loading" class="spm-muted">
      {{ t("simplePluginManager.loading") }}
    </p>

    <ul v-else class="spm-list" role="list">
      <li v-if="rows.length === 0" class="spm-empty">
        {{ t("simplePluginManager.empty") }}
      </li>
      <li v-for="row in rows" :key="row.id" class="spm-row">
        <span class="spm-title">{{ row.id }}</span>
        <span class="spm-ver">v{{ row.version }}</span>
        <label class="spm-switch" :title="t('simplePluginManager.toggleHint')">
          <input
            type="checkbox"
            :checked="!row.disabled"
            :disabled="busyId === row.id"
            @change="onToggleEnabled(row.id, ($event.target as HTMLInputElement).checked)"
          >
          <span class="spm-switch-ui" />
        </label>
        <button
          type="button"
          class="spm-uninstall"
          :disabled="busyId === row.id"
          @click="onUninstall(row.id)"
        >
          {{ t("simplePluginManager.uninstall") }}
        </button>
      </li>
    </ul>

    <PluginUiSlotSelectorDialog
      :state="selector"
      :busy="selectorBusy"
      @close="closeSelector"
      @confirm="onConfirmSlotSelector"
      @toggle-slot="toggleSlotChoice"
    />
  </div>
</template>

<style scoped>
.spm-root {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
  flex: 1;
}
.spm-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: var(--tool-space-2, 8px);
  align-items: center;
}
.spm-close {
  margin-left: auto;
  min-width: 2rem;
}
.spm-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow: auto;
  flex: 1;
  min-height: 0;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, 4px);
  background: var(--tool-chrome-editor, var(--bg-primary));
}
.spm-row {
  display: grid;
  grid-template-columns: 1fr auto auto auto;
  gap: var(--tool-space-3, 12px);
  align-items: center;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px);
  min-height: var(--tool-row-h, 32px);
  border-bottom: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}
.spm-row:last-child {
  border-bottom: none;
}
.spm-title {
  font-weight: 600;
  font-size: var(--tool-fs-md, 13px);
  overflow: hidden;
  text-overflow: ellipsis;
}
.spm-ver {
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-muted, #64748b));
}
.spm-uninstall {
  padding: 0 var(--tool-space-2, 8px);
  min-height: var(--tool-control-h-sm, 24px);
  border: 1px solid color-mix(in srgb, var(--tool-danger, #b91c1c) 35%, var(--tool-border, var(--border-light)));
  border-radius: var(--tool-radius, 4px);
  background: transparent;
  color: var(--tool-danger, #b91c1c);
  font-size: var(--tool-fs-sm, 12px);
  cursor: pointer;
}
.spm-uninstall:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.spm-switch {
  position: relative;
  display: inline-flex;
  cursor: pointer;
}
.spm-switch input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}
.spm-switch-ui {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: var(--border-light);
  transition: background 0.15s;
}
.spm-switch-ui::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.15s;
  box-shadow: 0 1px 2px rgb(0 0 0 / 20%);
}
.spm-switch input:checked + .spm-switch-ui {
  background: var(--accent, #3b82f6);
}
.spm-switch input:checked + .spm-switch-ui::after {
  transform: translateX(18px);
}
.spm-empty,
.spm-muted,
.spm-error {
  padding: 12px;
  margin: 0;
}
.spm-error {
  color: #b91c1c;
}
</style>
