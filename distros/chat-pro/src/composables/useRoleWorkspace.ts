import type { SessionMeta } from '@oclive/shared/api/chatStorage'
import type { useMainShell } from './useMainShell'
import { getChatStorageCapabilities, getChatStorageStats, listChatSessions } from '@oclive/shared/api/chatStorage'
import { chatThreadForSession, chatThreadStorageId } from '@oclive/shared/utils/chatThread'
import { effectiveChatSceneId } from '@oclive/shared/utils/pureChatScene'
import { computed, reactive, ref, watch } from 'vue'

/** UI navigation only. The existing shell/store remain the owners of activation and turns. */
export function useRoleWorkspace(shell: ReturnType<typeof useMainShell>) {
  const page = ref<'roles' | 'chat'>('roles')
  const busy = ref(false)
  const error = ref<unknown>(null)
  const capabilityError = ref<unknown>(null)
  const recent = reactive<Record<string, SessionMeta[]>>({})
  const recentErrors = reactive<Record<string, unknown>>({})
  const recentScenes = reactive<Record<string, string>>({})
  const drafts = reactive<Record<string, string>>({})
  const selected = reactive<Record<string, SessionMeta>>({})
  const activatedRoleId = ref('')
  const threadsSupported = ref(false)
  let refreshGeneration = 0

  const sceneId = computed(() => effectiveChatSceneId(shell.roleStore.roleInfo.interactionMode, shell.uiStore.sceneId))
  const roleId = computed(() => shell.roleStore.currentRoleId)
  const sessionKey = computed(() => shell.chatStore.chatSessionKey(roleId.value, sceneId.value))
  const draft = computed({
    get: () => drafts[sessionKey.value] ?? '',
    set: (value: string) => { drafts[sessionKey.value] = value },
  })
  const activeSession = computed(() => selected[roleId.value])

  async function refreshRecent(id: string, scene = 'home'): Promise<void> {
    try {
      const rows = await listChatSessions(id, scene, 50, 0)
      // Legacy trial-chat namespaces stay available in Data & records, not ordinary continuation.
      recent[id] = rows.filter((row) => {
        try {
          chatThreadForSession(id, row)
          return true
        }
        catch {
          return false
        }
      })
      recentScenes[id] = scene
      delete recentErrors[id]
    }
    catch (err) {
      recentErrors[id] = err
    }
  }

  async function refreshLibrary(): Promise<void> {
    const generation = ++refreshGeneration
    let storedScenes: Record<string, string> = {}
    try {
      const stats = await getChatStorageStats()
      storedScenes = Object.fromEntries(stats.map(role => [
        role.role_id,
        [...role.scenes].sort((a, b) => (b.last_active ?? '').localeCompare(a.last_active ?? ''))[0]?.scene_id ?? 'home',
      ]))
    }
    catch {
      // Recently activated roles still have an exact scene. Individual list failures remain visible.
    }
    if (generation !== refreshGeneration)
      return
    await Promise.all(shell.roleStore.roles.map(role => refreshRecent(role.id, selected[role.id]?.scene_id ?? recentScenes[role.id] ?? storedScenes[role.id] ?? 'home')))
    try {
      const caps = await getChatStorageCapabilities()
      if (generation === refreshGeneration)
        threadsSupported.value = caps.supports_chat_threads === true
      if (generation === refreshGeneration)
        capabilityError.value = caps.supports_chat_threads === true ? null : new Error(shell.t('workspace.hostUpgradeRequired'))
    }
    catch (err) {
      if (generation === refreshGeneration) {
        threadsSupported.value = false
        capabilityError.value = err
      }
    }
  }

  async function enterRole(id: string, target?: SessionMeta, newConversation = false): Promise<boolean> {
    if (busy.value || shell.roleSwitching.value)
      return false
    busy.value = true
    error.value = null
    const previousPage = page.value
    const previousRole = roleId.value
    shell.chatStore.cancelPendingSend()
    try {
      if (shell.roleStore.currentRoleId !== id || activatedRoleId.value !== id) {
        const activated = await shell.onSwitchRole(id, true)
        if (!activated)
          throw shell.roleActivationError.value || new Error(shell.t('workspace.roleLoadFailed'))
        activatedRoleId.value = id
      }
      await refreshRecent(id, sceneId.value)
      if (recentErrors[id])
        throw recentErrors[id]
      const rows = recent[id] ?? []
      const previous = selected[id]
      const candidate = newConversation ? null : target ?? previous ?? rows[0] ?? null
      if (candidate && (candidate.role_id !== id || candidate.scene_id !== sceneId.value))
        throw new Error(shell.t('workspace.recordScopeMismatch'))
      if (!candidate && !threadsSupported.value) {
        const caps = await getChatStorageCapabilities()
        threadsSupported.value = caps.supports_chat_threads === true
        if (!threadsSupported.value)
          throw new Error(shell.t('workspace.hostUpgradeRequired'))
        capabilityError.value = null
      }
      // An unsent conversation is kept in memory until its first successful backend append.
      if (candidate?.message_count === 0 && selected[id]?.session_id === candidate.session_id) {
        page.value = 'chat'
        return true
      }
      await shell.chatStore.selectChatSession(id, sceneId.value, candidate)
      const now = new Date().toISOString()
      selected[id] = candidate ?? {
        role_id: id,
        scene_id: sceneId.value,
        session_id: chatThreadStorageId(id, shell.chatStore.chatThreadId(id, sceneId.value) ?? null),
        created_at: now,
        updated_at: now,
        message_count: 0,
        last_message_snippet: '',
      }
      page.value = 'chat'
      return true
    }
    catch (err) {
      error.value = err
      page.value = previousPage === 'chat' && previousRole === id && selected[id] ? 'chat' : 'roles'
      return false
    }
    finally {
      busy.value = false
    }
  }

  function openLibrary(): void {
    page.value = 'roles'
    error.value = null
    void refreshLibrary()
  }

  async function onImported(id: string): Promise<void> {
    try {
      await shell.roleStore.loadRoles(false)
      delete selected[id]
      activatedRoleId.value = ''
      await refreshLibrary()
      await enterRole(id)
    }
    catch (err) {
      error.value = err
      page.value = 'roles'
    }
  }

  async function submitMessage(payload: { content: string }): Promise<void> {
    const key = sessionKey.value
    const outcome = await shell.onSend(payload)
    if (outcome?.status === 'failed' && key === sessionKey.value && !draft.value)
      draft.value = payload.content
  }

  watch(() => shell.startupState.value, (value) => {
    if (value === 'ready')
      void refreshLibrary()
  }, { immediate: true })
  watch(() => shell.chatStore.isLoading, async (loading, previous) => {
    if (!previous || loading || !roleId.value)
      return
    const id = roleId.value
    await refreshRecent(id, sceneId.value)
    const row = recent[id]?.find(item => item.session_id === selected[id]?.session_id)
    if (row)
      selected[id] = row
  })

  return { page, busy, error, capabilityError, recent, recentErrors, threadsSupported, draft, activeSession, enterRole, openLibrary, refreshLibrary, onImported, submitMessage }
}
