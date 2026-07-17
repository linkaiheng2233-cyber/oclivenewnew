export interface ManualChunkRule {
  pattern: RegExp
  chunk: string
}

/** Order matters: more specific patterns before broader matches. */
export const MANUAL_CHUNK_RULES: ManualChunkRule[] = [
  { pattern: /@sentry/, chunk: 'vendor-sentry' },
  { pattern: /@tauri-apps/, chunk: 'vendor-tauri' },
  { pattern: /vue-i18n/, chunk: 'vendor-i18n' },
  { pattern: /pinia-plugin-persistedstate/, chunk: 'vendor-pinia-persist' },
  { pattern: /pinia/, chunk: 'vendor-pinia' },
  { pattern: /acorn/, chunk: 'vendor-acorn' },
  { pattern: /idb-keyval/, chunk: 'vendor-idb' },
  { pattern: /\/vue\/|@vue\//, chunk: 'vendor-vue' },
]

export function resolveManualChunk(id: string): string | undefined {
  if (!id.includes('node_modules'))
    return undefined
  for (const rule of MANUAL_CHUNK_RULES) {
    if (rule.pattern.test(id))
      return rule.chunk
  }
  return 'vendor-misc'
}
