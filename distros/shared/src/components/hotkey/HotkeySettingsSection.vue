<script setup lang="ts">
import type { HotkeyBinding, HotkeyBindingsFile } from '@oclive/shared/api'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getHotkeyBindings,
  saveHotkeyBindings,
} from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import UiButton from '../ui/UiButton.vue'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiSection from '../ui/UiSection.vue'
import UiSelect from '../ui/UiSelect.vue'

const { t } = useI18n()
const { showToast } = useAppToast()

const loading = ref(false)
const file = ref<HotkeyBindingsFile>({ schemaVersion: 1, bindings: [] })

onMounted(async () => {
  loading.value = true
  try {
    file.value = await getHotkeyBindings()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
})

function addBinding(): void {
  const id
    = typeof crypto !== 'undefined' && crypto.randomUUID
      ? crypto.randomUUID()
      : `hk-${Date.now()}`
  const next: HotkeyBinding = {
    id,
    accelerator: '',
    enabled: false,
    action: { type: 'openLauncherList' },
  }
  file.value = {
    ...file.value,
    bindings: [...file.value.bindings, next],
  }
}

function removeAt(i: number): void {
  const next = [...file.value.bindings]
  next.splice(i, 1)
  file.value = { ...file.value, bindings: next }
}

function setActionType(i: number, actionType: string): void {
  const next = [...file.value.bindings]
  const b = next[i]
  if (!b)
    return
  if (actionType === 'openLauncherList') {
    b.action = { type: 'openLauncherList' }
  }
  else {
    b.action = {
      type: 'openPluginSlot',
      pluginId: '',
      slot: 'chat_toolbar',
      appearanceId: '',
    }
  }
  file.value = { ...file.value, bindings: next }
}

async function onSave(): Promise<void> {
  loading.value = true
  try {
    await saveHotkeyBindings(file.value)
    showToast('success', t('hotkeys.savedToast'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
}
</script>

<template>
  <UiSection :title="t('hotkeys.title')" :description="t('hotkeys.lead')">
    <p v-if="loading" class="hkset-muted">
      {{ t("common.loading") }}
    </p>
    <form v-else class="hkset-form" @submit.prevent="onSave">
      <div v-for="(b, i) in file.bindings" :key="b.id" class="hkset-card">
        <UiFieldRow :label="t('hotkeys.fieldAccelerator')">
          <input
            v-model="b.accelerator"
            type="text"
            class="ui-input hkset-input"
            :placeholder="t('hotkeys.accelPlaceholder')"
          >
        </UiFieldRow>
        <UiFieldRow :label="t('hotkeys.enabled')">
          <label class="hkset-chk">
            <input v-model="b.enabled" type="checkbox">
          </label>
        </UiFieldRow>
        <UiFieldRow :label="t('hotkeys.action')">
          <UiSelect
            :model-value="b.action.type"
            @change="setActionType(i, ($event.target as HTMLSelectElement).value)"
          >
            <option value="openLauncherList">
              {{ t("hotkeys.actionOpenLauncher") }}
            </option>
            <option value="openPluginSlot">
              {{ t("hotkeys.actionOpenSlot") }}
            </option>
          </UiSelect>
        </UiFieldRow>
        <template v-if="b.action.type === 'openPluginSlot'">
          <UiFieldRow :label="t('hotkeys.pluginId')">
            <input v-model="b.action.pluginId" type="text" class="ui-input hkset-input">
          </UiFieldRow>
          <UiFieldRow :label="t('hotkeys.slotName')">
            <input v-model="b.action.slot" type="text" class="ui-input hkset-input">
          </UiFieldRow>
          <UiFieldRow :label="t('hotkeys.appearanceOptional')">
            <input v-model="b.action.appearanceId" type="text" class="ui-input hkset-input">
          </UiFieldRow>
        </template>
        <div class="hkset-card__foot">
          <UiButton size="sm" variant="ghost" type="button" @click="removeAt(i)">
            {{ t("hotkeys.remove") }}
          </UiButton>
        </div>
      </div>
      <div class="hkset-actions">
        <UiButton size="sm" variant="secondary" type="button" @click="addBinding">
          {{ t("hotkeys.addRow") }}
        </UiButton>
        <UiButton size="sm" variant="primary" type="submit">
          {{ t("hotkeys.save") }}
        </UiButton>
      </div>
    </form>
  </UiSection>
</template>

<style scoped>
.hkset-muted {
  margin: 0;
  font-size: var(--tool-fs-md, 13px);
  color: var(--tool-text-muted, var(--text-secondary));
}
.hkset-form {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-3, 12px);
}
.hkset-card {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-2, 8px);
  padding: var(--tool-space-3, 12px);
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, 4px);
  background: var(--tool-chrome-sidebar, var(--tool-bg, var(--bg-secondary)));
}
.hkset-card__foot {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--tool-space-1, 4px);
}
.hkset-input {
  width: 100%;
}
.hkset-chk {
  display: flex;
  align-items: center;
  gap: var(--tool-space-2, 8px);
  font-size: var(--tool-fs-md, 13px);
  user-select: none;
  cursor: pointer;
}
.hkset-actions {
  display: flex;
  gap: var(--tool-space-2, 8px);
  flex-wrap: wrap;
}
</style>
