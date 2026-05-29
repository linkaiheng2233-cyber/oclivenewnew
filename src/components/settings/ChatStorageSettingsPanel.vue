<script setup lang="ts">
import type {
  ChatSearchResult,
  RoleChatStorageConfig,
  RoleStorageStat,
  SceneStorageStat,
  SessionMeta,
  StoredMessage,
} from '../../api/chatStorage'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  deleteChatMessage,
  deleteRoleChats,
  deleteSceneChats,
  editChatMessage,
  exportChatSession,
  exportRoleChats,
  fetchChatMessages,
  getChatStorageCapabilities,
  getChatStorageStats,
  getReplayProgress,
  getRoleChatStorageConfig,
  listChatSessions,
  replayMemoryExtraction,
  runChatAutoCleanup,
  saveRoleChatStorageConfig,
  searchChatMessages,
} from '../../api/chatStorage'
import { useAppToast } from '../../composables/useAppToast'
import { listRoles } from '../../api/role'
import { downloadBase64File, downloadTextFile } from '../../utils/download'

const { t } = useI18n()
const { showToast } = useAppToast()

type Level = 'roles' | 'scenes' | 'sessions' | 'messages'

const level = ref<Level>('roles')
const loading = ref(false)
const stats = ref<RoleStorageStat[]>([])
const roleNames = ref<Record<string, string>>({})
const selectedRole = ref<RoleStorageStat | null>(null)
const selectedScene = ref<SceneStorageStat | null>(null)
const selectedSession = ref<SessionMeta | null>(null)
const sessions = ref<SessionMeta[]>([])
const messages = ref<StoredMessage[]>([])
const statsLoadedAt = ref(0)
const CACHE_MS = 5 * 60 * 1000

const searchQuery = ref('')
const searchResults = ref<ChatSearchResult[]>([])
const searchActive = computed(() => searchQuery.value.trim().length > 0)

const showCleanupModal = ref(false)
const cleanupDraft = ref<RoleChatStorageConfig>({})
const cleanupRoleId = ref('')

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

function formatBytes(n: number): string {
  if (n < 1024)
    return `${n} B`
  if (n < 1024 * 1024)
    return `${(n / 1024).toFixed(1)} KB`
  return `${(n / (1024 * 1024)).toFixed(2)} MB`
}

async function loadRoleNames() {
  try {
    const roles = await listRoles()
    const map: Record<string, string> = {}
    for (const r of roles)
      map[r.id] = r.name || r.id
    roleNames.value = map
  }
  catch {
    /* optional */
  }
}

