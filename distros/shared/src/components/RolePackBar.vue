<script setup lang="ts">
import { save } from '@tauri-apps/api/dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { useRolePackImport } from '@oclive/shared/composables/useRolePackImport'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { exportRolePack } from '@oclive/shared/api'
import ImportProgressModal from './ImportProgressModal.vue'

const emit = defineEmits<{
  notify: [payload: { type: 'success' | 'error' | 'info' | 'warning', message: string }]
  imported: [roleId: string]
}>()
const { t } = useI18n()
const roleStore = useRoleStore()

/** Windows / cross-platform illegal filename characters */
function safeFileSegment(s: string): string {
  const seg = s.replace(/[<>:"/\\|?*\x00-\x1F]/g, '_').trim()
  return seg.length > 0 ? seg.slice(0, 80) : 'role'
}

function defaultExportFilename(): string {
  const name = safeFileSegment(roleStore.roleInfo.name || roleStore.currentRoleId)
  const ver = safeFileSegment(roleStore.roleInfo.version || '0')
  return `${name}_${ver}.ocpak`
}

const {
  conflictOpen,
  pendingPeek,
  importProgressOpen,
  importPercent,
  importMessage,
  importFileIndex,
  importFileTotal,
  importCurrentFile,
  closeConflict,
  confirmOverwrite,
  runImportWithPicker,
} = useRolePackImport({
  onImported: roleId => emit('imported', roleId),
  onNotify: payload => emit('notify', payload),
})

const conflictPrimaryRef = ref<HTMLButtonElement | null>(null)
const conflictCardRef = ref<HTMLElement | null>(null)

useModalFocusRestore(conflictOpen, conflictCardRef, {
  primary: conflictPrimaryRef,
})

async function onExport(): Promise<void> {
  try {
    const path = await save({
      filters: [{ name: t('common.rolePack.exportFilterName'), extensions: ['ocpak'] }],
      defaultPath: defaultExportFilename(),
    })
    if (!path || typeof path !== 'string')
      return
    await exportRolePack(roleStore.currentRoleId, path)
    emit('notify', { type: 'success', message: t('common.rolePack.exported') })
  }
  catch (e) {
    emit('notify', {
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
    })
  }
}

function onImport(): void {
  void runImportWithPicker('archive')
}

function onImportFolder(): void {
  void runImportWithPicker('folder')
}
</script>

<template>
  <div
    class="pack-bar"
    :title="t('common.rolePack.barTitle')"
  >
    <button type="button" class="btn" @click="onExport">
      {{ t("common.rolePack.export") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImport"
    >
      {{ t("common.rolePack.importArchive") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImportFolder"
    >
      {{ t("common.rolePack.importFolder") }}
    </button>

    <ImportProgressModal
      :open="importProgressOpen"
      :percent="importPercent"
      :message="importMessage"
      :file-index="importFileIndex"
      :file-total="importFileTotal"
      :current-file="importCurrentFile"
    />

    <Teleport to="body">
      <div
        v-if="conflictOpen && pendingPeek"
        class="modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="pack-conflict-title"
      >
        <div ref="conflictCardRef" class="modal-card" tabindex="-1" @click.stop @keydown.escape.stop="closeConflict">
          <h2 id="pack-conflict-title" class="modal-title">
            {{ t("common.rolePack.conflictTitle") }}
          </h2>
          <p class="modal-body">
            {{
              t("common.rolePack.conflictBody", {
                id: pendingPeek.id,
                name: pendingPeek.name,
                version: pendingPeek.version,
              })
            }}
          </p>
          <div class="modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              :disabled="importProgressOpen"
              @click="closeConflict"
            >
              {{ t("common.cancel") }}
            </button>
            <button
              ref="conflictPrimaryRef"
              type="button"
              class="btn btn-danger"
              :disabled="importProgressOpen"
              @click="confirmOverwrite"
            >
              {{ t("common.rolePack.overwrite") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.pack-bar {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.btn {
  font-size: 11px;
  padding: 4px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--btn-grad-a);
  color: var(--text-secondary);
  cursor: pointer;
}
.btn:hover {
  background: var(--btn-primary-hover-a);
  color: var(--text-primary);
}
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.45));
}
.modal-card {
  max-width: 400px;
  width: 100%;
  padding: 20px;
  border-radius: 12px;
  background: var(--bg-panel, #1a1a22);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.modal-title {
  margin: 0 0 12px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}
.modal-body {
  margin: 0 0 18px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
.btn-ghost {
  background: transparent;
}
.btn-danger {
  border-color: #c45c5c;
  background: linear-gradient(180deg, #a04040, #802828);
  color: #fff;
}
.btn-danger:hover {
  filter: brightness(1.08);
}
</style>
