import { open } from '@tauri-apps/api/shell'

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

/**
 * Open pack-editor for a role pack. Set `VITE_OCLIVE_PACK_EDITOR` to the editor executable or dev URL.
 */
export async function openPackEditorForRole(roleId: string): Promise<{ ok: boolean, message?: string }> {
  const editorPath = import.meta.env.VITE_OCLIVE_PACK_EDITOR?.trim()
  const rolesHint = import.meta.env.VITE_OCLIVE_ROLES_DIR?.trim()

  if (editorPath && isTauri()) {
    try {
      const target = editorPath.includes('?')
        ? `${editorPath}&role=${encodeURIComponent(roleId)}`
        : `${editorPath}?role=${encodeURIComponent(roleId)}`
      await open(target)
      return { ok: true }
    }
    catch (e) {
      return { ok: false, message: e instanceof Error ? e.message : String(e) }
    }
  }

  if (rolesHint && isTauri()) {
    try {
      const folder = `${rolesHint.replace(/\\/g, '/').replace(/\/+$/, '')}/${roleId}`
      await open(folder)
      return { ok: true, message: folder }
    }
    catch (e) {
      return { ok: false, message: e instanceof Error ? e.message : String(e) }
    }
  }

  return {
    ok: false,
    message: `Configure VITE_OCLIVE_PACK_EDITOR or VITE_OCLIVE_ROLES_DIR, then open role "${roleId}" in oclive-pack-editor.`,
  }
}
