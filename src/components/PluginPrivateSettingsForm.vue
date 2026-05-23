<script setup lang="ts">
import type { PluginUiSettingsDto, UiSchemaFieldDto } from '../api'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '../composables/useAppToast'
import {
  getPluginSettingsUi,

  setPluginSettingsConfig,

} from '../api'

const props = defineProps<{ pluginId: string }>()

const { showToast } = useAppToast()
const { t } = useI18n()
const loading = ref(true)
const saving = ref(false)
const dto = ref<PluginUiSettingsDto | null>(null)
const draft = ref<Record<string, unknown>>({})

const fields = computed(() => dto.value?.fields ?? [])

function fieldValue(key: string): unknown {
  return draft.value[key]
}

function setField(key: string, v: unknown) {
  draft.value = { ...draft.value, [key]: v }
}

function coerceInput(f: UiSchemaFieldDto, raw: string): unknown {
  const fieldType = f.type.trim().toLowerCase()
  if (fieldType === 'number') {
    const n = Number(raw)
    return Number.isFinite(n) ? n : 0
  }
  if (fieldType === 'bool' || fieldType === 'boolean') {
    return raw === 'true' || raw === '1'
  }
  return raw
}

async function load() {
  const pid = props.pluginId.trim()
  if (!pid) {
    dto.value = null
    loading.value = false
    return
  }
  loading.value = true
  try {
    const r = await getPluginSettingsUi(pid)
    dto.value = r
    const base
      = r.config && typeof r.config === 'object' && !Array.isArray(r.config)
        ? { ...(r.config as Record<string, unknown>) }
        : {}
    for (const f of r.fields) {
      const k = f.key.trim()
      if (!k || k in base)
        continue
      if (f.default !== undefined && f.default !== null) {
        base[k] = f.default as unknown
      }
    }
    draft.value = base
  }
  catch (e) {
    dto.value = null
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
}

onMounted(load)
watch(
  () => props.pluginId,
  () => {
    void load()
  },
)

async function onSave() {
  const pid = props.pluginId.trim()
  if (!pid)
    return
  saving.value = true
  try {
    await setPluginSettingsConfig(pid, draft.value)
    showToast('success', t('pluginManager.privateSettings.toastSaved'))
    await load()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="ppsf">
    <div v-if="loading" class="ppsf-muted">
      {{ t("pluginManager.privateSettings.loading") }}
    </div>
    <div v-else-if="!dto?.fields?.length" class="ppsf-muted">
      {{ t("pluginManager.privateSettings.noFields") }}
    </div>
    <template v-else>
      <p v-if="dto.uiTemplate" class="ppsf-hint">
        {{ t("pluginManager.privateSettings.templatePrefix") }}<code>{{ dto.uiTemplate }}</code>
      </p>
      <div class="ppsf-fields">
        <label v-for="f in fields" :key="f.key" class="ppsf-row">
          <span class="ppsf-label">
            {{ f.label || f.key }}
            <span v-if="f.required" class="ppsf-req">*</span>
          </span>
          <template v-if="f.type === 'number'">
            <input
              type="number"
              class="ppsf-input"
              :value="String(fieldValue(f.key) ?? '')"
              @input="
                setField(f.key, coerceInput(f, ($event.target as HTMLInputElement).value))
              "
            >
          </template>
          <template v-else-if="f.type === 'bool' || f.type === 'boolean'">
            <input
              type="checkbox"
              :checked="Boolean(fieldValue(f.key))"
              @change="
                setField(f.key, ($event.target as HTMLInputElement).checked)
              "
            >
          </template>
          <template v-else>
            <input
              type="text"
              class="ppsf-input"
              :value="String(fieldValue(f.key) ?? '')"
              @input="
                setField(f.key, ($event.target as HTMLInputElement).value)
              "
            >
          </template>
        </label>
      </div>
      <button type="button" class="ppsf-save" :disabled="saving" @click="onSave">
        {{ saving ? t("pluginManager.privateSettings.saving") : t("pluginManager.privateSettings.save") }}
      </button>
    </template>
  </div>
</template>

<style scoped>
.ppsf {
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 12px;
  background: var(--surface-1);
}
.ppsf-muted {
  font-size: 12px;
  color: var(--text-secondary);
}
.ppsf-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0 0 10px;
}
.ppsf-hint code {
  font-size: 11px;
}
.ppsf-fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ppsf-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
.ppsf-label {
  color: var(--text-secondary);
}
.ppsf-req {
  color: var(--danger, #c44);
}
.ppsf-input {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--surface-0);
  color: var(--text-primary);
  font-size: 13px;
}
.ppsf-save {
  margin-top: 12px;
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  background: var(--accent);
  color: var(--accent-fg, #fff);
}
.ppsf-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
