<script setup lang="ts">
import type { RoleStorageStat } from '../../../api/chatStorage'
import { useI18n } from 'vue-i18n'
import { formatStorageBytes } from './chatStorageFormat'

defineProps<{
  stats: RoleStorageStat[]
  roleNames: Record<string, string>
  loading: boolean
  supportsCleanup: boolean
  supportsReplay: boolean
}>()

const emit = defineEmits<{
  openRole: [row: RoleStorageStat]
  exportRole: [roleId: string, format: 'markdown' | 'json']
  cleanup: [roleId: string]
  replayRole: [roleId: string]
  deleteRole: [roleId: string]
}>()

const { t } = useI18n()
</script>

<template>
  <div v-if="loading" class="css-muted">
    {{ t('chatStorage.loading') }}
  </div>
  <ul v-else class="css-list">
    <li
      v-for="row in stats"
      :key="row.role_id"
      class="css-row"
    >
      <button type="button" class="css-row-main" @click="emit('openRole', row)">
        <span class="css-row-title">{{ roleNames[row.role_id] ?? row.role_id }}</span>
        <span class="css-row-meta">
          {{ t('chatStorage.scenesCount', { n: row.scene_count }) }}
          · {{ formatStorageBytes(row.total_size_bytes) }}
          <template v-if="row.last_active">
            · {{ row.last_active }}
          </template>
        </span>
      </button>
      <div class="css-actions">
        <button type="button" class="css-action" @click.stop="emit('exportRole', row.role_id, 'markdown')">
          {{ t('chatStorage.exportMd') }}
        </button>
        <button type="button" class="css-action" @click.stop="emit('exportRole', row.role_id, 'json')">
          {{ t('chatStorage.exportJson') }}
        </button>
        <button
          v-if="supportsCleanup"
          type="button"
          class="css-action"
          @click.stop="emit('cleanup', row.role_id)"
        >
          {{ t('chatStorage.autoCleanup') }}
        </button>
        <button
          v-if="supportsReplay"
          type="button"
          class="css-action"
          @click.stop="emit('replayRole', row.role_id)"
        >
          {{ t('chatStorage.replayMemory') }}
        </button>
        <button type="button" class="css-danger" @click.stop="emit('deleteRole', row.role_id)">
          {{ t('chatStorage.delete') }}
        </button>
      </div>
    </li>
    <li v-if="stats.length === 0" class="css-muted">
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
.css-muted {
  opacity: 0.7;
  font-size: 0.875rem;
}
</style>
