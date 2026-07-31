import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'
// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest'

describe('resolveOcliveShell', () => {
  afterEach(() => {
    localStorage.removeItem('oclive-runtime-skin')
    localStorage.removeItem('oclive-easteregg-unlocked')
    document.documentElement.removeAttribute('data-skin')
  })

  it('defaults to fluent when env unset', () => {
    expect(resolveOcliveShell()).toBe('fluent')
  })
})
