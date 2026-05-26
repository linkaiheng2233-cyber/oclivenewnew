<script setup lang="ts">
import { open as openDialog } from '@tauri-apps/api/dialog'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppToast } from '../../composables/useAppToast'
import {
  CLOUD_LLM_VENDORS,
  findCloudVendor,
  type CloudLlmVendorId,
} from '../../lib/cloudLlmVendors'
import {
  getLlmUserSettings,
  importGgufToOllama,
  listOllamaModels,
  openPathInFileManager,
  saveLlmUserSettings,
  scanLocalModelFiles,
  type LocalModelFile,
  type LlmUserSettings,
} from '../../api/llmSettings'
import { useRoleStore } from '../../stores/roleStore'

const emit = defineEmits<{
  openSettings: []
}>()

const roleStore = useRoleStore()
const { t, te } = useI18n()
const { showToast } = useAppToast()

const loading = ref(false)
const saving = ref(false)
const modelsLoading = ref(false)
const importing = ref(false)
const settings = ref<LlmUserSettings | null>(null)
const ollamaModels = ref<string[]>([])
const folderModelFiles = ref<LocalModelFile[]>([])

const providerTab = ref<'local' | 'cloud'>('local')
const ollamaBaseUrl = ref('')
const localModelsDir = ref('')
const selectedLocalModel = ref('')
const cloudVendorId = ref<CloudLlmVendorId>('deepseek')
const cloudApiStyle = ref<'openai' | 'oclive_jsonrpc'>('openai')
const remoteUrl = ref('')
const remoteToken = ref('')
const remoteModel = ref('')
const tokenTouched = ref(false)

const cloudVendorOptions = computed(() =>
  CLOUD_LLM_VENDORS.map(v => ({
    id: v.id,
    label: te(v.labelKey) ? t(v.labelKey) : v.id,
  })),
)

const cloudModelOptions = computed(() => {
  const preset = findCloudVendor(cloudVendorId.value)
  return preset?.models ?? []
})

const localModelSelectOptions = computed(() => {
  const ollama = ollamaModels.value.map(id => ({
    value: id,
    label: id,
    group: 'ollama' as const,
  }))
  const files = folderModelFiles.value.map(f => ({
    value: `file:${f.path}`,
    label: f.name,
    group: 'file' as const,
  }))
  return [...files, ...ollama]
})

const selectedLocalIsFile = computed(() => selectedLocalModel.value.startsWith('file:'))

const effectiveModel = computed(
  () => settings.value?.effectiveModel?.trim() || roleStore.roleInfo.effectiveOllamaModel?.trim() || '',
)

function vendorLabel(id: string): string {
  const key = `modelManager.vendors.${id}`
  return te(key) ? t(key) : id
}

function onCloudVendorChange(ev: Event): void {
  const id = (ev.target as HTMLSelectElement).value as CloudLlmVendorId
  cloudVendorId.value = id
  const preset = findCloudVendor(id)
  if (!preset) {
    return
  }
  cloudApiStyle.value = preset.apiStyle
  if (preset.baseUrl) {
    remoteUrl.value = preset.baseUrl
  }
  if (preset.models.length > 0 && !remoteModel.value) {
    remoteModel.value = preset.models[0]
  }
}

