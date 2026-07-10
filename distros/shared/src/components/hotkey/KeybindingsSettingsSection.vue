<script setup lang="ts">
import type { HotkeyBinding, HotkeyBindingsFile } from '@oclive/shared/api'
import { getHotkeyBindings, saveHotkeyBindings } from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import {
  KEYBINDING_ACTIONS,
  actionScope,
  createDefaultUnifiedBindingsFile,
  describeBindingOrUnbound,
  detectBindingConflicts,
  eventToBinding,
  getDefaultBindingForAction,
  getKeybindingAction,
  getPrimaryEffectiveAcceleratorForAction,
  isValidHoldAccelerator,
  loadUnifiedBindingsFile,
  normalizeAccelerator,
  saveUnifiedBindingsFile,
  type KeybindingActionId,
  type UnifiedBinding,
  type UnifiedBindingsFileV1,
} from '@oclive/shared/lib/keybindings'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import UiButton from '../ui/UiButton.vue'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiSection from '../ui/UiSection.vue'
import UiSelect from '../ui/UiSelect.vue'

const { t } = useI18n()
const { showToast } = useAppToast()

const loading = ref(false)
const file = ref<UnifiedBindingsFileV1>(loadUnifiedBindingsFile())

const capturingId = ref<string | null>(null)

const conflicts = computed(() => detectBindingConflicts(file.value))
const hasConflicts = computed(() => conflicts.value.length > 0)

const rows = computed(() => {
  const actions = KEYBINDING_ACTIONS.filter(a => a.scope !== 'global')
  const map = new Map<string, UnifiedBinding[]>()
  for (const b of file.value.bindings) {
    const arr = map.get(b.actionId) ?? []
    arr.push(b)
    map.set(b.actionId, arr)
  }
  return actions.map(a => ({
    action: a,
    bindings: map.get(a.id) ?? [],
  }))
})

const globalRows = computed(() => file.value.bindings.filter(isGlobalRow))

function ensureRowForAction(actionId: KeybindingActionId): UnifiedBinding {
  const existing = file.value.bindings.find(b => b.actionId === actionId)
  if (existing)
    return existing
  const next: UnifiedBinding = {
    id: typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `kb-${Date.now()}`,
    actionId,
    accelerator: getDefaultBindingForAction(actionId),
    enabled: true,
  }
  file.value = { schemaVersion: 1, bindings: [...file.value.bindings, next] }
  return next
}

function setBindingAccelerator(rowId: string, accel: string): void {
  file.value = {
    schemaVersion: 1,
    bindings: file.value.bindings.map((b) => {
      if (b.id !== rowId)
        return b
      return { ...b, accelerator: accel }
    }),
  }
}

function setBindingEnabled(rowId: string, enabled: boolean): void {
  file.value = {
    schemaVersion: 1,
    bindings: file.value.bindings.map((b) => {
      if (b.id !== rowId)
        return b
      return { ...b, enabled }
    }),
  }
}

function keyForHold(e: KeyboardEvent): string {
  if (e.code && /^Key[A-Z]$/.test(e.code))
    return e.code.slice(3)
  const k = (e.key || '').toUpperCase()
  if (k === ' ')
    return 'Space'
  if (k === 'ESCAPE')
    return 'Esc'
  return k
}

function onCaptureKeydown(e: KeyboardEvent, rowId: string, actionId: KeybindingActionId): void {
  if (capturingId.value !== rowId)
    return
  e.preventDefault()
  e.stopPropagation()

  if (e.key === 'Escape') {
    setBindingAccelerator(rowId, '')
    capturingId.value = null
    return
  }

  const scope = actionScope(actionId)
  if (scope === 'hold') {
    const k = normalizeAccelerator(keyForHold(e))
    if (isValidHoldAccelerator(k)) {
      setBindingAccelerator(rowId, k)
      capturingId.value = null
    }
    return
  }

  const accel = normalizeAccelerator(eventToBinding(e))
  if (!accel)
    return
  // ignore pure modifiers
  if (accel === 'Ctrl' || accel === 'Shift' || accel === 'Alt' || accel === 'Meta')
    return
  setBindingAccelerator(rowId, accel)
  capturingId.value = null
}

function beginCapture(rowId: string): void {
  capturingId.value = rowId
}

function clearCapture(): void {
  capturingId.value = null
}

