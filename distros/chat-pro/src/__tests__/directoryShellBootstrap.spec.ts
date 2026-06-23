import { describe, expect, it, vi } from 'vitest'
import { isDirectoryShellDisabled } from '@oclive/shared/utils/directoryShellBootstrap'

describe('directoryShellBootstrap', () => {
  it('isDirectoryShellDisabled reads VITE_OCLIVE_DISABLE_DIRECTORY_SHELL', () => {
    vi.stubEnv('VITE_OCLIVE_DISABLE_DIRECTORY_SHELL', '1')
    expect(isDirectoryShellDisabled()).toBe(true)
    vi.unstubAllEnvs()
  })
})
