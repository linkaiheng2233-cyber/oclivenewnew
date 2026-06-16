export type OcliveShellKind = 'tool' | 'fluent' | 'theater'

/** Default: tool. Set `VITE_OCLIVE_SHELL=fluent` or `theater` for alternate shells. */
export function resolveOcliveShell(): OcliveShellKind {
  const raw = import.meta.env.VITE_OCLIVE_SHELL?.trim().toLowerCase()
  if (raw === 'fluent')
    return 'fluent'
  if (raw === 'theater')
    return 'theater'
  return 'tool'
}
