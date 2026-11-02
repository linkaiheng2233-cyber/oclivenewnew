<script setup lang="ts">
import { exportRolePack } from '@oclive/shared/api'
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { useRolePackImport } from '@oclive/shared/composables/useRolePackImport'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { save } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ImportProgressModal from './ImportProgressModal.vue'

const props = withDefaults(defineProps<{ canExport?: boolean, disabled?: boolean }>(), { canExport: true, disabled: false })
const emit = defineEmits<{
  notify: [payload: { type: 'success' | 'error' | 'info' | 'warning', message: string, details?: string }]
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
  importBusy,
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

const conflictCancelRef = ref<HTMLButtonElement | null>(null)
const conflictCardRef = ref<HTMLElement | null>(null)

useModalFocusRestore(conflictOpen, conflictCardRef, {
  primary: conflictCancelRef,
  trapFocus: true,
})

async function onExport(): Promise<void> {
  if (props.disabled || importBusy.value || conflictOpen.value || !props.canExport || !roleStore.currentRoleId)
    return
  const roleId = roleStore.currentRoleId
  try {
    const path = await save({
      filters: [{ name: t('common.rolePack.exportFilterName'), extensions: ['ocpak'] }],
      defaultPath: defaultExportFilename(),
    })
    if (!path || typeof path !== 'string')
      return
    await exportRolePack(roleId, path)
    emit('notify', { type: 'success', message: t('common.rolePack.exported') })
  }
  catch (e) {
    const failure = toErrorPresentation(e)
    emit('notify', {
      type: 'error',
      message: failure.summary,
      details: failure.details,
    })
  }
}

function onImport(): void {
  if (!props.disabled)
    void runImportWithPicker('archive')
}

function onImportFolder(): void {
  if (!props.disabled)
    void runImportWithPicker('folder')
}
</script>

<template>
  <div
    class="pack-bar"
    :title="t('common.rolePack.barTitle')"
  >
    <button v-if="canExport" type="button" class="btn" :disabled="disabled || importBusy || conflictOpen" @click="onExport">
      {{ t("common.rolePack.export") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="disabled || importBusy || conflictOpen"
      @click="onImport"
    >
      {{ t("common.rolePack.importArchive") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="disabled || importBusy || conflictOpen"
      @click="onImportFolder"
    >
      {{ t("common.rolePack.importFolder") }}
    </button>

    <span v-if="importBusy && !importProgressOpen && !conflictOpen" class="pack-busy" role="status">
      {{ t('common.preparing') }}
    </span>

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
              ref="conflictCancelRef"
              type="button"
              class="btn btn-ghost"
              :disabled="importBusy"
              @click="closeConflict"
            >
              {{ t("common.cancel") }}
            </button>
            <button
              type="button"
              class="btn btn-danger"
              :disabled="importBusy"
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
.pack-busy {
  align-self: center;
  font-size: 11px;
  color: var(--text-secondary);
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
  z-index: 10070;
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
