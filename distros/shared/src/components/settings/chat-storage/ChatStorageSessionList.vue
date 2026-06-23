<script setup lang="ts">
import type {
  RoleStorageStat,
  SceneStorageStat,
  SessionMeta,
} from '@oclive/shared/api/chatStorage'
import { useI18n } from 'vue-i18n'
import { formatStorageBytes } from './chatStorageFormat'

defineProps<{
  level: 'scenes' | 'sessions'
  selectedRole: RoleStorageStat | null
  selectedScene: SceneStorageStat | null
  selectedRoleLabel: string
  sessions: SessionMeta[]
  supportsReplay: boolean
}>()

const emit = defineEmits<{
  openScene: [scene: SceneStorageStat]
  openSession: [session: SessionMeta]
  exportSession: [sessionId: string]
  replayScene: [roleId: string, sceneId: string]
  deleteScene: [roleId: string, sceneId: string]
}>()

const { t } = useI18n()
</script>

<template>
  <ul v-if="level === 'scenes' && selectedRole" class="css-list">
    <li
      v-for="scene in selectedRole.scenes"
      :key="scene.scene_id"
      class="css-row"
    >
      <button type="button" class="css-row-main" @click="emit('openScene', scene)">
        <span class="css-row-title">{{ scene.scene_id }}</span>
        <span class="css-row-meta">
          {{ t('chatStorage.sessionsCount', { n: scene.session_count }) }}
          · {{ formatStorageBytes(scene.total_size_bytes) }}
        </span>
      </button>
      <button
        v-if="supportsReplay"
        type="button"
        class="css-action"
        @click.stop="emit('replayScene', selectedRole.role_id, scene.scene_id)"
      >
        {{ t('chatStorage.replayMemory') }}
      </button>
      <button
        type="button"
        class="css-danger"
        @click.stop="emit('deleteScene', selectedRole.role_id, scene.scene_id)"
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
      <button type="button" class="css-row-main" @click="emit('openSession', sess)">
        <span class="css-row-title">{{ sess.session_id.slice(0, 24) }}…</span>
        <span class="css-row-meta">{{ sess.last_message_snippet || t('chatStorage.emptyPreview') }}</span>
      </button>
      <div class="css-actions">
        <button type="button" class="css-action" @click.stop="emit('exportSession', sess.session_id)">
          {{ t('chatStorage.export') }}
        </button>
      </div>
    </li>
    <li v-if="sessions.length === 0" class="css-muted">
      {{ t('chatStorage.empty') }}
    </li>
  </ul>
</template>

<style scoped>
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
.css-breadcrumb {
  padding: 0.25rem 0.5rem;
  font-size: 0.8rem;
  opacity: 0.8;
}
.css-muted {
  opacity: 0.7;
  font-size: 0.875rem;
}
</style>
