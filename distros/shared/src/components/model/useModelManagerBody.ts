// ! State and actions for the model manager panel body.

import type {
  LlmUserSettings,
  LocalLoraAdapter,
  LocalModelFile,
  LoraContentRating,
} from '@oclive/shared/api/llmSettings'
import { toastAsyncError, toErrorPresentation } from '@oclive/shared/api/helpers'
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

export function useModelManagerBody(options: {
  initialProvider?: 'local' | 'cloud'
  onSaved?: (roleId: string) => void
} = {}) {
  const roleStore = useRoleStore()
  const { t } = useI18n()
  const { showToast } = useAppToast()

  const loading = ref(false)
  const saving = ref(false)
  const probing = ref(false)
  const cloudModelsLoading = ref(false)
  const ollamaListState = ref<'unchecked' | 'checking' | 'ready' | 'error'>('unchecked')
  const modelsLoading = computed(() => ollamaListState.value === 'checking')
  const ollamaListStatusKey = computed(() => ({
    unchecked: 'modelManager.ollamaListUnchecked',
    checking: 'modelManager.ollamaListChecking',
    ready: 'modelManager.ollamaListReady',
    error: 'modelManager.ollamaListFailed',
  })[ollamaListState.value])
  const importing = ref(false)
  const savingGlobal = ref(false)
  const loraMutating = ref(false)
  const loraImporting = ref(false)
  const loraContentRating = ref<LoraContentRating>('general')
  const loraReplaceExisting = ref(false)
  const globalDefaultModel = ref('')
  const settings = ref<LlmUserSettings | null>(null)
  const settingsRoleId = ref<string | null>(null)
  const settingsLoadError = ref<unknown>(null)
  const settingsReady = computed(() => settings.value !== null && settingsRoleId.value === roleStore.currentRoleId)
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
  let cloudProbeGeneration = 0
  let cloudFormGeneration = 0
  let disposed = false
  let initialProvider = options.initialProvider
  let ollamaListRequest: AbortController | null = null
  // The global read can outlive the main settings read; retain their shared owner.
  let settingsReadRequest: AbortController | null = null

  function invalidateOllamaModels(preserveStatus = false): void {
    ollamaListRequest?.abort()
    ollamaListRequest = null
    ollamaModelsLoadGeneration += 1
    if (!preserveStatus || modelsLoading.value)
      ollamaListState.value = 'unchecked'
  }

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

  const cloudProbeNeedsSave = computed(() => {
    const saved = settings.value
    return loading.value || settingsLoadError.value !== null || !saved || settingsRoleId.value !== roleStore.currentRoleId
      || saved.provider !== 'cloud' || providerTab.value !== 'cloud'
      || !remoteUrl.value.trim() || !remoteModel.value.trim() || !saved.remoteTokenConfigured
      || remoteUrl.value.trim() !== saved.remoteUrl.trim()
      || remoteModel.value.trim() !== (saved.remoteModel || saved.sessionOllamaModel || '').trim()
      || remoteToken.value.trim().length > 0
  })

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
        toastAsyncError(e)
    }
    finally {
      if (generation === globalSaveGeneration)
        savingGlobal.value = false
    }
  }

  async function loadSettings({ preserveForm = false }: { preserveForm?: boolean } = {}): Promise<void> {
    if (disposed)
      return
    settingsReadRequest?.abort()
    const readRequest = new AbortController()
    settingsReadRequest = readRequest
    invalidateCloudProbe()
    settingsLoadError.value = null
    const generation = ++settingsLoadGeneration
    const roleId = roleStore.currentRoleId
    // A role switch owns all loading indicators from this point onward. Older
    // provider-list requests may still settle, but their generations cannot
    // overwrite the new role's form or leave its spinners active.
    invalidateOllamaModels(preserveForm)
    cloudModelsLoadGeneration += 1
    cloudModelsLoading.value = false
    loading.value = true
    void getGlobalOllamaModel({ signal: readRequest.signal })
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
          toastAsyncError(error)
        }
      })
    try {
      const s = await getLlmUserSettings(roleId, undefined, { signal: readRequest.signal })
      if (
        generation !== settingsLoadGeneration
        || roleStore.currentRoleId !== roleId
      ) {
        return
      }
      settings.value = s
      settingsRoleId.value = roleId
      if (preserveForm)
        return
      providerTab.value = initialProvider ?? (s.provider === 'cloud' ? 'cloud' : 'local')
      initialProvider = undefined
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
        settingsLoadError.value = e
        toastAsyncError(e)
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

  async function retryLoadSettings(): Promise<void> {
    await loadSettings({ preserveForm: settingsReady.value })
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
        toastAsyncError(e)
    }
    finally {
      if (generation === cloudModelsLoadGeneration)
        cloudModelsLoading.value = false
    }
  }

  async function refreshOllamaModels(opts?: { userInitiated?: boolean }): Promise<void> {
    ollamaListRequest?.abort()
    const request = new AbortController()
    ollamaListRequest = request
    const generation = ++ollamaModelsLoadGeneration
    const requestBaseUrl = ollamaBaseUrl.value.trim()
    ollamaListState.value = 'checking'
    try {
      const models = await listOllamaModels(requestBaseUrl, {
        signal: request.signal,
        errorSummary: t('modelManager.ollamaListFailureHint'),
      })
      if (
        generation !== ollamaModelsLoadGeneration
        || ollamaBaseUrl.value.trim() !== requestBaseUrl
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
      ollamaListState.value = 'ready'
    }
    catch (e) {
      if (
        generation === ollamaModelsLoadGeneration
        && ollamaBaseUrl.value.trim() === requestBaseUrl
      ) {
        ollamaListState.value = 'error'
        if (opts?.userInitiated || settings.value?.localRuntimeMode !== 'performance') {
          const failure = toErrorPresentation(e, t('modelManager.ollamaListFailureHint'))
          showToast('error', failure.summary, failure.details)
        }
      }
    }
    finally {
      if (generation === ollamaModelsLoadGeneration)
        ollamaListRequest = null
    }
  }

  async function pickModelsFolder(): Promise<void> {
    try {
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
    catch (err) {
      toastAsyncError(err)
    }
  }

  async function scanCurrentFolder(): Promise<void> {
    if (!localModelsDir.value.trim()) {
      showToast('info', t('modelManager.pickFolderFirst'))
      return
    }
    try {
      folderModelFiles.value = await scanLocalModelFiles(localModelsDir.value)
    }
    catch (err) {
      toastAsyncError(err)
    }
  }

  async function openModelsFolder(): Promise<void> {
    if (!localModelsDir.value.trim()) {
      showToast('info', t('modelManager.pickFolderFirst'))
      return
    }
    try {
      await openPathInFileManager(localModelsDir.value)
    }
    catch (err) {
      toastAsyncError(err)
    }
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
      toastAsyncError(e)
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
      toastAsyncError(e)
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
      toastAsyncError(e)
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
      toastAsyncError(e)
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

  function invalidateCloudProbe(): void {
    cloudProbeGeneration += 1
    probing.value = false
  }

  async function probeCloudConnection(source: 'manual' | 'saved'): Promise<boolean> {
    if (disposed)
      return false
    const generation = ++cloudProbeGeneration
    const roleId = roleStore.currentRoleId
    const isCurrent = () => !disposed && generation === cloudProbeGeneration && roleId === roleStore.currentRoleId
    probing.value = true
    try {
      await probeCloudLlm(roleId)
      if (isCurrent())
        showToast(source === 'saved' ? 'info' : 'success', t('modelManager.probeOk'))
    }
    catch (e) {
      if (isCurrent()) {
        if (source === 'saved') {
          const failure = toErrorPresentation(e)
          showToast('warning', failure.summary, failure.details)
        }
        else {
          toastAsyncError(e)
        }
      }
    }
    finally {
      if (isCurrent())
        probing.value = false
    }
    // A closed/changed form loses permission to update UI, even if the remote
    // probe physically finishes. This does not assert cancellation upstream.
    return isCurrent()
  }

  async function runCloudProbeAfterSave(): Promise<boolean> {
    return probeCloudConnection('saved')
  }

  async function onProbeCloud(): Promise<void> {
    if (disposed || probing.value || saving.value || loading.value)
      return
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
    if (cloudProbeNeedsSave.value) {
      showToast('info', t('modelManager.probeNeedSave'))
      return
    }
    await probeCloudConnection('manual')
  }

  async function onSave(): Promise<void> {
    if (disposed || saving.value || loading.value || !settingsReady.value || settingsLoadError.value !== null)
      return
    const generation = ++settingsSaveGeneration
    const roleId = roleStore.currentRoleId
    const formGeneration = cloudFormGeneration
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
        rememberCloudModel(req.remoteModel ?? '')
        cloudModelHistory.value = getCloudModelHistory()
        showToast('success', t('modelManager.saveOk'))
        if (formGeneration !== cloudFormGeneration) {
          await loadSettings({ preserveForm: true })
          return
        }
        if (!await runCloudProbeAfterSave())
          return
      }
      if (disposed || generation !== settingsSaveGeneration || roleStore.currentRoleId !== roleId)
        return
      options.onSaved?.(roleId)
      if (!disposed && generation === settingsSaveGeneration && roleStore.currentRoleId === roleId)
        await loadSettings()
    }
    catch (e) {
      if (
        generation === settingsSaveGeneration
        && roleStore.currentRoleId === roleId
      ) {
        toastAsyncError(e)
      }
    }
    finally {
      if (generation === settingsSaveGeneration)
        saving.value = false
    }
  }

  watch(ollamaBaseUrl, () => invalidateOllamaModels(), { flush: 'sync' })

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
    [providerTab, remoteUrl, remoteModel, remoteToken],
    () => {
      cloudFormGeneration += 1
      invalidateCloudProbe()
    },
    { flush: 'sync' },
  )

  watch(
    () => roleStore.currentRoleId,
    () => {
      settingsSaveGeneration += 1
      saving.value = false
      void loadSettings()
    },
    { immediate: true, flush: 'sync' },
  )

  onBeforeUnmount(() => {
    disposed = true
    settingsReadRequest?.abort()
    settingsReadRequest = null
    invalidateCloudProbe()
    settingsLoadGeneration += 1
    invalidateOllamaModels()
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
    cloudProbeNeedsSave,
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
    ollamaListState,
    ollamaListStatusKey,
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
    settingsLoadError,
    settingsReady,
    retryLoadSettings,
    settingsLoadGeneration,
    settingsSaveGeneration,
    showToast,
    t,
    toggleLora,
  }
}
