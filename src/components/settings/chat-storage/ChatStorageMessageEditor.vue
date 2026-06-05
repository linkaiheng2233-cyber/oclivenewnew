<script setup lang="ts">
import type { RoleStorageStat, SceneStorageStat, SessionMeta, StoredMessage } from '../../../api/chatStorage'
import { useI18n } from 'vue-i18n'

defineProps<{
  selectedSession: SessionMeta
  selectedRole: RoleStorageStat | null
  selectedScene: SceneStorageStat | null
  messages: StoredMessage[]
  editingMessageId: string | null
  editingContent: string
  supportsReplay: boolean
}>()

const emit = defineEmits<{
  'update:editingContent': [value: string]
  'startEdit': [msg: StoredMessage]
  'cancelEdit': []
  'confirmEdit': []
  'deleteMessage': [msg: StoredMessage]
  'exportSession': [sessionId: string, format: 'markdown' | 'json']
  'replaySession': [roleId: string, sceneId: string | undefined, sessionId: string]
}>()

const { t } = useI18n()
</script>

<template>
  <ul class="css-list css-messages">
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
        <textarea
          :value="editingContent"
          class="css-edit-area"
          rows="3"
          @input="emit('update:editingContent', ($event.target as HTMLTextAreaElement).value)"
        />
        <div class="css-msg-actions">
          <button type="button" class="css-action" @click="emit('confirmEdit')">
            {{ t('chatStorage.save') }}
          </button>
          <button type="button" class="css-action" @click="emit('cancelEdit')">
            {{ t('chatStorage.cancel') }}
          </button>
        </div>
      </template>
      <template v-else>
        <p class="css-msg-body">
          {{ msg.content }}
        </p>
        <div v-if="msg.sender === 'user'" class="css-msg-actions">
          <button type="button" class="css-action" @click="emit('startEdit', msg)">
            {{ t('chatStorage.edit') }}
          </button>
          <button type="button" class="css-danger-inline" @click="emit('deleteMessage', msg)">
            {{ t('chatStorage.delete') }}
          </button>
        </div>
      </template>
    </li>
    <li class="css-export-row css-row">
      <button type="button" class="css-action" @click="emit('exportSession', selectedSession.session_id, 'markdown')">
        {{ t('chatStorage.exportMd') }}
      </button>
      <button type="button" class="css-action" @click="emit('exportSession', selectedSession.session_id, 'json')">
        {{ t('chatStorage.exportJson') }}
      </button>
      <button
        v-if="supportsReplay && selectedRole"
        type="button"
        class="css-action"
        @click="emit('replaySession', selectedRole.role_id, selectedScene?.scene_id, selectedSession.session_id)"
      >
        {{ t('chatStorage.replayMemory') }}
      </button>
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
  display: flex;
}
.css-action {
  padding: 0 0.6rem;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font-size: 0.75rem;
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
</style>
