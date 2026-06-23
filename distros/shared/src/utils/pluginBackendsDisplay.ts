import type { DirectoryPluginSlots, PluginBackends } from '@oclive/shared/api'

const SLOT_KEYS: (keyof DirectoryPluginSlots)[] = [
  'memory',
  'emotion',
  'event',
  'prompt',
  'llm',
  'agent',
]

/** Format `directory_plugins` slots as one-line debug text; returns `none` when all empty. */
export function formatDirectoryPluginSlots(
  slots: DirectoryPluginSlots | undefined | null,
): string {
  if (!slots)
    return 'none'
  const parts: string[] = []
  for (const k of SLOT_KEYS) {
    const raw = slots[k]
    const v = typeof raw === 'string' ? raw.trim() : ''
    if (v)
      parts.push(`${k}=${v}`)
  }
  return parts.length ? parts.join(', ') : 'none'
}

/** True when any module uses `directory` or a slot is non-empty; controls "directory plugin" row visibility. */
export function usesDirectoryPlugins(pb: PluginBackends): boolean {
  if (
    pb.memory === 'directory'
    || pb.emotion === 'directory'
    || pb.event === 'directory'
    || pb.prompt === 'directory'
    || pb.llm === 'directory'
    || pb.agent === 'directory'
  ) {
    return true
  }
  return formatDirectoryPluginSlots(pb.directory_plugins) !== 'none'
}
