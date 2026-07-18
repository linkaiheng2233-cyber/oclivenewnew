import type { UnlistenFn } from '@tauri-apps/api/event'
import { importRolePack, peekRolePack } from '@oclive/shared/api'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

export interface RolePackNotifyPayload {
  type: 'success' | 'error' | 'info' | 'warning'
  message: string
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
    try {
      return await fn()
    }
    finally {
      unlistenProgress?.()
      unlistenProgress = null
      importProgressOpen.value = false
    }
  }

  function closeConflict(): void {
    conflictOpen.value = false
    pendingPath.value = null
    pendingPeek.value = null
  }

  async function finishImport(roleId: string, message: string): Promise<void> {
    await roleStore.loadRoles()
    await options?.onImported?.(roleId)
    notify({ type: 'success', message })
  }

  async function confirmOverwrite(): Promise<void> {
    const path = pendingPath.value
    if (!path) {
      closeConflict()
      return
    }
    if (importProgressOpen.value)
      return
    try {
      const roleId = await withImportProgress(() => importRolePack(path, true))
      await finishImport(roleId, t('common.rolePack.importedOverwrite', { id: roleId }))
    }
    catch (e) {
      notify({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
      })
    }
    finally {
      closeConflict()
    }
  }

  async function runImportFlow(path: string): Promise<string | null> {
    const peek = await peekRolePack(path)
    const exists = roleStore.roles.some(r => r.id === peek.id)
    if (exists) {
      pendingPath.value = path
      pendingPeek.value = peek
      conflictOpen.value = true
      return null
    }

    const roleId = await withImportProgress(() => importRolePack(path, false))
    await finishImport(roleId, t('common.rolePack.imported', { name: peek.name }))
    return roleId
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
    if (importProgressOpen.value)
      return null
    try {
      const path = await pickImportSource(mode)
      if (!path)
        return null
      return await runImportFlow(path)
    }
    catch (e) {
      notify({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
      })
      return null
    }
  }

  return {
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
    runImportFlow,
  }
}
