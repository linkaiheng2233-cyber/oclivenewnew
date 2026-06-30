export type OcliveShellKind = 'tool' | 'fluent' | 'theater'

/** Default: fluent. Set `VITE_OCLIVE_SHELL=tool` or `theater` for alternate shells. */
export function resolveOcliveShell(): OcliveShellKind {
  const raw = import.meta.env.VITE_OCLIVE_SHELL?.trim().toLowerCase()
  if (raw === 'tool')
    return 'tool'
  if (raw === 'theater')
    return 'theater'
  return 'fluent'
}
