export type OcliveShellKind = 'tool' | 'fluent'

/** Default: tool. Set `VITE_OCLIVE_SHELL=fluent` for alternate shell. */
export function resolveOcliveShell(): OcliveShellKind {
  const raw = import.meta.env.VITE_OCLIVE_SHELL?.trim().toLowerCase()
  if (raw === 'fluent') {
    return 'fluent'
  }
  return 'tool'
}