async function loadSettings(): Promise<void> {
  loading.value = true
  try {
    const s = await getLlmUserSettings(roleStore.currentRoleId)
    settings.value = s
    providerTab.value = s.provider === 'cloud' ? 'cloud' : 'local'
    ollamaBaseUrl.value = s.ollamaBaseUrl
    localModelsDir.value = s.localModelsDir
    folderModelFiles.value = s.localModelFiles ?? []
    cloudVendorId.value = (findCloudVendor(s.cloudVendor)?.id ?? 'custom') as CloudLlmVendorId
    cloudApiStyle.value = s.cloudApiStyle === 'oclive_jsonrpc' ? 'oclive_jsonrpc' : 'openai'
    remoteUrl.value = s.remoteUrl
    remoteModel.value = s.remoteModel || s.sessionOllamaModel || ''
    remoteToken.value = ''
    tokenTouched.value = false

    const session = s.sessionOllamaModel?.trim()
    if (session && folderModelFiles.value.some(f => f.path === session)) {
      selectedLocalModel.value = `file:${session}`
    }
    else {
      selectedLocalModel.value = session || s.packOllamaModel?.trim() || s.effectiveModel || ''
    }

    if (providerTab.value === 'local') {
      await refreshOllamaModels()
    }
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
}

async function refreshOllamaModels(): Promise<void> {
  modelsLoading.value = true
  try {
    ollamaModels.value = await listOllamaModels(ollamaBaseUrl.value)
    const cur = selectedLocalModel.value
    if (cur && !cur.startsWith('file:') && !ollamaModels.value.includes(cur)) {
      ollamaModels.value = [cur, ...ollamaModels.value]
    }
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    modelsLoading.value = false
  }
}

async function pickModelsFolder(): Promise<void> {
  const picked = await openDialog({
    directory: true,
    multiple: false,
    defaultPath: localModelsDir.value || undefined,
  })
  if (!picked || Array.isArray(picked)) {
    return
  }
  localModelsDir.value = picked
  folderModelFiles.value = await scanLocalModelFiles(picked)
  if (folderModelFiles.value.length > 0 && !selectedLocalModel.value) {
    selectedLocalModel.value = `file:${folderModelFiles.value[0].path}`
  }
}

async function scanCurrentFolder(): Promise<void> {
  if (!localModelsDir.value.trim()) {
    showToast('info', t('modelManager.pickFolderFirst'))
    return
  }
  folderModelFiles.value = await scanLocalModelFiles(localModelsDir.value)
}

async function openModelsFolder(): Promise<void> {
  if (!localModelsDir.value.trim()) {
    showToast('info', t('modelManager.pickFolderFirst'))
    return
  }
  await openPathInFileManager(localModelsDir.value)
}

async function importSelectedFileToOllama(): Promise<void> {
  if (!selectedLocalIsFile.value) {
    return
  }
  const path = selectedLocalModel.value.slice('file:'.length)
  importing.value = true
  try {
    const name = await importGgufToOllama({
      filePath: path,
      ollamaBaseUrl: ollamaBaseUrl.value,
    })
    showToast('success', t('modelManager.importOk', { name }))
    selectedLocalModel.value = name
    await refreshOllamaModels()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    importing.value = false
  }
}

async function resolveLocalModelForSave(): Promise<string> {
  const sel = selectedLocalModel.value.trim()
  if (sel.startsWith('file:')) {
    const path = sel.slice('file:'.length)
    const name = await importGgufToOllama({
      filePath: path,
      ollamaBaseUrl: ollamaBaseUrl.value,
    })
    return name
  }
  return sel
}

async function onSave(): Promise<void> {
  saving.value = true
  try {
    if (providerTab.value === 'local') {
      const model = await resolveLocalModelForSave()
      const info = await saveLlmUserSettings({
        roleId: roleStore.currentRoleId,
        provider: 'local',
        ollamaBaseUrl: ollamaBaseUrl.value.trim(),
        localModelsDir: localModelsDir.value.trim(),
        ollamaModel: model,
        cloudApiStyle: 'openai',
      })
      roleStore.applyRoleInfo(info)
    }
    else {
      const req = {
        roleId: roleStore.currentRoleId,
        provider: 'cloud' as const,
        cloudVendor: cloudVendorId.value,
        cloudApiStyle: cloudApiStyle.value,
        remoteUrl: remoteUrl.value.trim(),
        remoteModel: remoteModel.value.trim(),
      }
      if (tokenTouched.value) {
        Object.assign(req, { remoteToken: remoteToken.value })
      }
      const info = await saveLlmUserSettings(req)
      roleStore.applyRoleInfo(info)
    }
    showToast('success', t('modelManager.saveOk'))
    await loadSettings()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    saving.value = false
  }
}

watch(providerTab, (tab) => {
  if (tab === 'local' && ollamaModels.value.length === 0) {
    void refreshOllamaModels()
  }
})

watch(
  () => roleStore.currentRoleId,
  () => {
    void loadSettings()
  },
  { immediate: true },
)
</script>

<template>
  <div class="mm-root">
    <p class="mm-lead">
      {{ t("modelManager.lead") }}
    </p>

    <p v-if="loading" class="mm-muted">
      {{ t("modelManager.loading") }}
    </p>

    <template v-else>
      <div class="mm-effective" role="status">
        <span class="mm-effective-label">{{ t("modelManager.effectiveModelLabel") }}</span>
        <code class="mm-mono">{{ effectiveModel || t("modelManager.effectiveModelEmpty") }}</code>
      </div>

      <div class="mm-tabs" role="tablist" :aria-label="t('modelManager.providerTabsAria')">
        <button
          type="button"
          role="tab"
          class="mm-tab"
          :class="{ 'is-active': providerTab === 'local' }"
          :aria-selected="providerTab === 'local'"
          @click="providerTab = 'local'"
        >
          {{ t("modelManager.tabLocal") }}
        </button>
        <button
          type="button"
          role="tab"
          class="mm-tab"
          :class="{ 'is-active': providerTab === 'cloud' }"
          :aria-selected="providerTab === 'cloud'"
          @click="providerTab = 'cloud'"
        >
          {{ t("modelManager.tabCloud") }}
        </button>
      </div>

      <section v-show="providerTab === 'local'" class="mm-panel" role="tabpanel">
        <h3 class="mm-h3">
          {{ t("modelManager.localTitle") }}
        </h3>
        <p class="mm-muted">
          {{ t("modelManager.localLead") }}
        </p>

        <label class="mm-field">
          <span>{{ t("modelManager.ollamaBaseUrlLabel") }}</span>
          <input v-model="ollamaBaseUrl" type="url" class="mm-input" autocomplete="off">
        </label>

        <label class="mm-field">
          <span>{{ t("modelManager.localModelsDirLabel") }}</span>
          <input v-model="localModelsDir" type="text" class="mm-input" readonly>
        </label>
        <div class="mm-row-actions">
          <button type="button" class="mm-btn" @click="pickModelsFolder">
            {{ t("modelManager.pickModelsFolder") }}
          </button>
          <button
            type="button"
            class="mm-btn"
            :disabled="!localModelsDir"
            @click="openModelsFolder"
          >
            {{ t("modelManager.openModelsFolder") }}
          </button>
          <button type="button" class="mm-btn" @click="scanCurrentFolder">
            {{ t("modelManager.scanFolderModels") }}
          </button>
        </div>

        <div class="mm-row-actions">
          <button
            type="button"
            class="mm-btn"
            :disabled="modelsLoading || saving"
            @click="refreshOllamaModels"
          >
            {{ modelsLoading ? t("modelManager.refreshingModels") : t("modelManager.refreshModels") }}
          </button>
          <span
            v-if="settings"
            class="mm-status"
            :class="settings.ollamaReachable ? 'mm-ok' : 'mm-bad'"
          >
            {{
              settings.ollamaReachable
                ? t("settings.envCheckOllamaOk")
                : t("settings.envCheckOllamaFail")
            }}
          </span>
        </div>

        <label class="mm-field">
          <span>{{ t("modelManager.localModelLabel") }}</span>
          <select
            v-model="selectedLocalModel"
            class="mm-select"
            :disabled="localModelSelectOptions.length === 0 && !modelsLoading"
          >
            <option v-if="localModelSelectOptions.length === 0" value="">
              {{ t("modelManager.noLocalModels") }}
            </option>
            <optgroup v-if="folderModelFiles.length" :label="t('modelManager.folderModelsLabel')">
              <option
                v-for="f in folderModelFiles"
                :key="f.path"
                :value="`file:${f.path}`"
              >
                {{ f.name }}
              </option>
            </optgroup>
            <optgroup v-if="ollamaModels.length" label="Ollama">
              <option v-for="m in ollamaModels" :key="m" :value="m">
                {{ m }}
              </option>
            </optgroup>
          </select>
        </label>

        <button
          v-if="selectedLocalIsFile"
          type="button"
          class="mm-btn"
          :disabled="importing || saving"
          @click="importSelectedFileToOllama"
        >
          {{ importing ? t("modelManager.importingToOllama") : t("modelManager.importToOllama") }}
        </button>

        <p v-if="settings?.packOllamaModel" class="mm-muted mm-small">
          {{ t("modelManager.packDefaultModel", { model: settings.packOllamaModel }) }}
        </p>
      </section>

      <section v-show="providerTab === 'cloud'" class="mm-panel" role="tabpanel">
        <h3 class="mm-h3">
          {{ t("modelManager.cloudTitle") }}
        </h3>
        <p class="mm-muted">
          {{ t("modelManager.cloudLead") }}
        </p>

        <label class="mm-field">
          <span>{{ t("modelManager.cloudVendorLabel") }}</span>
          <select
            class="mm-select"
            :value="cloudVendorId"
            @change="onCloudVendorChange"
          >
            <option v-for="v in cloudVendorOptions" :key="v.id" :value="v.id">
              {{ v.label }}
            </option>
          </select>
        </label>

        <label class="mm-field">
          <span>{{ t("modelManager.cloudApiStyleLabel") }}</span>
          <select v-model="cloudApiStyle" class="mm-select">
            <option value="openai">
              {{ t("modelManager.apiStyleOpenai") }}
            </option>
            <option value="oclive_jsonrpc">
              {{ t("modelManager.apiStyleJsonRpc") }}
            </option>
          </select>
        </label>

        <label class="mm-field">
          <span>{{ t("modelManager.remoteUrlLabel") }}</span>
          <input
            v-model="remoteUrl"
            type="url"
            class="mm-input"
            :placeholder="t('modelManager.remoteUrlPlaceholder')"
          >
        </label>

        <label class="mm-field">
          <span>{{ t("modelManager.remoteTokenLabel") }}</span>
          <input
            v-model="remoteToken"
            type="password"
            class="mm-input"
            :placeholder="
              settings?.remoteTokenConfigured
                ? t('modelManager.remoteTokenPlaceholderSet')
                : t('modelManager.remoteTokenPlaceholder')
            "
            @input="tokenTouched = true"
          >
        </label>
        <p v-if="settings?.remoteTokenEnvActive" class="mm-hint">
          {{ t("modelManager.envTokenNote") }}
        </p>

        <label class="mm-field">
          <span>{{ t("modelManager.remoteModelLabel") }}</span>
          <select
            v-if="cloudModelOptions.length > 0"
            v-model="remoteModel"
            class="mm-select"
          >
            <option v-for="m in cloudModelOptions" :key="m" :value="m">
              {{ m }}
            </option>
          </select>
          <input
            v-else
            v-model="remoteModel"
            type="text"
            class="mm-input"
            :placeholder="t('modelManager.remoteModelPlaceholder')"
          >
        </label>
        <p class="mm-muted mm-small">
          {{ vendorLabel(cloudVendorId) }} · {{ cloudApiStyle === 'openai' ? 'OpenAI' : 'JSON-RPC' }}
        </p>
      </section>

      <footer class="mm-footer">
        <button type="button" class="mm-btn mm-btn--primary" :disabled="saving || loading" @click="onSave">
          {{ saving ? t("modelManager.saving") : t("modelManager.saveApply") }}
        </button>
        <button type="button" class="mm-btn mm-btn--ghost" @click="emit('openSettings')">
          {{ t("modelManager.openSettings") }}
        </button>
      </footer>
    </template>
  </div>
</template>

<style scoped>
.mm-root {
  flex: 1;
  min-height: 0;
  overflow: auto;
  font-size: 13px;
  color: var(--text-primary);
}
.mm-lead {
  margin: 0 0 12px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.mm-muted {
  margin: 0 0 10px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.mm-small {
  font-size: 11px;
}
.mm-effective {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 14px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--panel-bg-soft, var(--bg-elevated));
  border: 1px solid var(--border-light);
}
.mm-effective-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.mm-mono {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  word-break: break-word;
}
.mm-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
}
.mm-tab {
  flex: 1;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.mm-tab.is-active {
  border-color: var(--accent, #6b8cff);
  background: color-mix(in srgb, var(--accent, #6b8cff) 12%, var(--bg-elevated));
}
.mm-panel {
  margin-bottom: 12px;
}
.mm-h3 {
  margin: 0 0 6px;
  font-size: 0.95rem;
  font-weight: 600;
}
.mm-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}
.mm-input,
.mm-select {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: inherit;
  font-size: 13px;
}
.mm-select {
  font-family: ui-monospace, monospace;
}
.mm-row-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.mm-status {
  font-size: 12px;
}
.mm-hint {
  margin: -6px 0 10px;
  font-size: 11px;
  color: var(--accent, #6b8cff);
}
.mm-footer {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
}
.mm-btn {
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 12px;
}
.mm-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.mm-btn--primary {
  background: var(--accent, #6b8cff);
  border-color: transparent;
  color: #fff;
}
.mm-btn--ghost {
  background: transparent;
}
.mm-ok {
  color: var(--status-ok, #3a9d5c);
}
.mm-bad {
  color: var(--status-bad, #c44);
}
</style>
