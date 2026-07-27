<script setup lang="ts">
import type {
  LlmUserSettings,
  LocalLoraAdapter,
  LocalModelFile,
  LoraContentRating,
} from '@oclive/shared/api/llmSettings'
import {
  activateLocalLoraAdapter,
  deleteLocalLoraAdapter,
  getGlobalOllamaModel,
  getLlmUserSettings,
  importGgufToOllama,
  importLocalLoraAdapter,
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
import {
  confirm as confirmDialog,
  open as openDialog,
} from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import HelpHint from '../shared/HelpHint.vue'
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
const loraMutating = ref(false)
const loraImporting = ref(false)
const loraContentRating = ref<LoraContentRating>('general')
const loraReplaceExisting = ref(false)
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

function formatAdapterSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0)
    return '0 B'
  const units = ['B', 'KiB', 'MiB', 'GiB']
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  return `${value >= 10 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`
}

function isUsableOllamaModelId(model: string | null | undefined): boolean {
  const t = model?.trim() ?? ''
  if (!t || t.startsWith('file:'))
    return false
  if (t.includes('\\'))
    return false
  if (/^[a-z]:/i.test(t))
    return false
  if (t.startsWith('/') || t.startsWith('\\\\'))
    return false
  return true
}

function resolveLocalModelSelection(s: LlmUserSettings): string {
  const localPath = s.localModelPath?.trim()
  if (localPath)
    return `file:${localPath}`
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

const selectableModelFiles = computed<LocalModelFile[]>(() => {
  const files = [...folderModelFiles.value]
  const configured = settings.value?.localModelPath?.trim()
  if (
    settings.value?.localRuntimeMode === 'performance'
    && configured
    && !files.some(file => file.path === configured)
  ) {
    files.unshift({
      path: configured,
      name: configured,
      sizeBytes: 0,
      contentRating: 'general',
      description: null,
      license: null,
      source: null,
      sha256: null,
    })
  }
  return files
})

const selectedLocalModelFile = computed<LocalModelFile | null>(() => {
  const selected = selectedLocalModel.value.trim()
  if (!selected.startsWith('file:'))
    return null
  const path = selected.slice('file:'.length)
  return selectableModelFiles.value.find(file => file.path === path) ?? null
})

const selectedBaseChanged = computed(() => {
  const selectedPath = selectedLocalModelFile.value?.path.trim() ?? ''
  const configuredPath = settings.value?.localModelPath?.trim() ?? ''
  return selectedPath.toLocaleLowerCase() !== configuredPath.toLocaleLowerCase()
})

const baseSwitchWillDeactivateLora = computed(
  () => selectedBaseChanged.value && Boolean(settings.value?.activeLocalLoraAdapterId),
)

function localModelOptionLabel(model: LocalModelFile): string {
  return model.contentRating === 'adult'
    ? `${model.name} · ${t('modelManager.baseRatingAdult')}`
    : model.name
}

const localModelSelectOptions = computed(() => {
  const ollama = ollamaModels.value.map(id => ({
    value: id,
    label: id,
    group: 'ollama' as const,
  }))
  const files = selectableModelFiles.value.map(f => ({
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
  () => (
    settings.value?.localRuntimeMode === 'performance'
    && settings.value.localModelPath?.trim()
      ? settings.value.localModelPath.trim()
      : settings.value?.effectiveModel?.trim()
        || roleStore.roleInfo.effectiveOllamaModel?.trim()
        || ''
  ),
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
    if (settings.value?.localRuntimeMode !== 'performance')
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

async function pickAndImportLora(): Promise<void> {
  if (settings.value?.localRuntimeMode !== 'performance') {
    showToast('error', t('modelManager.loraNeedsPerformance'))
    return
  }
  const picked = await openDialog({
    directory: false,
    multiple: false,
    filters: [{
      name: 'llama.cpp LoRA',
      extensions: ['gguf', 'ocadapter'],
    }],
  })
  if (!picked || Array.isArray(picked))
    return

  loraImporting.value = true
  try {
    const adapter = await importLocalLoraAdapter({
      sourcePath: picked,
      baseModel: settings.value.localModelPath || undefined,
      contentRating: loraContentRating.value,
      replaceExisting: loraReplaceExisting.value,
    })
    showToast('success', t('modelManager.loraImportOk', { name: adapter.name }))
    await loadSettings()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loraImporting.value = false
  }
}

async function toggleLora(adapter: LocalLoraAdapter): Promise<void> {
  if (!adapter.active && !settings.value?.localModelPath?.trim()) {
    showToast('error', t('modelManager.loraSaveBaseFirst'))
    return
  }
  let adultAcknowledged = false
  if (!adapter.active && adapter.contentRating === 'adult') {
    adultAcknowledged = await confirmDialog(
      t('modelManager.loraAdultConfirm', { name: adapter.name }),
      {
        title: t('modelManager.loraTitle'),
        type: 'warning',
      },
    )
    if (!adultAcknowledged)
      return
  }
  loraMutating.value = true
  try {
    await activateLocalLoraAdapter(adapter.active ? null : adapter.id, adultAcknowledged)
    showToast(
      'success',
      adapter.active
        ? t('modelManager.loraDeactivateOk')
        : t('modelManager.loraActivateOk', { name: adapter.name }),
    )
    await loadSettings()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loraMutating.value = false
  }
}

async function removeLora(adapter: LocalLoraAdapter): Promise<void> {
  if (adapter.active)
    return
  if (!await confirmDialog(
    t('modelManager.loraDeleteConfirm', { name: adapter.name }),
    {
      title: t('modelManager.loraTitle'),
      type: 'warning',
    },
  )) {
    return
  }
  loraMutating.value = true
  try {
    await deleteLocalLoraAdapter(adapter.id)
    showToast('success', t('modelManager.loraDeleteOk', { name: adapter.name }))
    await loadSettings()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loraMutating.value = false
  }
}

async function resolveLocalRuntimeSelectionForSave(): Promise<{
  localModelPath: string
  ollamaModel: string
}> {
  const selected = selectedLocalModel.value.trim()
  if (selected.startsWith('file:') && settings.value?.localRuntimeMode === 'performance') {
    return {
      localModelPath: selected.slice('file:'.length),
      ollamaModel: globalDefaultModel.value.trim(),
    }
  }
  return {
    localModelPath: '',
    ollamaModel: await resolveLocalModelForSave(),
  }
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
      const local = await resolveLocalRuntimeSelectionForSave()
      let adultContentAcknowledged = false
      if (
        selectedBaseChanged.value
        && selectedLocalModelFile.value?.contentRating === 'adult'
      ) {
        adultContentAcknowledged = await confirmDialog(
          t('modelManager.baseAdultConfirm', { name: selectedLocalModelFile.value.name }),
          {
            title: t('modelManager.baseAdultTitle'),
            type: 'warning',
          },
        )
        if (!adultContentAcknowledged)
          return
      }
      const deactivatedLora = baseSwitchWillDeactivateLora.value
      const info = await saveLlmUserSettings({
        roleId: roleStore.currentRoleId,
        provider: 'local',
        ollamaBaseUrl: ollamaBaseUrl.value.trim(),
        localModelsDir: localModelsDir.value.trim(),
        localModelPath: local.localModelPath,
        adultContentAcknowledged,
        ollamaModel: local.ollamaModel,
        cloudApiStyle: 'openai',
      })
      roleStore.applyRoleInfo(info)
      showToast('success', t('modelManager.saveOk'))
      if (deactivatedLora)
        showToast('warning', t('modelManager.baseSwitchLoraDeactivated'))
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
          {{ settings?.localRuntimeMode === 'performance'
            ? t("modelManager.ollamaFallbackModelLabel")
            : t("modelManager.globalDefaultModelLabel") }}
        </h3>
        <p class="mm-muted mm-small">
          {{ settings?.localRuntimeMode === 'performance'
            ? t("modelManager.ollamaFallbackModelLead")
            : t("modelManager.globalDefaultModelLead") }}
        </p>
        <label class="mm-field">
          <span>
            {{ settings?.localRuntimeMode === 'performance'
              ? t("modelManager.ollamaFallbackModelLabel")
              : t("modelManager.globalDefaultModelLabel") }}
          </span>
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
          {{ settings?.localRuntimeMode === 'performance'
            ? t("modelManager.performanceTitle")
            : t("modelManager.localTitle") }}
        </h3>
        <p class="mm-muted">
          {{ settings?.localRuntimeMode === 'performance'
            ? t("modelManager.performanceLead")
            : t("modelManager.localLead") }}
        </p>

        <div
          v-if="settings?.localRuntimeMode === 'performance'"
          class="mm-effective"
          role="status"
        >
          <span class="mm-effective-label">{{ t("modelManager.performanceStatusLabel") }}</span>
          <span :class="settings.performanceReady ? 'mm-ok' : 'mm-muted'">
            {{ settings.performanceReady
              ? t("modelManager.performanceReady")
              : t("modelManager.performanceFallbackActive") }}
          </span>
          <code class="mm-mono">{{ settings.performanceEndpoint }}</code>
          <span class="mm-muted mm-small">{{ settings.performanceDetail }}</span>
        </div>

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
                : settings.localRuntimeMode === 'performance'
                  ? t("modelManager.ollamaFallbackUnavailable")
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
            <optgroup
              v-if="selectableModelFiles.length"
              :label="settings?.localRuntimeMode === 'performance'
                ? t('modelManager.performanceModelsLabel')
                : t('modelManager.folderModelsLabel')"
            >
              <option
                v-for="f in selectableModelFiles"
                :key="f.path"
                :value="`file:${f.path}`"
              >
                {{ localModelOptionLabel(f) }}
              </option>
            </optgroup>
            <optgroup
              v-if="ollamaModels.length"
              :label="settings?.localRuntimeMode === 'performance'
                ? t('modelManager.ollamaFallbackGroup')
                : 'Ollama'"
            >
              <option v-for="m in ollamaModels" :key="m" :value="m">
                {{ m }}
              </option>
            </optgroup>
          </select>
        </label>

        <div
          v-if="selectedLocalModelFile"
          class="mm-base-card"
          :class="{ 'is-adult': selectedLocalModelFile.contentRating === 'adult' }"
          role="note"
        >
          <div class="mm-base-card-heading">
            <strong>{{ selectedLocalModelFile.name }}</strong>
            <span
              class="mm-lora-badge"
              :class="{ 'is-adult': selectedLocalModelFile.contentRating === 'adult' }"
            >
              {{
                selectedLocalModelFile.contentRating === 'adult'
                  ? t("modelManager.baseRatingAdult")
                  : t("modelManager.baseRatingGeneral")
              }}
            </span>
          </div>
          <p v-if="selectedLocalModelFile.description" class="mm-field-hint">
            {{ selectedLocalModelFile.description }}
          </p>
          <div class="mm-lora-meta">
            <span>{{ formatAdapterSize(selectedLocalModelFile.sizeBytes) }}</span>
            <span v-if="selectedLocalModelFile.license">
              {{ t("modelManager.baseLicense", { license: selectedLocalModelFile.license }) }}
            </span>
          </div>
          <p class="mm-base-combination-note">
            {{ t("modelManager.baseCombinationNotice") }}
          </p>
          <p
            v-if="selectedLocalModelFile.contentRating === 'adult'"
            class="mm-base-adult-note"
          >
            {{ t("modelManager.baseAdultNotice") }}
          </p>
          <p v-if="baseSwitchWillDeactivateLora" class="mm-base-switch-note">
            {{ t("modelManager.baseSwitchLoraNotice") }}
          </p>
        </div>

        <button
          v-if="selectedLocalIsFile && settings?.localRuntimeMode !== 'performance'"
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

        <section
          v-if="settings?.localRuntimeMode === 'performance'"
          class="mm-lora"
          aria-labelledby="mm-lora-title"
        >
          <div class="mm-lora-heading">
            <div>
              <h4 id="mm-lora-title" class="mm-h4">
                {{ t("modelManager.loraTitle") }}
              </h4>
              <p class="mm-muted">
                {{ t("modelManager.loraLead") }}
              </p>
            </div>
            <button
              type="button"
              class="mm-btn"
              :disabled="loading || loraMutating || loraImporting"
              @click="loadSettings"
            >
              {{ t("modelManager.loraRefresh") }}
            </button>
          </div>

          <div class="mm-lora-route" role="note">
            <div class="mm-lora-route-head">
              <strong>{{ t("modelManager.loraRuntimeRouteTitle") }}</strong>
              <HelpHint
                :paragraphs="[
                  t('modelManager.loraRuntimeHint1'),
                  t('modelManager.loraRuntimeHint2'),
                ]"
              />
            </div>
            <div class="mm-lora-route-steps">
              <span class="mm-lora-route-step is-primary">
                {{ t("modelManager.loraRuntimePrimary") }}
              </span>
              <span class="mm-lora-route-arrow" aria-hidden="true">→</span>
              <span class="mm-lora-route-step">
                {{ t("modelManager.loraRuntimeSwitch") }}
              </span>
              <span class="mm-lora-route-arrow" aria-hidden="true">→</span>
              <span class="mm-lora-route-step is-fallback">
                {{ t("modelManager.loraRuntimeFallback") }}
              </span>
            </div>
          </div>

          <div class="mm-lora-import">
            <label class="mm-field mm-lora-rating">
              <span>{{ t("modelManager.loraContentRating") }}</span>
              <select v-model="loraContentRating" class="mm-select">
                <option value="general">
                  {{ t("modelManager.loraRatingGeneral") }}
                </option>
                <option value="adult">
                  {{ t("modelManager.loraRatingAdult") }}
                </option>
              </select>
            </label>
            <label class="mm-check">
              <input v-model="loraReplaceExisting" type="checkbox">
              <span>{{ t("modelManager.loraReplaceExisting") }}</span>
            </label>
            <button
              type="button"
              class="mm-btn mm-btn--primary"
              :disabled="loraImporting || loraMutating || saving"
              @click="pickAndImportLora"
            >
              {{
                loraImporting
                  ? t("modelManager.loraImporting")
                  : t("modelManager.loraImport")
              }}
            </button>
          </div>
          <p class="mm-field-hint">
            {{ t("modelManager.loraImportHint") }}
          </p>

          <p
            v-if="!settings.localLoraAdapters?.length"
            class="mm-muted mm-lora-empty"
          >
            {{ t("modelManager.loraEmpty") }}
          </p>
          <div v-else class="mm-lora-list">
            <article
              v-for="adapter in settings.localLoraAdapters"
              :key="adapter.id"
              class="mm-lora-card"
              :class="{ 'is-active': adapter.active }"
            >
              <div class="mm-lora-card-main">
                <div class="mm-lora-name-row">
                  <strong>{{ adapter.name }}</strong>
                  <span v-if="adapter.active" class="mm-lora-badge is-active">
                    {{ t("modelManager.loraActive") }}
                  </span>
                  <span
                    class="mm-lora-badge"
                    :class="{ 'is-adult': adapter.contentRating === 'adult' }"
                  >
                    {{
                      adapter.contentRating === "adult"
                        ? t("modelManager.loraRatingAdult")
                        : t("modelManager.loraRatingGeneral")
                    }}
                  </span>
                </div>
                <div class="mm-lora-meta">
                  <code>{{ adapter.id }}</code>
                  <span>v{{ adapter.version }}</span>
                  <span>{{ formatAdapterSize(adapter.sizeBytes) }}</span>
                  <span v-if="adapter.architecture">
                    {{ adapter.architecture }}
                  </span>
                </div>
                <p v-if="adapter.baseModel" class="mm-field-hint">
                  {{ t("modelManager.loraBaseModel", { model: adapter.baseModel }) }}
                </p>
                <p v-if="adapter.description" class="mm-field-hint">
                  {{ adapter.description }}
                </p>
              </div>
              <div class="mm-row-actions mm-lora-actions">
                <button
                  type="button"
                  class="mm-btn"
                  :class="{ 'mm-btn--primary': !adapter.active }"
                  :disabled="loraMutating || loraImporting || (!adapter.active && !settings.localModelPath)"
                  @click="toggleLora(adapter)"
                >
                  {{
                    adapter.active
                      ? t("modelManager.loraDeactivate")
                      : t("modelManager.loraActivate")
                  }}
                </button>
                <button
                  type="button"
                  class="mm-btn mm-btn--danger"
                  :disabled="adapter.active || loraMutating || loraImporting"
                  @click="removeLora(adapter)"
                >
                  {{ t("modelManager.loraDelete") }}
                </button>
              </div>
            </article>
          </div>
        </section>
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
.mm-h4 {
  margin: 0 0 4px;
  font-size: 0.88rem;
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
.mm-check {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-size: 12px;
}
.mm-base-card {
  margin: -2px 0 14px;
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: 9px;
  background: var(--panel-bg-soft, var(--bg-elevated));
}
.mm-base-card.is-adult {
  border-color: color-mix(in srgb, #b56a86 52%, var(--border-light));
}
.mm-base-card-heading {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;
  margin-bottom: 4px;
}
.mm-base-card .mm-field-hint,
.mm-base-card .mm-lora-meta {
  margin: 5px 0 0;
}
.mm-base-combination-note,
.mm-base-adult-note,
.mm-base-switch-note {
  margin: 7px 0 0;
  font-size: 11px;
  line-height: 1.45;
}
.mm-base-combination-note {
  color: var(--text-secondary);
}
.mm-base-adult-note {
  color: #b56a86;
}
.mm-base-switch-note {
  color: var(--accent, #6b8cff);
}
.mm-lora {
  margin-top: 18px;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}
.mm-lora-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.mm-lora-route {
  margin: 12px 0 14px;
  padding: 10px;
  border: 1px solid color-mix(in srgb, var(--accent, #6b8cff) 30%, var(--border-light));
  border-radius: 9px;
  background: color-mix(in srgb, var(--accent, #6b8cff) 7%, var(--bg-elevated));
}
.mm-lora-route-head {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  font-size: 12px;
}
.mm-lora-route-steps {
  display: grid;
  grid-template-columns:
    minmax(0, 1fr)
    auto
    minmax(0, 1fr)
    auto
    minmax(0, 1fr);
  align-items: center;
  gap: 6px;
}
.mm-lora-route-step {
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid var(--border-light);
  border-radius: 7px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 11px;
  line-height: 1.45;
  text-align: center;
}
.mm-lora-route-step.is-primary {
  border-color: color-mix(in srgb, var(--status-ok, #3a9d5c) 55%, var(--border-light));
  color: var(--text-primary);
}
.mm-lora-route-step.is-fallback {
  border-style: dashed;
}
.mm-lora-route-arrow {
  color: var(--text-secondary);
  font-size: 12px;
}
.mm-lora-import {
  display: flex;
  flex-wrap: wrap;
  align-items: end;
  gap: 10px;
}
.mm-lora-rating {
  min-width: 150px;
  margin-bottom: 0;
}
.mm-lora-empty {
  margin-top: 14px;
  padding: 12px;
  border: 1px dashed var(--border-light);
  border-radius: 8px;
  text-align: center;
}
.mm-lora-list {
  display: grid;
  gap: 9px;
  margin-top: 14px;
}
.mm-lora-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: 9px;
  background: var(--panel-bg-soft, var(--bg-elevated));
}
.mm-lora-card.is-active {
  border-color: color-mix(in srgb, var(--status-ok, #3a9d5c) 65%, var(--border-light));
}
.mm-lora-card-main {
  min-width: 0;
}
.mm-lora-name-row,
.mm-lora-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;
}
.mm-lora-meta {
  margin-top: 5px;
  color: var(--text-secondary);
  font-size: 11px;
}
.mm-lora-meta code {
  overflow-wrap: anywhere;
}
.mm-lora-badge {
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--text-secondary) 12%, transparent);
  color: var(--text-secondary);
  font-size: 10px;
}
.mm-lora-badge.is-active {
  background: color-mix(in srgb, var(--status-ok, #3a9d5c) 16%, transparent);
  color: var(--status-ok, #3a9d5c);
}
.mm-lora-badge.is-adult {
  background: color-mix(in srgb, #b56a86 16%, transparent);
  color: #b56a86;
}
.mm-lora-actions {
  flex: 0 0 auto;
  margin-bottom: 0;
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
.mm-btn--danger {
  color: var(--status-bad, #c44);
}
@media (max-width: 620px) {
  .mm-lora-card {
    align-items: stretch;
    flex-direction: column;
  }
  .mm-lora-route-steps {
    grid-template-columns: 1fr;
  }
  .mm-lora-route-arrow {
    display: none;
  }
}
.mm-ok {
  color: var(--status-ok, #3a9d5c);
}
.mm-bad {
  color: var(--status-bad, #c44);
}
</style>
