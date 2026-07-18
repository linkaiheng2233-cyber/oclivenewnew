<script setup lang="ts">
import { useChatStorageSettings } from '@oclive/shared/composables/useChatStorageSettings'
import { useI18n } from 'vue-i18n'
import UiButton from '../ui/UiButton.vue'
import ChatStorageMessageEditor from './chat-storage/ChatStorageMessageEditor.vue'
import ChatStorageReplayBar from './chat-storage/ChatStorageReplayBar.vue'
import ChatStorageSessionList from './chat-storage/ChatStorageSessionList.vue'
import ChatStorageStatsPanel from './chat-storage/ChatStorageStatsPanel.vue'

const { t } = useI18n()

const {
  level,
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
} = useChatStorageSettings()

defineExpose({ refreshStats })
</script>

<template>
  <section class="css-panel tool-mgmt-panel">
    <div class="css-head">
      <UiButton
        v-if="level !== 'roles'"
        size="sm"
        variant="ghost"
        @click="goBack"
      >
        {{ t('chatStorage.back') }}
      </UiButton>
      <h3 class="css-title">
        {{ t('chatStorage.title') }}
      </h3>
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="loading"
        @click="refreshStats(true)"
      >
        {{ t('chatStorage.refresh') }}
      </UiButton>
    </div>
    <p class="css-lead">
      {{ t('chatStorage.lead') }}
    </p>
    <p v-if="capabilitiesDegraded" class="css-degraded-hint text-amber-600 dark:text-amber-400 text-sm">
      {{ t('chatStorage.capabilitiesDegraded') }}
    </p>
    <p v-if="capabilities.backend_kind" class="css-backend-hint">
      {{ t('chatStorage.backendLabel', { backend: backendLabel }) }}
    </p>
    <div
      v-if="selectedRole && roleStorageConfig"
      class="css-location"
    >
      <span class="text-muted-foreground">{{ t('chatStorage.location') }}:</span>
      <span
        v-if="roleStorageConfig.location === 'role_pack'"
        class="css-location-badge css-location-badge--pack"
      >
        {{ t('chatStorage.followsRolePack') }}
      </span>
      <span
        v-else
        class="css-location-badge css-location-badge--global"
      >
        {{ t('chatStorage.globalLocation') }}
      </span>
    </div>

    <div class="css-root">
      <span class="css-root-label">{{ t('chatStorage.storageRoot') }}</span>
      <code class="css-root-path">{{ storageRoot || '…' }}</code>
      <UiButton size="sm" variant="secondary" class="css-root-btn" @click="changeStorageRoot">
        {{ t('chatStorage.changeRoot') }}
      </UiButton>
      <p class="css-muted css-root-hint">
        {{ t('chatStorage.storageRootHint') }}
      </p>
    </div>

    <div v-if="capabilities.supports_search" class="css-search">
      <input
        v-model="searchQuery"
        type="search"
        class="css-search-input"
        :placeholder="t('chatStorage.searchPlaceholder')"
        @keydown.enter="runSearch"
      >
      <UiButton size="sm" variant="primary" :disabled="loading" @click="runSearch">
        {{ t('chatStorage.search') }}
      </UiButton>
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

    <ChatStorageReplayBar :active="replayActive" :percent="replayProgress" />

    <ChatStorageStatsPanel
      v-if="level === 'roles'"
      :stats="stats"
      :role-names="roleNames"
      :loading="loading"
      :supports-cleanup="capabilities.supports_cleanup"
      :supports-replay="capabilities.supports_replay"
      @open-role="openRole"
      @export-role="handleExportRole"
      @cleanup="openCleanupSettings"
      @replay-role="(id) => startMemoryReplay('role', id)"
      @delete-role="confirmDeleteRole"
    />

    <ChatStorageSessionList
      v-else-if="level === 'scenes' || level === 'sessions'"
      :level="level === 'scenes' ? 'scenes' : 'sessions'"
      :selected-role="selectedRole"
      :selected-scene="selectedScene"
      :selected-role-label="selectedRoleLabel"
      :sessions="sessions"
      :supports-replay="capabilities.supports_replay"
      @open-scene="openScene"
      @open-session="openSession"
      @export-session="(id) => handleExportSession(id, 'markdown')"
      @replay-scene="(roleId, sceneId) => startMemoryReplay('scene', roleId, sceneId)"
      @delete-scene="confirmDeleteScene"
    />

    <ChatStorageMessageEditor
      v-else-if="level === 'messages' && selectedSession"
      :selected-session="selectedSession"
      :selected-role="selectedRole"
      :selected-scene="selectedScene"
      :messages="messages"
      :editing-message-id="editingMessageId"
      :editing-content="editingContent"
      :supports-replay="capabilities.supports_replay"
      @update:editing-content="editingContent = $event"
      @start-edit="startEditMessage"
      @cancel-edit="cancelEdit"
      @confirm-edit="confirmEditMessage"
      @delete-message="confirmDeleteMessage"
      @export-session="handleExportSession"
      @replay-session="(roleId, sceneId, sessionId) => startMemoryReplay('session', roleId, sceneId, sessionId)"
    />

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
.css-location {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
}
.css-location-badge {
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
}
.css-location-badge--pack {
  background: color-mix(in srgb, var(--oc-primary, #6366f1) 12%, transparent);
  color: var(--oc-primary, #6366f1);
}
.css-location-badge--global {
  background: var(--oc-muted-bg, rgba(128, 128, 128, 0.15));
  color: inherit;
}
.css-root {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-bottom: 0.75rem;
  font-size: 0.85rem;
}
.css-root-path {
  word-break: break-all;
  font-size: 0.8rem;
}
.css-root-hint {
  margin: 0;
}
.css-root-btn {
  align-self: flex-start;
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
.css-action {
  padding: 0 0.6rem;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font-size: 0.75rem;
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