function removeRow(rowId: string): void {
  file.value = {
    schemaVersion: 1,
    bindings: file.value.bindings.filter(b => b.id !== rowId),
  }
}

function addGlobalRow(actionId: 'plugin.openLauncher' | 'plugin.openSlot'): void {
  const id = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `kb-${Date.now()}`
  const next: UnifiedBinding = {
    id,
    actionId,
    accelerator: '',
    enabled: false,
    params: actionId === 'plugin.openSlot'
      ? { pluginId: '', slot: 'chat_toolbar', appearanceId: '' }
      : undefined,
  }
  file.value = { schemaVersion: 1, bindings: [...file.value.bindings, next] }
}

function setGlobalActionType(rowId: string, actionType: string): void {
  file.value = {
    schemaVersion: 1,
    bindings: file.value.bindings.map((b) => {
      if (b.id !== rowId)
        return b
      if (actionType === 'plugin.openLauncher') {
        return { ...b, actionId: 'plugin.openLauncher', params: undefined }
      }
      return {
        ...b,
        actionId: 'plugin.openSlot',
        params: { pluginId: '', slot: 'chat_toolbar', appearanceId: '' },
      }
    }),
  }
}

function setRowParams(rowId: string, patch: Record<string, unknown>): void {
  file.value = {
    schemaVersion: 1,
    bindings: file.value.bindings.map((b) => {
      if (b.id !== rowId)
        return b
      return { ...b, params: { ...(b.params ?? {}), ...patch } }
    }),
  }
}

function toHotkeyBindingsFile(f: UnifiedBindingsFileV1): HotkeyBindingsFile {
  const bindings: HotkeyBinding[] = f.bindings
    .filter(b => actionScope(b.actionId) === 'global')
    .map((b) => {
      const accel = normalizeAccelerator(b.accelerator)
      const enabled = b.enabled === true && accel.length > 0
      if (b.actionId === 'plugin.openLauncher') {
        return {
          id: b.id,
          accelerator: accel,
          enabled,
          action: { type: 'openLauncherList' },
        }
      }
      const pluginId = String((b.params as any)?.pluginId ?? '')
      const slot = String((b.params as any)?.slot ?? '')
      const appearanceId = String((b.params as any)?.appearanceId ?? '')
      return {
        id: b.id,
        accelerator: accel,
        enabled,
        action: {
          type: 'openPluginSlot',
          pluginId,
          slot,
          appearanceId: appearanceId || undefined,
        },
      }
    })
  return { schemaVersion: 1, bindings }
}

function mergeGlobalBindingsIntoUnified(global: HotkeyBindingsFile): void {
  const globals: UnifiedBinding[] = global.bindings.map((b) => {
    if (b.action.type === 'openLauncherList') {
      return {
        id: b.id,
        actionId: 'plugin.openLauncher',
        accelerator: b.accelerator,
        enabled: b.enabled,
      }
    }
    return {
      id: b.id,
      actionId: 'plugin.openSlot',
      accelerator: b.accelerator,
      enabled: b.enabled,
      params: {
        pluginId: b.action.pluginId,
        slot: b.action.slot,
        appearanceId: b.action.appearanceId ?? '',
      },
    }
  })
  const nonGlobal = file.value.bindings.filter(b => actionScope(b.actionId) !== 'global')
  file.value = { schemaVersion: 1, bindings: [...nonGlobal, ...globals] }
}

