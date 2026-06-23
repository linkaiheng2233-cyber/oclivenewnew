import { describe, expect, it } from 'vitest'
import { resolveVisualAdapter } from './index'

describe('visual adapters', () => {
  it('resolves image by default', () => {
    expect(resolveVisualAdapter('image').kind).toBe('image')
  })

  it('resolves live2d adapter', () => {
    expect(resolveVisualAdapter('live2d').kind).toBe('live2d')
  })

  it('supports inner context mode param', () => {
    expect(resolveVisualAdapter('procedural', { mode: 'inner' }).kind).toBe('procedural')
  })
})
