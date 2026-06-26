<script setup lang="ts">
import type { LlmUserSettings, LocalModelFile } from '@oclive/shared/api/llmSettings'
import { open as openDialog } from '@tauri-apps/api/dialog'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getGlobalOllamaModel,
  getLlmUserSettings,
  importGgufToOllama,
  listCloudModels,
  listOllamaModels,

  openPathInFileManager,
  probeCloudLlm,
  saveLlmUserSettings,
  scanLocalModelFiles,
  setGlobalOllamaModel,
} from '@oclive/shared/api/llmSettings'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import {
  getCloudModelHistory,
  mergeCloudModelOptions,
  rememberCloudModel,
} from '@oclive/shared/composables/useCloudModelHistory'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import UiButton from '../ui/UiButton.vue'

const emit = defineEmits<{
  openSettings: []
}>()

const roleStore = useRoleStore()
const { t } = useI18n()
const { showToast } = useAppToast()

const loading = ref(false)
const saving = ref(false)
const probing = ref(false)
const cloudModelsLoading = ref(false)
const modelsLoading = ref(false)
const importing = ref(false)
const savingGlobal = ref(false)
const globalDefaultModel = ref('')
const settings = ref<LlmUserSettings | null>(null)
const ollamaModels = ref<string[]>([])
const folderModelFiles = ref<LocalModelFile[]>([])

const providerTab = ref<'local' | 'cloud'>('local')
const ollamaBaseUrl = ref('')
const localModelsDir = ref('')
const selectedLocalModel = ref('')
const remoteUrl = ref('')
const remoteToken = ref('')
const remoteModel = ref('')
const cloudModels = ref<string[]>([])
const cloudModelHistory = ref<string[]>(getCloudModelHistory())

function isUsableOllamaModelId(model: string | null | undefined): boolean {
  const t = model?.trim() ?? ''
  if (!t || t.startsWith('file:'))
    return false
  if (t.includes('\\'))
    return false
  if (/^[a-zA-Z]:/.test(t))
    return false
  if (t.startsWith('/') || t.startsWith('\\\\'))
    return false
  return true
}

function resolveLocalModelSelection(s: LlmUserSettings): string {
  const session = s.sessionOllamaModel?.trim()
  if (session && isUsableOllamaModelId(session)) {
    if (folderModelFiles.value.some(f => f.path === session))
      return `file:${session}`
    return session
  }
  const global = globalDefaultModel.value.trim()
  if (global && isUsableOllamaModelId(global))
    return global
  const effective = s.effectiveModel?.trim()
  if (effective && isUsableOllamaModelId(effective))
    return effective
  const pack = s.packOllamaModel?.trim()
  if (pack && isUsableOllamaModelId(pack))
    return pack
  return ''
}

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

const cloudModelOptions = computed(() =>
  mergeCloudModelOptions(cloudModels.value, cloudModelHistory.value, remoteModel.value),
)

function canListCloudModels(): boolean {
  if (!remoteUrl.value.trim())
    return false
  const tokenInput = remoteToken.value.trim()
  return tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
}

const effectiveModel = computed(
  () => settings.value?.effectiveModel?.trim() || roleStore.roleInfo.effectiveOllamaModel?.trim() || '',
)