onMounted(async () => {
  loading.value = true
  try {
    // Ensure defaults exist even if storage is empty/corrupt
    const local = loadUnifiedBindingsFile()
    file.value = local.schemaVersion === 1 ? local : createDefaultUnifiedBindingsFile()
    const global = await getHotkeyBindings()
    mergeGlobalBindingsIntoUnified(global)
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
})

async function onSave(): Promise<void> {
  if (hasConflicts.value) {
    showToast('error', t('keybindings.conflictToast'))
    return
  }
  // basic hold validation
  const holdRows = file.value.bindings.filter(b => actionScope(b.actionId) === 'hold' && b.enabled)
  for (const r of holdRows) {
    if (r.accelerator && !isValidHoldAccelerator(r.accelerator)) {
      showToast('error', t('keybindings.holdInvalidToast'))
      return
    }
  }

  loading.value = true
  try {
    saveUnifiedBindingsFile(file.value)
    const globalFile = toHotkeyBindingsFile(file.value)
    await saveHotkeyBindings(globalFile)
    showToast('success', t('keybindings.savedToast'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
}

function onResetDefaults(): void {
  file.value = createDefaultUnifiedBindingsFile()
}

function effectiveLabel(actionId: KeybindingActionId): string {
  return describeBindingOrUnbound(getPrimaryEffectiveAcceleratorForAction(file.value, actionId))
}

function isGlobalRow(b: UnifiedBinding): boolean {
  return actionScope(b.actionId) === 'global'
}
</script>

<template>
  <UiSection :title="t('keybindings.title')" :description="t('keybindings.lead')">
    <p v-if="loading" class="kb-muted">
      {{ t("common.loading") }}
    </p>
    <div v-else class="kb-body">
      <div v-if="hasConflicts" class="kb-warn" role="alert">
        {{ t("keybindings.conflictInline") }}
      </div>

      <div class="kb-grid">
        <div v-for="row in rows" :key="row.action.id" class="kb-row">
          <div class="kb-row__title">
            <div class="kb-row__name">
              {{ t(row.action.titleKey) }}
            </div>
            <div class="kb-row__meta">
              <span class="kb-pill">{{ row.action.scope }}</span>
              <span v-if="row.action.immersiveOnly" class="kb-pill kb-pill--muted">
                {{ t("keybindings.immersiveOnly") }}
              </span>
            </div>
          </div>

          <div class="kb-row__main">
            <template v-if="row.action.scope !== 'global'">
              <UiFieldRow :label="t('keybindings.current')">
                <span class="kb-mono">{{ effectiveLabel(row.action.id) }}</span>
              </UiFieldRow>
              <UiFieldRow :label="t('keybindings.capture')">
                <div class="kb-capture">
                  <UiButton
                    size="sm"
                    variant="secondary"
                    type="button"
                    :disabled="capturingId === ensureRowForAction(row.action.id).id"
                    @click="beginCapture(ensureRowForAction(row.action.id).id)"
                  >
                    {{
                      capturingId === ensureRowForAction(row.action.id).id
                        ? t("keybindings.capturing")
                        : t("keybindings.captureBtn")
                    }}
                  </UiButton>
                  <UiButton
                    size="sm"
                    variant="ghost"
                    type="button"
                    @click="setBindingAccelerator(ensureRowForAction(row.action.id).id, '')"
                  >
                    {{ t("keybindings.clear") }}
                  </UiButton>
                  <UiButton
                    v-if="row.action.id === 'app.openShortcutHelp'"
                    size="sm"
                    variant="ghost"
                    type="button"
                    @click="setBindingAccelerator(ensureRowForAction(row.action.id).id, 'Ctrl+LongPress')"
                  >
                    {{ t("keybindings.useCtrlLongPress") }}
                  </UiButton>
                  <UiButton
                    v-if="capturingId === ensureRowForAction(row.action.id).id"
                    size="sm"
                    variant="ghost"
                    type="button"
                    @click="clearCapture"
                  >
                    {{ t("common.cancel") }}
                  </UiButton>
                </div>
                <div
                  v-if="capturingId === ensureRowForAction(row.action.id).id"
                  class="kb-capture__hint"
                  tabindex="0"
                  @keydown.capture="onCaptureKeydown($event, ensureRowForAction(row.action.id).id, row.action.id)"
                >
                  {{ t("keybindings.captureHint") }}
                </div>
              </UiFieldRow>
              <UiFieldRow :label="t('keybindings.enabled')">
                <label class="kb-chk">
                  <input
                    type="checkbox"
                    :checked="ensureRowForAction(row.action.id).enabled"
                    @change="setBindingEnabled(ensureRowForAction(row.action.id).id, ($event.target as HTMLInputElement).checked)"
                  >
                </label>
              </UiFieldRow>
            </template>

            <template v-else />
          </div>
        </div>
      </div>

      <div class="kb-row">
        <div class="kb-row__title">
          <div class="kb-row__name">
            {{ t("keybindings.globalTitle") }}
          </div>
        </div>
        <div class="kb-global-lead">
          {{ t("keybindings.globalLead") }}
        </div>
        <div class="kb-global-list">
          <div v-for="b in globalRows" :key="b.id" class="kb-global-card">
            <UiFieldRow :label="t('keybindings.globalAction')">
              <UiSelect
                :model-value="b.actionId"
                @change="setGlobalActionType(b.id, ($event.target as HTMLSelectElement).value)"
              >
                <option value="plugin.openLauncher">
                  {{ t(getKeybindingAction('plugin.openLauncher')!.titleKey) }}
                </option>
                <option value="plugin.openSlot">
                  {{ t(getKeybindingAction('plugin.openSlot')!.titleKey) }}
                </option>
              </UiSelect>
            </UiFieldRow>
            <UiFieldRow :label="t('keybindings.globalAccelerator')">
              <input
                :value="b.accelerator"
                type="text"
                class="ui-input kb-input"
                :placeholder="t('keybindings.accelPlaceholder')"
                @input="setBindingAccelerator(b.id, ($event.target as HTMLInputElement).value)"
              >
            </UiFieldRow>
            <UiFieldRow :label="t('keybindings.enabled')">
              <label class="kb-chk">
                <input
                  type="checkbox"
                  :checked="b.enabled"
                  @change="setBindingEnabled(b.id, ($event.target as HTMLInputElement).checked)"
                >
              </label>
            </UiFieldRow>
            <template v-if="b.actionId === 'plugin.openSlot'">
              <UiFieldRow :label="t('keybindings.pluginId')">
                <input
                  :value="String((b.params as any)?.pluginId ?? '')"
                  type="text"
                  class="ui-input kb-input"
                  @input="setRowParams(b.id, { pluginId: ($event.target as HTMLInputElement).value })"
                >
              </UiFieldRow>
              <UiFieldRow :label="t('keybindings.slotName')">
                <input
                  :value="String((b.params as any)?.slot ?? '')"
                  type="text"
                  class="ui-input kb-input"
                  @input="setRowParams(b.id, { slot: ($event.target as HTMLInputElement).value })"
                >
              </UiFieldRow>
              <UiFieldRow :label="t('keybindings.appearanceOptional')">
                <input
                  :value="String((b.params as any)?.appearanceId ?? '')"
                  type="text"
                  class="ui-input kb-input"
                  @input="setRowParams(b.id, { appearanceId: ($event.target as HTMLInputElement).value })"
                >
              </UiFieldRow>
            </template>
            <div class="kb-global-card__foot">
              <UiButton size="sm" variant="ghost" type="button" @click="removeRow(b.id)">
                {{ t("common.remove") }}
              </UiButton>
            </div>
          </div>
        </div>
        <div class="kb-global-actions">
          <UiButton size="sm" variant="secondary" type="button" @click="addGlobalRow('plugin.openLauncher')">
            {{ t("keybindings.addGlobalLauncher") }}
          </UiButton>
          <UiButton size="sm" variant="secondary" type="button" @click="addGlobalRow('plugin.openSlot')">
            {{ t("keybindings.addGlobalSlot") }}
          </UiButton>
        </div>
      </div>

      <div class="kb-actions">
        <UiButton size="sm" variant="secondary" type="button" @click="onResetDefaults">
          {{ t("keybindings.resetDefaults") }}
        </UiButton>
        <UiButton size="sm" variant="primary" type="button" :disabled="loading" @click="onSave">
          {{ t("keybindings.save") }}
        </UiButton>
      </div>
    </div>
  </UiSection>
</template>

<style scoped>
.kb-muted {
  margin: 0;
  font-size: var(--tool-fs-md, 13px);
  color: var(--tool-text-muted, var(--text-secondary));
}
.kb-body {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-3, 12px);
}
.kb-warn {
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--danger, #b91c1c) 35%, var(--border-light));
  background: color-mix(in srgb, var(--danger, #b91c1c) 10%, transparent);
  font-size: 12px;
  color: var(--text-primary);
}
.kb-grid {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-3, 12px);
}
.kb-row {
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: 10px;
  padding: 12px;
  background: var(--bg-secondary);
}
.kb-row__title {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}
.kb-row__name {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}
.kb-row__meta {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.kb-pill {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  background: var(--bg-primary);
}
.kb-pill--muted {
  opacity: 0.8;
}
.kb-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
.kb-capture {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.kb-capture__hint {
  margin-top: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px dashed var(--border-light);
  background: color-mix(in srgb, var(--border-light) 20%, transparent);
  font-size: 12px;
  color: var(--text-secondary);
  outline: none;
}
.kb-chk {
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
}
.kb-input {
  width: 100%;
}
.kb-global-lead {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.kb-global-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.kb-global-card {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.kb-global-card__foot {
  display: flex;
  justify-content: flex-end;
  padding-top: 6px;
}
.kb-global-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
}
.kb-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
</style>