async function refreshStats(force = false) {
  const now = Date.now()
  if (!force && stats.value.length > 0 && now - statsLoadedAt.value < CACHE_MS)
    return
  loading.value = true
  try {
    stats.value = await getChatStorageStats()
    statsLoadedAt.value = Date.now()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function runSearch() {
  const q = searchQuery.value.trim()
  if (!q) {
    searchResults.value = []
    return
  }
  loading.value = true
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
    loading.value = false
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
  selectedRole.value = row
  selectedScene.value = null
  selectedSession.value = null
  sessions.value = []
  messages.value = []
  level.value = 'scenes'
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
  loading.value = true
  try {
    sessions.value = await listChatSessions(role.role_id, scene.scene_id, 50, 0)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function openSession(session: SessionMeta) {
  selectedSession.value = session
  level.value = 'messages'
  loading.value = true
  try {
    messages.value = await fetchChatMessages(session.session_id, 500, 0)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
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
  loading.value = true
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
    loading.value = false
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
  loading.value = true
  try {
    const taskId = await replayMemoryExtraction(source, {
      role_id: roleId,
      scene_id: sceneId ?? null,
      session_id: sessionId ?? null,
    })
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
          if (p.errors.length)
            showToast('error', p.errors.join('; '))
          else
            showToast('info', t('chatStorage.replayDone', {
              turns: p.processed_turns,
              newMem: p.new_memories,
              updated: p.updated_memories,
              skipped: p.skipped_memories,
            }))
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
    loading.value = false
  }
}

const replayActive = computed(() => replayTaskId.value !== null)

async function handleExportRole(roleId: string, format: 'markdown' | 'json') {
  loading.value = true
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
    loading.value = false
  }
}

async function openCleanupSettings(roleId: string) {
  cleanupRoleId.value = roleId
  loading.value = true
  try {
    cleanupDraft.value = await getRoleChatStorageConfig(roleId)
    showCleanupModal.value = true
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function saveCleanupSettings() {
  const roleId = cleanupRoleId.value
  loading.value = true
  try {
    await saveRoleChatStorageConfig(roleId, cleanupDraft.value)
    const result = await runChatAutoCleanup(roleId)
    showCleanupModal.value = false
    showToast('info', t('chatStorage.cleanupSaved', {
      sessions: result.sessions_deleted,
      size: formatBytes(result.bytes_freed),
    }))
    await refreshStats(true)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function confirmDeleteRole(roleId: string) {
  if (!window.confirm(t('chatStorage.deleteRoleConfirm')))
    return
  loading.value = true
  try {
    const res = await deleteRoleChats(roleId)
    showToast('info', t('chatStorage.deletedToast', {
      sessions: res.sessions_deleted,
      size: formatBytes(res.bytes_freed),
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
    loading.value = false
  }
}

async function confirmDeleteScene(roleId: string, sceneId: string) {
  if (!window.confirm(t('chatStorage.deleteSceneConfirm')))
    return
  loading.value = true
  try {
    const res = await deleteSceneChats(roleId, sceneId)
    showToast('info', t('chatStorage.deletedToast', {
      sessions: res.sessions_deleted,
      size: formatBytes(res.bytes_freed),
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
    loading.value = false
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
  loading.value = true
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
    loading.value = false
  }
}

async function confirmDeleteMessage(msg: StoredMessage) {
  if (!window.confirm(t('chatStorage.deleteMessageConfirm')))
    return
  loading.value = true
  try {
    await deleteChatMessage(msg.id)
    if (selectedSession.value)
      await openSession(selectedSession.value)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadRoleNames()
  void refreshStats()
  void getChatStorageCapabilities().then((c) => {
    capabilities.value = c
  }).catch(() => {
    /* keep defaults */
  })
})

defineExpose({ refreshStats })
</script>

<template>
  <section class="css-panel">
    <div class="css-head">
      <button
        v-if="level !== 'roles'"
        type="button"
        class="css-back"
        @click="goBack"
      >
        {{ t('chatStorage.back') }}
      </button>
      <h3 class="css-title">
        {{ t('chatStorage.title') }}
      </h3>
      <button
        type="button"
        class="css-refresh"
        :disabled="loading"
        @click="refreshStats(true)"
      >
        {{ t('chatStorage.refresh') }}
      </button>
    </div>
    <p class="css-lead">
      {{ t('chatStorage.lead') }}
    </p>
    <p v-if="capabilities.backend_kind" class="css-backend-hint">
      {{ t('chatStorage.backendLabel', { backend: backendLabel }) }}
    </p>

    <div v-if="capabilities.supports_search" class="css-search">
      <input
        v-model="searchQuery"
        type="search"
        class="css-search-input"
        :placeholder="t('chatStorage.searchPlaceholder')"
        @keydown.enter="runSearch"
      >
      <button type="button" class="css-search-btn" :disabled="loading" @click="runSearch">
        {{ t('chatStorage.search') }}
      </button>
    </div>

    <ul v-if="capabilities.supports_search && searchActive && searchResults.length > 0" class="css-list css-search-results">
      <li class="css-breadcrumb">
        {{ t('chatStorage.searchResults', { n: searchResults.length }) }}
      </li>
      <li
        v-for="row in searchResults"
        :key="row.message.id"
        class="css-row"
      >
        <button type="button" class="css-row-main" @click="jumpToSearchResult(row)">
          <span class="css-row-title">{{ row.highlight_snippet }}</span>
          <span class="css-row-meta">
            {{ roleNames[row.role_id] ?? row.role_id }} / {{ row.scene_id }}
            · {{ row.message.created_at }}
          </span>
        </button>
      </li>
    </ul>

    <div v-if="replayActive" class="css-replay-bar">
      <div class="css-replay-track">
        <div class="css-replay-fill" :style="{ width: `${replayProgress}%` }" />
      </div>
      <span class="css-muted">{{ t('chatStorage.replayRunning', { percent: replayProgress }) }}</span>
    </div>

    <div v-if="loading" class="css-muted">
      {{ t('chatStorage.loading') }}
    </div>

    <ul v-else-if="level === 'roles'" class="css-list">
      <li
        v-for="row in stats"
        :key="row.role_id"
        class="css-row"
      >
        <button type="button" class="css-row-main" @click="openRole(row)">
          <span class="css-row-title">{{ roleNames[row.role_id] ?? row.role_id }}</span>
          <span class="css-row-meta">
            {{ t('chatStorage.scenesCount', { n: row.scene_count }) }}
            · {{ formatBytes(row.total_size_bytes) }}
            <template v-if="row.last_active">
              · {{ row.last_active }}
            </template>
          </span>
        </button>
        <div class="css-actions">
          <button type="button" class="css-action" @click.stop="handleExportRole(row.role_id, 'markdown')">
            {{ t('chatStorage.exportMd') }}
          </button>
          <button type="button" class="css-action" @click.stop="handleExportRole(row.role_id, 'json')">
            {{ t('chatStorage.exportJson') }}
          </button>
          <button
            v-if="capabilities.supports_cleanup"
            type="button"
            class="css-action"
            @click.stop="openCleanupSettings(row.role_id)"
          >
            {{ t('chatStorage.autoCleanup') }}
          </button>
          <button
            v-if="capabilities.supports_replay"
            type="button"
            class="css-action"
            @click.stop="startMemoryReplay('role', row.role_id)"
          >
            {{ t('chatStorage.replayMemory') }}
          </button>
          <button type="button" class="css-danger" @click.stop="confirmDeleteRole(row.role_id)">
            {{ t('chatStorage.delete') }}
          </button>
        </div>
      </li>
      <li v-if="stats.length === 0" class="css-muted">
        {{ t('chatStorage.empty') }}
      </li>
    </ul>

    <ul v-else-if="level === 'scenes' && selectedRole" class="css-list">
      <li
        v-for="scene in selectedRole.scenes"
        :key="scene.scene_id"
        class="css-row"
      >
        <button type="button" class="css-row-main" @click="openScene(scene)">
          <span class="css-row-title">{{ scene.scene_id }}</span>
          <span class="css-row-meta">
            {{ t('chatStorage.sessionsCount', { n: scene.session_count }) }}
            · {{ formatBytes(scene.total_size_bytes) }}
          </span>
        </button>
        <button
          v-if="capabilities.supports_replay"
          type="button"
          class="css-action"
          @click.stop="startMemoryReplay('scene', selectedRole.role_id, scene.scene_id)"
        >
          {{ t('chatStorage.replayMemory') }}
        </button>
        <button
          type="button"
          class="css-danger"
          @click.stop="confirmDeleteScene(selectedRole.role_id, scene.scene_id)"
        >
          {{ t('chatStorage.delete') }}
        </button>
      </li>
    </ul>

    <ul v-else-if="level === 'sessions' && selectedRole && selectedScene" class="css-list">
      <li class="css-breadcrumb">
        {{ selectedRoleLabel }} / {{ selectedScene.scene_id }}
      </li>
      <li
        v-for="sess in sessions"
        :key="sess.session_id"
        class="css-row"
      >
        <button type="button" class="css-row-main" @click="openSession(sess)">
          <span class="css-row-title">{{ sess.session_id.slice(0, 24) }}…</span>
          <span class="css-row-meta">{{ sess.last_message_snippet || t('chatStorage.emptyPreview') }}</span>
        </button>
        <div class="css-actions">
          <button type="button" class="css-action" @click.stop="handleExportSession(sess.session_id, 'markdown')">
            {{ t('chatStorage.export') }}
          </button>
        </div>
      </li>
      <li v-if="sessions.length === 0" class="css-muted">
        {{ t('chatStorage.empty') }}
      </li>
    </ul>

    <ul v-else-if="level === 'messages' && selectedSession" class="css-list css-messages">
      <li class="css-breadcrumb">
        {{ selectedSession.session_id.slice(0, 32) }}…
      </li>
      <li
        v-for="msg in messages"
        :key="msg.id"
        class="css-msg"
        :class="{ 'css-msg-user': msg.sender === 'user' }"
      >
        <div class="css-msg-head">
          <span>{{ msg.sender }}</span>
          <span class="css-msg-time">{{ msg.created_at }}</span>
        </div>
        <template v-if="editingMessageId === msg.id">
          <textarea v-model="editingContent" class="css-edit-area" rows="3" />
          <div class="css-msg-actions">
            <button type="button" class="css-action" @click="confirmEditMessage">
              {{ t('chatStorage.save') }}
            </button>
            <button type="button" class="css-action" @click="cancelEdit">
              {{ t('chatStorage.cancel') }}
            </button>
          </div>
        </template>
        <template v-else>
          <p class="css-msg-body">
            {{ msg.content }}
          </p>
          <div v-if="msg.sender === 'user'" class="css-msg-actions">
            <button type="button" class="css-action" @click="startEditMessage(msg)">
              {{ t('chatStorage.edit') }}
            </button>
            <button type="button" class="css-danger-inline" @click="confirmDeleteMessage(msg)">
              {{ t('chatStorage.delete') }}
            </button>
          </div>
        </template>
      </li>
      <li class="css-export-row css-row">
        <button type="button" class="css-action" @click="handleExportSession(selectedSession.session_id, 'markdown')">
          {{ t('chatStorage.exportMd') }}
        </button>
        <button type="button" class="css-action" @click="handleExportSession(selectedSession.session_id, 'json')">
          {{ t('chatStorage.exportJson') }}
        </button>
        <button
          v-if="capabilities.supports_replay && selectedRole"
          type="button"
          class="css-action"
          @click="startMemoryReplay('session', selectedRole.role_id, selectedScene?.scene_id, selectedSession.session_id)"
        >
          {{ t('chatStorage.replayMemory') }}
        </button>
      </li>
    </ul>

    <div v-if="showCleanupModal" class="css-modal-backdrop" @click.self="showCleanupModal = false">
      <div class="css-modal">
        <h4>{{ t('chatStorage.autoCleanupTitle') }}</h4>
        <label class="css-field">
          <span>{{ t('chatStorage.maxMessages') }}</span>
          <input
            v-model.number="cleanupDraft.max_messages_per_session"
            type="number"
            min="2"
            placeholder="500"
          >
        </label>
        <label class="css-field">
          <span>{{ t('chatStorage.autoCleanupDays') }}</span>
          <input
            v-model.number="cleanupDraft.auto_cleanup_days"
            type="number"
            min="1"
            :placeholder="t('chatStorage.optional')"
          >
        </label>
        <label class="css-field">
          <span>{{ t('chatStorage.autoCleanupMaxSessions') }}</span>
          <input
            v-model.number="cleanupDraft.auto_cleanup_max_sessions"
            type="number"
            min="1"
            :placeholder="t('chatStorage.optional')"
          >
        </label>
        <p class="css-muted">
          {{ t('chatStorage.autoCleanupHint') }}
        </p>
        <div class="css-modal-actions">
          <button type="button" class="css-action" @click="saveCleanupSettings">
            {{ t('chatStorage.save') }}
          </button>
          <button type="button" class="css-action" @click="showCleanupModal = false">
            {{ t('chatStorage.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.css-replay-bar {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.css-replay-track {
  height: 6px;
  border-radius: 3px;
  background: var(--oc-border, #333);
  overflow: hidden;
}
.css-replay-fill {
  height: 100%;
  background: var(--oc-accent, #6cf);
  transition: width 0.3s ease;
}
.css-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.css-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.css-title {
  margin: 0;
  font-size: 1rem;
  flex: 1;
}
.css-lead {
  margin: 0;
  font-size: 0.875rem;
  opacity: 0.85;
}
.css-backend-hint {
  font-size: 0.85rem;
  color: var(--oc-muted, #888);
  margin-bottom: 0.75rem;
}
.css-search {
  display: flex;
  gap: 0.5rem;
}
.css-search-input {
  flex: 1;
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--oc-border, #333);
  background: transparent;
  color: inherit;
}
.css-search-btn {
  padding: 0.4rem 0.75rem;
  font-size: 0.85rem;
}
.css-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.css-row {
  display: flex;
  gap: 0.5rem;
  align-items: stretch;
  border: 1px solid var(--oc-border, #333);
  border-radius: 8px;
  overflow: hidden;
}
.css-row-main {
  flex: 1;
  text-align: left;
  padding: 0.6rem 0.75rem;
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}
.css-row-title {
  display: block;
  font-weight: 600;
}
.css-row-meta {
  display: block;
  font-size: 0.8rem;
  opacity: 0.75;
  margin-top: 0.2rem;
}
.css-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  border-left: 1px solid var(--oc-border, #333);
}
.css-action {
  padding: 0 0.6rem;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font-size: 0.75rem;
  white-space: nowrap;
}
.css-danger {
  padding: 0 0.75rem;
  border: none;
  border-left: 1px solid var(--oc-border, #333);
  background: rgba(180, 40, 40, 0.15);
  color: inherit;
  cursor: pointer;
  font-size: 0.8rem;
}
.css-danger-inline {
  padding: 0.2rem 0.5rem;
  border: none;
  background: rgba(180, 40, 40, 0.15);
  color: inherit;
  cursor: pointer;
  font-size: 0.75rem;
  border-radius: 4px;
}
.css-muted {
  opacity: 0.7;
  font-size: 0.875rem;
}
.css-back,
.css-refresh {
  font-size: 0.8rem;
  padding: 0.25rem 0.5rem;
}
.css-breadcrumb {
  padding: 0.25rem 0.5rem;
  font-size: 0.8rem;
  opacity: 0.8;
}
.css-msg {
  padding: 0.6rem 0.75rem;
  border: 1px solid var(--oc-border, #333);
  border-radius: 8px;
}
.css-msg-user {
  border-color: rgba(80, 120, 200, 0.35);
}
.css-msg-head {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  opacity: 0.8;
  margin-bottom: 0.35rem;
}
.css-msg-body {
  margin: 0;
  white-space: pre-wrap;
  font-size: 0.875rem;
}
.css-msg-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.4rem;
}
.css-edit-area {
  width: 100%;
  box-sizing: border-box;
  padding: 0.4rem;
  border-radius: 6px;
  border: 1px solid var(--oc-border, #333);
  background: transparent;
  color: inherit;
  font: inherit;
}
.css-export-row {
  justify-content: flex-start;
  padding: 0.5rem;
  gap: 0.5rem;
}
.css-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}
.css-modal {
  background: var(--oc-bg, #1a1a1a);
  border: 1px solid var(--oc-border, #333);
  border-radius: 10px;
  padding: 1rem;
  min-width: 280px;
  max-width: 90vw;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
.css-modal h4 {
  margin: 0;
}
.css-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
}
.css-field input {
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--oc-border, #333);
  background: transparent;
  color: inherit;
}
.css-modal-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}
</style>
