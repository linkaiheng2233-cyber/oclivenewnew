<script setup lang="ts">
import type { RoleStorageStat, SceneStorageStat, SessionMeta } from '../../api/chatStorage'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  deleteRoleChats,
  deleteSceneChats,
  fetchChatMessages,
  getChatStorageStats,
  listChatSessions,
} from '../../api/chatStorage'
import { useAppToast } from '../../composables/useAppToast'
import { listRoles } from '../../api/role'
const { t } = useI18n()
const { showToast } = useAppToast()

type Level = 'roles' | 'scenes' | 'sessions'

const level = ref<Level>('roles')
const loading = ref(false)
const stats = ref<RoleStorageStat[]>([])
const roleNames = ref<Record<string, string>>({})
const selectedRole = ref<RoleStorageStat | null>(null)
const selectedScene = ref<SceneStorageStat | null>(null)
const sessions = ref<SessionMeta[]>([])
const sessionMessages = ref<{ sessionId: string, preview: string }[]>([])
const statsLoadedAt = ref(0)
const CACHE_MS = 5 * 60 * 1000

const selectedRoleLabel = computed(() => {
  const id = selectedRole.value?.role_id ?? ''
  return roleNames.value[id] ?? id
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

function openRole(row: RoleStorageStat) {
  selectedRole.value = row
  selectedScene.value = null
  sessions.value = []
  sessionMessages.value = []
  level.value = 'scenes'
}

function openScene(scene: SceneStorageStat) {
  selectedScene.value = scene
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
    sessionMessages.value = sessions.value.map(s => ({
      sessionId: s.session_id,
      preview: s.last_message_snippet || t('chatStorage.emptyPreview'),
    }))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function previewSession(sessionId: string) {
  loading.value = true
  try {
    const msgs = await fetchChatMessages(sessionId, 20, 0)
    const tail = msgs.slice(-3).map(m => `${m.sender}: ${m.content}`).join('\n')
    showToast('info', tail || t('chatStorage.emptyPreview'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

function goBack() {
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

onMounted(() => {
  void loadRoleNames()
  void refreshStats()
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
        <button
          type="button"
          class="css-danger"
          @click.stop="confirmDeleteRole(row.role_id)"
        >
          {{ t('chatStorage.delete') }}
        </button>
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
        v-for="item in sessionMessages"
        :key="item.sessionId"
        class="css-row"
      >
        <button type="button" class="css-row-main" @click="previewSession(item.sessionId)">
          <span class="css-row-title">{{ item.sessionId.slice(0, 24) }}…</span>
          <span class="css-row-meta">{{ item.preview }}</span>
        </button>
      </li>
      <li v-if="sessionMessages.length === 0" class="css-muted">
        {{ t('chatStorage.empty') }}
      </li>
    </ul>
  </section>
</template>

<style scoped>
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
.css-danger {
  padding: 0 0.75rem;
  border: none;
  border-left: 1px solid var(--oc-border, #333);
  background: rgba(180, 40, 40, 0.15);
  color: inherit;
  cursor: pointer;
  font-size: 0.8rem;
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
</style>
