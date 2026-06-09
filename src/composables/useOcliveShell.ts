export type OcliveShellKind = 'tool' | 'fluent' | 'theater'

/** Default: tool. Set `VITE_OCLIVE_SHELL=fluent|theater` for alternate shells. */
export function resolveOcliveShell(): OcliveShellKind {
  const raw = import.meta.env.VITE_OCLIVE_SHELL?.trim().toLowerCase()
  if (raw === 'fluent') {
    return 'fluent'
  }
  if (raw === 'theater') {
    return 'theater'
  }
  return 'tool'
}

/** Optional runtime override when attached kernel reports `distro_id=theater`. */
export function resolveShellFromDistroId(distroId: string | null | undefined): OcliveShellKind | null {
  if (distroId?.trim().toLowerCase() === 'theater') {
    return 'theater'
  }
  return null
}
