import { describe, expect, it } from 'vitest'
import { resolveManualChunk } from './manualChunks'

describe('resolveManualChunk', () => {
  it('routes vue-i18n before generic vue paths', () => {
    expect(resolveManualChunk('/node_modules/vue-i18n/dist/vue-i18n.mjs')).toBe('vendor-i18n')
  })

  it('routes @vue-flow before vendor-vue', () => {
    expect(resolveManualChunk('/node_modules/@vue-flow/core/dist/index.js')).toBe('vendor-vue-flow')
  })

  it('routes pinia-plugin-persistedstate before pinia', () => {
    expect(resolveManualChunk('/node_modules/pinia-plugin-persistedstate/dist/index.js')).toBe(
      'vendor-pinia-persist',
    )
  })

  it('returns undefined for app source', () => {
    expect(resolveManualChunk('/src/App.vue')).toBeUndefined()
  })

  it('falls back to vendor-misc for unknown node_modules', () => {
    expect(resolveManualChunk('/node_modules/lodash-es/lodash.js')).toBe('vendor-misc')
  })
})
