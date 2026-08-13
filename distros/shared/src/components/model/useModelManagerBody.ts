// ! State and actions for the model manager panel body.

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
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

export function useModelManagerBody() {
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
  let settingsLoadGeneration = 0
  let ollamaModelsLoadGeneration = 0
  let cloudModelsLoadGeneration = 0
  let settingsSaveGeneration = 0
  let globalSaveGeneration = 0

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
    return (
      selectableModelFiles.value.find(file => file.path === path) ?? null
    )
  })

  const selectedBaseChanged = computed(() => {
    const selectedPath = selectedLocalModelFile.value?.path.trim() ?? ''
    const configuredPath = settings.value?.localModelPath?.trim() ?? ''
    return (
      selectedPath.toLocaleLowerCase() !== configuredPath.toLocaleLowerCase()
    )
  })

  const baseSwitchWillDeactivateLora = computed(
    () =>
      selectedBaseChanged.value
      && Boolean(settings.value?.activeLocalLoraAdapterId),
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

  const selectedLocalIsFile = computed(() =>
    selectedLocalModel.value.startsWith('file:'),
  )

  const cloudModelOptions = computed(() =>
    mergeCloudModelOptions(
      cloudModels.value,
      cloudModelHistory.value,
      remoteModel.value,
    ),
  )

  function canListCloudModels(): boolean {
    if (!remoteUrl.value.trim())
      return false
    const tokenInput = remoteToken.value.trim()
    return (
      tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
    )
  }

  const effectiveModel = computed(() =>
    settings.value?.localRuntimeMode === 'performance'
    && settings.value.localModelPath?.trim()
      ? settings.value.localModelPath.trim()
      : settings.value?.effectiveModel?.trim()
        || roleStore.roleInfo.effectiveOllamaModel?.trim()
        || '',
  )

  async function saveGlobalDefaultModel(): Promise<void> {
    const model = globalDefaultModel.value.trim()
    if (!model) {
      showToast('error', t('modelManager.globalDefaultModelNeedModel'))
      return
    }
    const generation = ++globalSaveGeneration
    savingGlobal.value = true
    try {
      const g = await setGlobalOllamaModel(model, roleStore.currentRoleId)
      if (generation !== globalSaveGeneration)
        return
      globalDefaultModel.value = g.model
      await roleStore.refreshRoleInfo()
      if (generation !== globalSaveGeneration)
        return
      showToast('success', t('modelManager.globalDefaultModelSaveOk'))
    }
    catch (e) {
      if (generation === globalSaveGeneration)
        showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      if (generation === globalSaveGeneration)
        savingGlobal.value = false
    }
  }

  async function loadSettings(): Promise<void> {
    const generation = ++settingsLoadGeneration
    const roleId = roleStore.currentRoleId
    // A role switch owns all loading indicators from this point onward. Older
    // provider-list requests may still settle, but their generations cannot
    // overwrite the new role's form or leave its spinners active.
    ollamaModelsLoadGeneration += 1
    cloudModelsLoadGeneration += 1
    modelsLoading.value = false
    cloudModelsLoading.value = false
    loading.value = true
    void getGlobalOllamaModel()
      .then((global) => {
        if (
          generation === settingsLoadGeneration
          && roleStore.currentRoleId === roleId
        ) {
          globalDefaultModel.value = global.model?.trim() || ''
        }
      })
      .catch((error) => {
        if (
          generation === settingsLoadGeneration
          && roleStore.currentRoleId === roleId
        ) {
          showToast(
            'error',
            error instanceof Error ? error.message : String(error),
          )
        }
      })
    try {
      const s = await getLlmUserSettings(roleId)
      if (
        generation !== settingsLoadGeneration
        || roleStore.currentRoleId !== roleId
      ) {
        return
      }
      settings.value = s
      providerTab.value = s.provider === 'cloud' ? 'cloud' : 'local'
      ollamaBaseUrl.value = s.ollamaBaseUrl
      localModelsDir.value = s.localModelsDir
      folderModelFiles.value = s.localModelFiles ?? []
      remoteUrl.value = s.remoteUrl
      remoteModel.value = s.remoteModel || s.sessionOllamaModel || ''
      remoteToken.value = ''

      selectedLocalModel.value = resolveLocalModelSelection(s)

      if (providerTab.value === 'local')
        await refreshOllamaModels()
      else if (canListCloudModels())
        await refreshCloudModels({ silent: true })
    }
    catch (e) {
      if (
        generation === settingsLoadGeneration
        && roleStore.currentRoleId === roleId
      ) {
        showToast('error', e instanceof Error ? e.message : String(e))
      }
    }
    finally {
      if (
        generation === settingsLoadGeneration
        && roleStore.currentRoleId === roleId
      ) {
        loading.value = false
      }
    }
  }

  async function refreshCloudModels(opts?: {
    silent?: boolean
  }): Promise<void> {
    const generation = ++cloudModelsLoadGeneration
    cloudModelsLoading.value = false
    const requestUrl = remoteUrl.value.trim()
    const tokenInput = remoteToken.value.trim()
    const tokenAlreadyConfigured = Boolean(
      settings.value?.remoteTokenConfigured,
    )
    if (!requestUrl) {
      if (!opts?.silent)
        showToast('error', t('modelManager.needRemoteUrl'))
      return
    }
    const hasKey = tokenInput.length > 0 || tokenAlreadyConfigured
    if (!hasKey) {
      if (!opts?.silent)
        showToast('error', t('modelManager.needApiKey'))
      return
    }
    cloudModelsLoading.value = true
    try {
      const req: { remoteUrl: string, remoteToken?: string } = {
        remoteUrl: requestUrl,
      }
      if (tokenInput.length > 0)
        req.remoteToken = tokenInput
      const models = await listCloudModels(req)
      if (
        generation !== cloudModelsLoadGeneration
        || remoteUrl.value.trim() !== requestUrl
        || remoteToken.value.trim() !== tokenInput
        || Boolean(settings.value?.remoteTokenConfigured)
        !== tokenAlreadyConfigured
      ) {
        return
      }
      cloudModels.value = models
      if (!opts?.silent) {
        showToast(
          'success',
          t('modelManager.cloudModelsOk', { count: cloudModels.value.length }),
        )
      }
    }
    catch (e) {
      if (generation === cloudModelsLoadGeneration && !opts?.silent)
        showToast('error', e instanceof Error ? e.message : String(e))
    }
    finally {
      if (generation === cloudModelsLoadGeneration)
        cloudModelsLoading.value = false
    }
  }

  async function refreshOllamaModels(): Promise<void> {
    const generation = ++ollamaModelsLoadGeneration
    const requestBaseUrl = ollamaBaseUrl.value
    modelsLoading.value = true
    try {
      const models = await listOllamaModels(requestBaseUrl)
      if (
        generation !== ollamaModelsLoadGeneration
        || ollamaBaseUrl.value !== requestBaseUrl
      ) {
        return
      }
      ollamaModels.value = models
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
      if (
        generation === ollamaModelsLoadGeneration
        && settings.value?.localRuntimeMode !== 'performance'
      ) {
        showToast('error', e instanceof Error ? e.message : String(e))
      }
    }
    finally {
      if (generation === ollamaModelsLoadGeneration)
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
      filters: [
        {
          name: 'llama.cpp LoRA',
          extensions: ['gguf', 'ocadapter'],
        },
      ],
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
      showToast(
        'success',
        t('modelManager.loraImportOk', { name: adapter.name }),
      )
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
          kind: 'warning',
        },
      )
      if (!adultAcknowledged)
        return
    }
    loraMutating.value = true
    try {
      await activateLocalLoraAdapter(
        adapter.active ? null : adapter.id,
        adultAcknowledged,
      )
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
    if (
      !(await confirmDialog(
        t('modelManager.loraDeleteConfirm', { name: adapter.name }),
        {
          title: t('modelManager.loraTitle'),
          kind: 'warning',
        },
      ))
    ) {
      return
    }
    loraMutating.value = true
    try {
      await deleteLocalLoraAdapter(adapter.id)
      showToast(
        'success',
        t('modelManager.loraDeleteOk', { name: adapter.name }),
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

  async function resolveLocalRuntimeSelectionForSave(): Promise<{
    localModelPath: string
    ollamaModel: string
  }> {
    const selected = selectedLocalModel.value.trim()
    if (
      selected.startsWith('file:')
      && settings.value?.localRuntimeMode === 'performance'
    ) {
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
    const hasKey
      = tokenInput.length > 0 || Boolean(settings.value?.remoteTokenConfigured)
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
    const generation = ++settingsSaveGeneration
    const roleId = roleStore.currentRoleId
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
            t('modelManager.baseAdultConfirm', {
              name: selectedLocalModelFile.value.name,
            }),
            {
              title: t('modelManager.baseAdultTitle'),
              kind: 'warning',
            },
          )
          if (!adultContentAcknowledged)
            return
        }
        const deactivatedLora = baseSwitchWillDeactivateLora.value
        const info = await saveLlmUserSettings({
          roleId,
          provider: 'local',
          ollamaBaseUrl: ollamaBaseUrl.value.trim(),
          localModelsDir: localModelsDir.value.trim(),
          localModelPath: local.localModelPath,
          adultContentAcknowledged,
          ollamaModel: local.ollamaModel,
          cloudApiStyle: 'openai',
        })
        if (
          generation !== settingsSaveGeneration
          || roleStore.currentRoleId !== roleId
        ) {
          return
        }
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
        const hasKey
          = tokenInput.length > 0
            || Boolean(settings.value?.remoteTokenConfigured)
        if (!hasKey) {
          showToast('error', t('modelManager.needApiKey'))
          return
        }
        const req: Parameters<typeof saveLlmUserSettings>[0] = {
          roleId,
          provider: 'cloud',
          cloudApiStyle: 'openai',
          remoteUrl: remoteUrl.value.trim(),
          remoteModel: remoteModel.value.trim(),
        }
        if (tokenInput.length > 0) {
          req.remoteToken = tokenInput
        }
        const info = await saveLlmUserSettings(req)
        if (
          generation !== settingsSaveGeneration
          || roleStore.currentRoleId !== roleId
        ) {
          return
        }
        roleStore.applyRoleInfo(info)
        rememberCloudModel(remoteModel.value.trim())
        cloudModelHistory.value = getCloudModelHistory()
        showToast('success', t('modelManager.saveOk'))
        await runCloudProbeAfterSave()
      }
      await loadSettings()
    }
    catch (e) {
      if (
        generation === settingsSaveGeneration
        && roleStore.currentRoleId === roleId
      ) {
        showToast('error', e instanceof Error ? e.message : String(e))
      }
    }
    finally {
      if (generation === settingsSaveGeneration)
        saving.value = false
    }
  }

  watch(providerTab, (tab) => {
    if (loading.value)
      return
    if (tab === 'local' && ollamaModels.value.length === 0) {
      void refreshOllamaModels()
    }
    if (
      tab === 'cloud'
      && cloudModels.value.length === 0
      && canListCloudModels()
    ) {
      void refreshCloudModels({ silent: true })
    }
  })

  watch(
    () => roleStore.currentRoleId,
    () => void loadSettings(),
    { immediate: true },
  )

  onBeforeUnmount(() => {
    settingsLoadGeneration += 1
    ollamaModelsLoadGeneration += 1
    cloudModelsLoadGeneration += 1
    settingsSaveGeneration += 1
    globalSaveGeneration += 1
  })

  return {
    baseSwitchWillDeactivateLora,
    canListCloudModels,
    cloudModelHistory,
    cloudModelOptions,
    cloudModels,
    cloudModelsLoadGeneration,
    cloudModelsLoading,
    effectiveModel,
    folderModelFiles,
    formatAdapterSize,
    globalDefaultModel,
    globalSaveGeneration,
    importSelectedFileToOllama,
    importing,
    isUsableOllamaModelId,
    loadSettings,
    loading,
    localModelOptionLabel,
    localModelSelectOptions,
    localModelsDir,
    loraContentRating,
    loraImporting,
    loraMutating,
    loraReplaceExisting,
    modelsLoading,
    ollamaBaseUrl,
    ollamaModels,
    ollamaModelsLoadGeneration,
    onProbeCloud,
    onSave,
    openModelsFolder,
    pickAndImportLora,
    pickModelsFolder,
    probing,
    providerTab,
    refreshCloudModels,
    refreshOllamaModels,
    remoteModel,
    remoteToken,
    remoteUrl,
    removeLora,
    resolveLocalModelForSave,
    resolveLocalModelSelection,
    resolveLocalRuntimeSelectionForSave,
    roleStore,
    runCloudProbeAfterSave,
    saveGlobalDefaultModel,
    saving,
    savingGlobal,
    scanCurrentFolder,
    selectableModelFiles,
    selectedBaseChanged,
    selectedLocalIsFile,
    selectedLocalModel,
    selectedLocalModelFile,
    settings,
    settingsLoadGeneration,
    settingsSaveGeneration,
    showToast,
    t,
    toggleLora,
  }
}
