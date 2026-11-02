import type { UnlistenFn } from '@tauri-apps/api/event'
import { importRolePack, peekRolePack } from '@oclive/shared/api'
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

export interface RolePackNotifyPayload {
  type: 'success' | 'error' | 'info' | 'warning'
  message: string
  details?: string
}

export function useRolePackImport(options?: {
  onImported?: (roleId: string) => void | Promise<void>
  onNotify?: (payload: RolePackNotifyPayload) => void
}) {
  const { t } = useI18n()
  const roleStore = useRoleStore()

  const conflictOpen = ref(false)
  const pendingPath = ref<string | null>(null)
  const pendingPeek = ref<{ id: string, name: string, version: string } | null>(null)
  const importBusy = ref(false)

  const importProgressOpen = ref(false)
  const importPercent = ref(0)
  const importMessage = ref('')
  const importFileIndex = ref<number | null>(null)
  const importFileTotal = ref<number | null>(null)
  const importCurrentFile = ref<string | null>(null)
  let unlistenProgress: UnlistenFn | null = null

  function notify(payload: RolePackNotifyPayload) {
    options?.onNotify?.(payload)
  }

  async function withImportProgress<T>(fn: () => Promise<T>): Promise<T> {
    importProgressOpen.value = true
    importPercent.value = 0
    importMessage.value = t('common.preparing')
    importFileIndex.value = null
    importFileTotal.value = null
    importCurrentFile.value = null
    try {
      unlistenProgress = await listen<{
        percent: number
        message: string
        fileIndex?: number
        fileTotal?: number
        currentFile?: string
      }>('import_progress', (e) => {
        importPercent.value = e.payload.percent
        importMessage.value = e.payload.message
        importFileIndex.value
          = typeof e.payload.fileIndex === 'number' ? e.payload.fileIndex : null
        importFileTotal.value
          = typeof e.payload.fileTotal === 'number' ? e.payload.fileTotal : null
        importCurrentFile.value
          = typeof e.payload.currentFile === 'string' ? e.payload.currentFile : null
      })
      return await fn()
    }
    finally {
      unlistenProgress?.()
      unlistenProgress = null
      importProgressOpen.value = false
    }
  }

  function closeConflict(): void {
    if (importBusy.value)
      return
    conflictOpen.value = false
    pendingPath.value = null
    pendingPeek.value = null
  }

  async function finishImport(roleId: string, message: string): Promise<void> {
    try {
      await roleStore.loadRoles()
      await options?.onImported?.(roleId)
      notify({ type: 'success', message })
    }
    catch (err) {
      const failure = toErrorPresentation(err, t('errorUi.importRefreshFailed'))
      notify({ type: 'warning', message: failure.summary, details: failure.details })
    }
  }

  async function installPack(path: string, overwrite: boolean, message: (roleId: string) => string): Promise<string> {
    return withImportProgress(async () => {
      const roleId = await importRolePack(path, overwrite)
      await finishImport(roleId, message(roleId))
      return roleId
    })
  }

  async function confirmOverwrite(): Promise<void> {
    if (importBusy.value)
      return
    const path = pendingPath.value
    if (!path) {
      closeConflict()
      return
    }
    importBusy.value = true
    try {
      await installPack(path, true, roleId => t('common.rolePack.importedOverwrite', { id: roleId }))
    }
    catch (e) {
      const failure = toErrorPresentation(e)
      notify({ type: 'error', message: failure.summary, details: failure.details })
    }
    finally {
      importBusy.value = false
      closeConflict()
    }
  }

  async function importFromPath(path: string): Promise<string | null> {
    const peek = await peekRolePack(path)
    const exists = roleStore.roles.some(r => r.id === peek.id)
    if (exists) {
      pendingPath.value = path
      pendingPeek.value = peek
      // Waiting for consent is not background work. Enable Cancel before
      // the dialog's focus restoration chooses its initial control.
      importBusy.value = false
      conflictOpen.value = true
      return null
    }

    return installPack(path, false, () => t('common.rolePack.imported', { name: peek.name }))
  }

  async function runImportFlow(path: string): Promise<string | null> {
    if (importBusy.value || conflictOpen.value)
      return null
    importBusy.value = true
    try {
      return await importFromPath(path)
    }
    finally {
      importBusy.value = false
    }
  }

  async function pickImportSource(mode: 'archive' | 'folder'): Promise<string | null> {
    const path = await open(
      mode === 'folder'
        ? { directory: true, multiple: false }
        : {
            filters: [{ name: t('common.rolePack.importFilterName'), extensions: ['ocpak', 'zip'] }],
            multiple: false,
            directory: false,
          },
    )
    if (path === null || Array.isArray(path))
      return null
    return path
  }

  async function runImportWithPicker(mode: 'archive' | 'folder'): Promise<string | null> {
    if (importBusy.value || conflictOpen.value)
      return null
    importBusy.value = true
    try {
      const path = await pickImportSource(mode)
      if (!path)
        return null
      return await importFromPath(path)
    }
    catch (e) {
      const failure = toErrorPresentation(e)
      notify({ type: 'error', message: failure.summary, details: failure.details })
      return null
    }
    finally {
      importBusy.value = false
    }
  }

  return {
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
    runImportFlow,
  }
}
