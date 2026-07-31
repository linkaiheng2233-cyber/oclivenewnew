import type {
  ChatSearchResult,
  ReplayTarget,
  RoleChatStorageConfig,
  RoleStorageStat,
  SceneStorageStat,
  SessionMeta,
  StoredMessage,
} from '@oclive/shared/api/chatStorage'
import {
  deleteChatMessage,
  deleteRoleChats,
  deleteSceneChats,
  editChatMessage,
  exportChatSession,
  exportRoleChats,
  fetchChatMessages,
  getChatStorageCapabilities,
  getChatStorageRoot,
  getChatStorageStats,
  getReplayProgress,
  getRoleChatStorageConfig,
  listChatSessions,
  replayMemoryExtraction,
  runChatAutoCleanup,
  saveRoleChatStorageConfig,
  searchChatMessages,
  setChatStorageRoot,
} from '@oclive/shared/api/chatStorage'
import { listRoles } from '@oclive/shared/api/role'
import { formatStorageBytes } from '@oclive/shared/components/settings/chat-storage/chatStorageFormat'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { downloadBase64File, downloadTextFile } from '@oclive/shared/utils/download'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

type Level = 'roles' | 'scenes' | 'sessions' | 'messages'

export function useChatStorageSettings() {
  const { t } = useI18n()
  const { showToast } = useAppToast()

  const level = ref<Level>('roles')
  const loadingStats = ref(false)
  const loadingSearch = ref(false)
  const loadingSessions = ref(false)
  const loadingMessages = ref(false)
  const loadingMutating = ref(false)
  const loading = computed(
    () =>
      loadingStats.value
      || loadingSearch.value
      || loadingSessions.value
      || loadingMessages.value
      || loadingMutating.value,
  )
  const stats = ref<RoleStorageStat[]>([])
  const roleNames = ref<Record<string, string>>({})
  const selectedRole = ref<RoleStorageStat | null>(null)
  const selectedScene = ref<SceneStorageStat | null>(null)
  const selectedSession = ref<SessionMeta | null>(null)
  const sessions = ref<SessionMeta[]>([])
  const messages = ref<StoredMessage[]>([])
  const statsLoadedAt = ref(0)
  const storageRoot = ref('')
  const CACHE_MS = 5 * 60 * 1000

  const searchQuery = ref('')
  const searchResults = ref<ChatSearchResult[]>([])
  const searchActive = computed(() => searchQuery.value.trim().length > 0)

  const showCleanupModal = ref(false)
  const cleanupDraft = ref<RoleChatStorageConfig>({})
  const cleanupRoleId = ref('')

  const roleStorageConfig = ref<RoleChatStorageConfig | null>(null)
  let openRoleGeneration = 0

  const editingMessageId = ref<string | null>(null)
  const editingContent = ref('')

  const replayTaskId = ref<string | null>(null)
  const replayProgress = ref<number>(0)
  const replayPolling = ref<ReturnType<typeof setInterval> | null>(null)

  const capabilities = ref({
    backend_kind: 'hybrid',
    supports_search: true,
    supports_replay: false,
    supports_cleanup: false,
  })
  const capabilitiesDegraded = ref(false)

  onUnmounted(() => {
    if (replayPolling.value)
      clearInterval(replayPolling.value)
  })

  const selectedRoleLabel = computed(() => {
    const id = selectedRole.value?.role_id ?? ''
    return roleNames.value[id] ?? id
  })

  const backendLabel = computed(() => {
    const kind = capabilities.value?.backend_kind ?? 'hybrid'
    return t(`chatStorage.backends.${kind}`) ?? kind
  })

  const replayActive = computed(() => replayTaskId.value !== null)

  async function loadStorageRoot() {
    try {
      storageRoot.value = await getChatStorageRoot()
    }
    catch (err) {
      console.warn('[ChatStorageSettings] getChatStorageRoot failed', err)
    }
  }

  async function changeStorageRoot() {
    try {
      const picked = await open({ directory: true, multiple: false })
      if (!picked || Array.isArray(picked))
        return
      storageRoot.value = await setChatStorageRoot(picked, true)
      showToast('success', t('chatStorage.rootUpdated'))
      await refreshStats(true)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function loadRoleNames() {
    try {
      const roles = await listRoles()
      const map: Record<string, string> = {}
      for (const r of roles)
        map[r.id] = r.name || r.id
      roleNames.value = map
    }
    catch (err) {
      console.warn('[ChatStorageSettings] loadRoleNames failed', err)
    }
  }

  async function refreshStats(force = false) {
    const now = Date.now()
    if (!force && stats.value.length > 0 && now - statsLoadedAt.value < CACHE_MS)
      return
    loadingStats.value = true
    try {
      stats.value = await getChatStorageStats()
      statsLoadedAt.value = Date.now()
    }
    catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      if (msg.includes('get_storage_stats') || msg.includes('does not support'))
        showToast('info', t('chatStorage.statsUnsupported'))
      else
        showToast('error', msg)
    }
    finally {
      loadingStats.value = false
    }
  }

  async function runSearch() {
    const q = searchQuery.value.trim()
    if (!q) {
      searchResults.value = []
      return
    }
    loadingSearch.value = true
    try {
      searchResults.value = await searchChatMessages(
        q,
        selectedRole.value?.role_id ?? null,
        100,
        0,
      )
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingSearch.value = false
    }
  }

  async function jumpToSearchResult(row: ChatSearchResult) {
    const roleStat = stats.value.find(r => r.role_id === row.role_id)
    if (!roleStat)
      return
    selectedRole.value = roleStat
    const scene = roleStat.scenes.find(s => s.scene_id === row.scene_id)
    if (!scene)
      return
    selectedScene.value = scene
    level.value = 'sessions'
    await loadSessions()
    const sess = sessions.value.find(s => s.session_id === row.session_id)
    if (sess)
      await openSession(sess)
  }

  function openRole(row: RoleStorageStat) {
    const gen = ++openRoleGeneration
    selectedRole.value = row
    selectedScene.value = null
    selectedSession.value = null
    sessions.value = []
    messages.value = []
    roleStorageConfig.value = null
    level.value = 'scenes'
    void getRoleChatStorageConfig(row.role_id)
      .then((cfg) => {
        if (gen !== openRoleGeneration)
          return
        roleStorageConfig.value = cfg
      })
      .catch((err) => {
        if (gen !== openRoleGeneration)
          return
        roleStorageConfig.value = null
        showToast('error', err instanceof Error ? err.message : String(err))
      })
  }

  function openScene(scene: SceneStorageStat) {
    selectedScene.value = scene
    selectedSession.value = null
    messages.value = []
    level.value = 'sessions'
    void loadSessions()
  }

  async function loadSessions() {
    const role = selectedRole.value
    const scene = selectedScene.value
    if (!role || !scene)
      return
    loadingSessions.value = true
    try {
      sessions.value = await listChatSessions(role.role_id, scene.scene_id, 50, 0)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingSessions.value = false
    }
  }

  async function openSession(session: SessionMeta) {
    selectedSession.value = session
    level.value = 'messages'
    loadingMessages.value = true
    try {
      messages.value = await fetchChatMessages(session.session_id, 500, 0)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMessages.value = false
    }
  }

  function goBack() {
    if (level.value === 'messages') {
      level.value = 'sessions'
      selectedSession.value = null
      messages.value = []
      return
    }
    if (level.value === 'sessions') {
      level.value = 'scenes'
      selectedScene.value = null
      return
    }
    if (level.value === 'scenes') {
      level.value = 'roles'
      selectedRole.value = null
    }
  }

  async function handleExportSession(sessionId: string, format: 'markdown' | 'json') {
    loadingMutating.value = true
    try {
      const res = await exportChatSession(sessionId, format)
      if (res.content_encoding === 'base64')
        downloadBase64File(res.suggested_filename, res.content, res.mime_type)
      else
        downloadTextFile(res.suggested_filename, res.content, res.mime_type)
      showToast('info', t('chatStorage.exported', { name: res.suggested_filename }))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function startMemoryReplay(
    source: 'session' | 'scene' | 'role',
    roleId: string,
    sceneId?: string,
    sessionId?: string,
  ) {
    if (!window.confirm(t('chatStorage.replayConfirm')))
      return
    loadingMutating.value = true
    try {
      const roleCfg = await getRoleChatStorageConfig(roleId)
      const target: ReplayTarget = {
        role_id: roleId,
        scene_id: sceneId ?? null,
        session_id: sessionId ?? null,
        similarity_threshold: roleCfg.replay_similarity_threshold ?? 0.6,
      }
      const taskId = await replayMemoryExtraction(source, target)
      replayTaskId.value = taskId
      replayProgress.value = 0
      if (replayPolling.value)
        clearInterval(replayPolling.value)
      replayPolling.value = setInterval(async () => {
        if (!replayTaskId.value)
          return
        try {
          const p = await getReplayProgress(replayTaskId.value)
          replayProgress.value = p.percent
          if (p.done) {
            if (replayPolling.value)
              clearInterval(replayPolling.value)
            replayPolling.value = null
            replayTaskId.value = null
            if (p.errors.length) {
              showToast('error', p.errors.join('; '))
            }
            else {
              showToast('info', t('chatStorage.replayDone', {
                turns: p.processed_turns,
                newMem: p.new_memories,
                updated: p.updated_memories,
                skipped: p.skipped_memories,
              }))
            }
          }
        }
        catch (err) {
          if (replayPolling.value)
            clearInterval(replayPolling.value)
          replayPolling.value = null
          replayTaskId.value = null
          showToast('error', err instanceof Error ? err.message : t('chatStorage.replayFailed'))
        }
      }, 800)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : t('chatStorage.replayFailed'))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function handleExportRole(roleId: string, format: 'markdown' | 'json') {
    loadingMutating.value = true
    try {
      const res = await exportRoleChats(roleId, format)
      if (res.content_encoding === 'base64')
        downloadBase64File(res.suggested_filename, res.content, res.mime_type)
      else
        downloadTextFile(res.suggested_filename, res.content, res.mime_type)
      showToast('info', t('chatStorage.exported', { name: res.suggested_filename }))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function openCleanupSettings(roleId: string) {
    cleanupRoleId.value = roleId
    loadingMutating.value = true
    try {
      cleanupDraft.value = await getRoleChatStorageConfig(roleId)
      showCleanupModal.value = true
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function saveCleanupSettings() {
    const roleId = cleanupRoleId.value
    loadingMutating.value = true
    try {
      await saveRoleChatStorageConfig(roleId, cleanupDraft.value)
      const result = await runChatAutoCleanup(roleId)
      showCleanupModal.value = false
      showToast('info', t('chatStorage.cleanupSaved', {
        sessions: result.sessions_deleted,
        size: formatStorageBytes(result.bytes_freed),
      }))
      await refreshStats(true)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function confirmDeleteRole(roleId: string) {
    if (!window.confirm(t('chatStorage.deleteRoleConfirm')))
      return
    loadingMutating.value = true
    try {
      const res = await deleteRoleChats(roleId)
      showToast('info', t('chatStorage.deletedToast', {
        sessions: res.sessions_deleted,
        size: formatStorageBytes(res.bytes_freed),
      }))
      level.value = 'roles'
      selectedRole.value = null
      selectedScene.value = null
      await refreshStats(true)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function confirmDeleteScene(roleId: string, sceneId: string) {
    if (!window.confirm(t('chatStorage.deleteSceneConfirm')))
      return
    loadingMutating.value = true
    try {
      const res = await deleteSceneChats(roleId, sceneId)
      showToast('info', t('chatStorage.deletedToast', {
        sessions: res.sessions_deleted,
        size: formatStorageBytes(res.bytes_freed),
      }))
      selectedScene.value = null
      level.value = 'scenes'
      await refreshStats(true)
      if (selectedRole.value) {
        const updated = stats.value.find(r => r.role_id === roleId)
        if (updated)
          selectedRole.value = updated
      }
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  function startEditMessage(msg: StoredMessage) {
    if (msg.sender !== 'user')
      return
    editingMessageId.value = msg.id
    editingContent.value = msg.content
  }

  function cancelEdit() {
    editingMessageId.value = null
    editingContent.value = ''
  }

  async function confirmEditMessage() {
    const id = editingMessageId.value
    if (!id)
      return
    loadingMutating.value = true
    try {
      await editChatMessage(id, editingContent.value)
      cancelEdit()
      if (selectedSession.value)
        await openSession(selectedSession.value)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  async function confirmDeleteMessage(msg: StoredMessage) {
    if (!window.confirm(t('chatStorage.deleteMessageConfirm')))
      return
    loadingMutating.value = true
    try {
      await deleteChatMessage(msg.id)
      if (selectedSession.value)
        await openSession(selectedSession.value)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      loadingMutating.value = false
    }
  }

  onMounted(() => {
    void loadRoleNames()
    void loadStorageRoot()
    void refreshStats()
    void getChatStorageCapabilities().then((c) => {
      capabilities.value = c
      capabilitiesDegraded.value = false
    }).catch((err) => {
      capabilitiesDegraded.value = true
      capabilities.value = {
        backend_kind: 'hybrid',
        supports_search: false,
        supports_replay: false,
        supports_cleanup: false,
      }
      console.warn('[ChatStorageSettings] getChatStorageCapabilities failed', err)
    })
  })

  return {
    level,
    loadingStats,
    loadingSearch,
    loadingSessions,
    loadingMessages,
    loadingMutating,
    loading,
    stats,
    roleNames,
    selectedRole,
    selectedScene,
    selectedSession,
    sessions,
    messages,
    storageRoot,
    searchQuery,
    searchResults,
    searchActive,
    showCleanupModal,
    cleanupDraft,
    roleStorageConfig,
    editingMessageId,
    editingContent,
    replayProgress,
    capabilities,
    capabilitiesDegraded,
    selectedRoleLabel,
    backendLabel,
    replayActive,
    changeStorageRoot,
    refreshStats,
    runSearch,
    jumpToSearchResult,
    openRole,
    openScene,
    openSession,
    goBack,
    handleExportSession,
    startMemoryReplay,
    handleExportRole,
    openCleanupSettings,
    saveCleanupSettings,
    confirmDeleteRole,
    confirmDeleteScene,
    startEditMessage,
    cancelEdit,
    confirmEditMessage,
    confirmDeleteMessage,
  }
}
