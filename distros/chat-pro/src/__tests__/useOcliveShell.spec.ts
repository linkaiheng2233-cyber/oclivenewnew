import { describe, expect, it } from 'vitest'
import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'

describe('resolveOcliveShell', () => {
  it('defaults to fluent when env unset', () => {
    expect(resolveOcliveShell()).toBe('fluent')
  })
})