async function loadGlobalDefaultModel(): Promise<void> {
  try {
    const g = await getGlobalOllamaModel()
    globalDefaultModel.value = g.model?.trim() || ''
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

async function saveGlobalDefaultModel(): Promise<void> {
  const model = globalDefaultModel.value.trim()
  if (!model) {
    showToast('error', t('modelManager.globalDefaultModelNeedModel'))
    return
  }
  savingGlobal.value = true
  try {
    const g = await setGlobalOllamaModel(model, roleStore.currentRoleId)
    globalDefaultModel.value = g.model
    await roleStore.loadRoleInfo()
    showToast('success', t('modelManager.globalDefaultModelSaveOk'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    savingGlobal.value = false
  }
}

async function loadSettings(): Promise<void> {
  loading.value = true
  try {
    await loadGlobalDefaultModel()
    const s = await getLlmUserSettings(roleStore.currentRoleId)
    settings.value = s
    providerTab.value = s.provider === 'cloud' ? 'cloud' : 'local'
    ollamaBaseUrl.value = s.ollamaBaseUrl
    localModelsDir.value = s.localModelsDir
    folderModelFiles.value = s.localModelFiles ?? []
    remoteUrl.value = s.remoteUrl
    remoteModel.value = s.remoteModel || s.sessionOllamaModel || ''
    remoteToken.value = ''

    selectedLocalModel.value = resolveLocalModelSelection(s)

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

async function refreshCloudModels(opts?: { silent?: boolean }): Promise<void> {
  if (!remoteUrl.value.trim()) {
    if (!opts?.silent)
      showToast('error', t('modelManager.needRemoteUrl'))
    return
  }
  const tokenInput = remoteToken.value.trim()
  const hasKey = tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
  if (!hasKey) {
    if (!opts?.silent)
      showToast('error', t('modelManager.needApiKey'))
    return
  }
  cloudModelsLoading.value = true
  try {
    const req: { remoteUrl: string, remoteToken?: string } = {
      remoteUrl: remoteUrl.value.trim(),
    }
    if (tokenInput.length > 0)
      req.remoteToken = tokenInput
    cloudModels.value = await listCloudModels(req)
    if (!opts?.silent)
      showToast('success', t('modelManager.cloudModelsOk', { count: cloudModels.value.length }))
  }
  catch (e) {
    if (!opts?.silent)
      showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    cloudModelsLoading.value = false
  }
}

async function refreshOllamaModels(): Promise<void> {
  modelsLoading.value = true
  try {
    ollamaModels.value = await listOllamaModels(ollamaBaseUrl.value)
    const cur = selectedLocalModel.value
    if (
      cur
      && !cur.startsWith('file:')
      && isUsableOllamaModelId(cur)
      && !ollamaModels.value.includes(cur)
    ) {
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
    const first = folderModelFiles.value[0]
    if (first)
      selectedLocalModel.value = `file:${first.path}`
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

async function runCloudProbeAfterSave(): Promise<void> {
  try {
    await probeCloudLlm(roleStore.currentRoleId)
    showToast('info', t('modelManager.probeOk'))
  }
  catch (e) {
    showToast('warning', e instanceof Error ? e.message : String(e))
  }
}

async function onProbeCloud(): Promise<void> {
  if (!remoteUrl.value.trim()) {
    showToast('error', t('modelManager.needRemoteUrl'))
    return
  }
  if (!remoteModel.value.trim()) {
    showToast('error', t('modelManager.needRemoteModel'))
    return
  }
  const tokenInput = remoteToken.value.trim()
  const hasKey = tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
  if (!hasKey) {
    showToast('error', t('modelManager.needApiKey'))
    return
  }
  probing.value = true
  try {
    await probeCloudLlm(roleStore.currentRoleId)
    showToast('success', t('modelManager.probeOk'))
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    probing.value = false
  }
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
      showToast('success', t('modelManager.saveOk'))
    }
    else {
      if (!remoteUrl.value.trim()) {
        showToast('error', t('modelManager.needRemoteUrl'))
        return
      }
      if (!remoteModel.value.trim()) {
        showToast('error', t('modelManager.needRemoteModel'))
        return
      }
      const tokenInput = remoteToken.value.trim()
      const hasKey = tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
      if (!hasKey) {
        showToast('error', t('modelManager.needApiKey'))
        return
      }
      const req: Parameters<typeof saveLlmUserSettings>[0] = {
        roleId: roleStore.currentRoleId,
        provider: 'cloud',
        cloudApiStyle: 'openai',
        remoteUrl: remoteUrl.value.trim(),
        remoteModel: remoteModel.value.trim(),
      }
      if (tokenInput.length > 0) {
        req.remoteToken = tokenInput
      }
      const info = await saveLlmUserSettings(req)
      roleStore.applyRoleInfo(info)
      rememberCloudModel(remoteModel.value.trim())
      cloudModelHistory.value = getCloudModelHistory()
      showToast('success', t('modelManager.saveOk'))
      await runCloudProbeAfterSave()
    }
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
  if (tab === 'cloud' && cloudModels.value.length === 0 && canListCloudModels()) {
    void refreshCloudModels({ silent: true })
  }
})

watch(
  () => roleStore.currentRoleId,
  () => {
    void loadSettings()
    void refreshOllamaModels()
  },
  { immediate: true },
)
</script>

<template>
  <div class="mm-root tool-mgmt-panel">
    <p class="mm-lead">
      {{ t("modelManager.lead") }}
    </p>

    <p v-if="loading" class="mm-muted">
      {{ t("modelManager.loading") }}
    </p>

    <template v-else>
      <section class="mm-panel mm-global-default">
        <h3 class="mm-h3">
          {{ t("modelManager.globalDefaultModelLabel") }}
        </h3>
        <p class="mm-muted mm-small">
          {{ t("modelManager.globalDefaultModelLead") }}
        </p>
        <label class="mm-field">
          <span>{{ t("modelManager.globalDefaultModelLabel") }}</span>
          <select
            v-model="globalDefaultModel"
            class="mm-select"
            :disabled="modelsLoading || savingGlobal"
          >
            <option v-if="!globalDefaultModel && ollamaModels.length === 0" value="">
              {{ t("modelManager.noLocalModels") }}
            </option>
            <option v-for="m in ollamaModels" :key="`global-${m}`" :value="m">
              {{ m }}
            </option>
            <option
              v-if="globalDefaultModel && !ollamaModels.includes(globalDefaultModel)"
              :value="globalDefaultModel"
            >
              {{ globalDefaultModel }}
            </option>
          </select>
        </label>
        <div class="mm-row-actions">
          <button
            type="button"
            class="mm-btn mm-btn-primary"
            :disabled="savingGlobal || !globalDefaultModel.trim()"
            @click="saveGlobalDefaultModel"
          >
            {{ savingGlobal ? t("modelManager.globalDefaultModelSaving") : t("modelManager.globalDefaultModelSave") }}
          </button>
          <button
            type="button"
            class="mm-btn"
            :disabled="modelsLoading || savingGlobal"
            @click="refreshOllamaModels"
          >
            {{ modelsLoading ? t("modelManager.refreshingModels") : t("modelManager.refreshModels") }}
          </button>
        </div>
      </section>

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

        <div class="mm-help" role="note">
          <p class="mm-help-title">
            {{ t("modelManager.cloudFieldsTitle") }}
          </p>
          <ul class="mm-help-list">
            <li>{{ t("modelManager.cloudFieldUrl") }}</li>
            <li>{{ t("modelManager.cloudFieldKey") }}</li>
            <li>{{ t("modelManager.cloudFieldModel") }}</li>
          </ul>
        </div>

        <p class="mm-hint mm-hint-note">
          {{ t("modelManager.cloudEnvCheckNote") }}
        </p>

        <label class="mm-field">
          <span>{{ t("modelManager.remoteUrlLabel") }}</span>
          <input
            v-model="remoteUrl"
            type="url"
            class="mm-input"
            :placeholder="t('modelManager.remoteUrlPlaceholder')"
          >
          <span class="mm-field-hint">{{ t("modelManager.remoteUrlHint") }}</span>
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
          >
          <span class="mm-field-hint">{{ t("modelManager.remoteTokenHint") }}</span>
        </label>
        <p v-if="settings?.remoteTokenEnvActive" class="mm-hint">
          {{ t("modelManager.envTokenNote") }}
        </p>

        <label class="mm-field">
          <span>{{ t("modelManager.remoteModelLabel") }}</span>
          <input
            v-model="remoteModel"
            list="mm-cloud-model-list"
            type="text"
            class="mm-input"
            :placeholder="t('modelManager.remoteModelPlaceholder')"
          >
          <datalist id="mm-cloud-model-list">
            <option v-for="m in cloudModelOptions" :key="m" :value="m" />
          </datalist>
          <span class="mm-field-hint">{{ t("modelManager.remoteModelHint") }}</span>
        </label>

        <div class="mm-row-actions">
          <button
            type="button"
            class="mm-btn"
            :disabled="cloudModelsLoading || saving || !canListCloudModels()"
            @click="refreshCloudModels()"
          >
            {{
              cloudModelsLoading
                ? t("modelManager.refreshingCloudModels")
                : t("modelManager.refreshCloudModels")
            }}
          </button>
          <span v-if="cloudModelHistory.length" class="mm-muted mm-small">
            {{ t("modelManager.cloudModelHistoryHint", { count: cloudModelHistory.length }) }}
          </span>
        </div>
      </section>

      <footer class="mm-footer">
        <UiButton
          size="sm"
          variant="primary"
          :disabled="saving || loading"
          @click="onSave"
        >
          {{ saving ? t("modelManager.saving") : t("modelManager.saveApply") }}
        </UiButton>
        <UiButton
          v-if="providerTab === 'cloud'"
          size="sm"
          variant="ghost"
          :disabled="probing || saving || loading"
          @click="onProbeCloud"
        >
          {{ probing ? t("modelManager.probing") : t("modelManager.probeCloud") }}
        </UiButton>
        <UiButton size="sm" variant="ghost" @click="emit('openSettings')">
          {{ t("modelManager.openSettings") }}
        </UiButton>
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
.mm-help {
  margin-bottom: 14px;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--panel-bg-soft, var(--bg-elevated));
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.mm-help-title {
  margin: 0 0 6px;
  font-weight: 600;
  color: var(--text-primary);
}
.mm-help-list {
  margin: 0;
  padding-left: 1.2em;
}
.mm-help-list li {
  margin-bottom: 4px;
}
.mm-help-list li:last-child {
  margin-bottom: 0;
}
.mm-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}
.mm-field-hint {
  font-size: 11px;
  line-height: 1.4;
  color: var(--text-secondary);
  opacity: 0.9;
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
