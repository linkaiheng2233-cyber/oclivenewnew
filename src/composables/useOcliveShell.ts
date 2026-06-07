export type OcliveShellKind = 'tool' | 'fluent'

/** Default: tool. Set `VITE_OCLIVE_SHELL=fluent` to keep the legacy Fluent shell. */
export function resolveOcliveShell(): OcliveShellKind {
  return import.meta.env.VITE_OCLIVE_SHELL === 'fluent' ? 'fluent' : 'tool'
}
