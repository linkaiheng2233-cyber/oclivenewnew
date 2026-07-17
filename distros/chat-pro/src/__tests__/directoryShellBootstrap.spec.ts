import { isDirectoryShellDisabled } from '@oclive/shared/utils/directoryShellBootstrap'
import { isUnsafeInlinePluginVueEnabled } from '@oclive/shared/utils/vueComponentSecurity'
import { describe, expect, it, vi } from 'vitest'

describe('directoryShellBootstrap', () => {
  it('isDirectoryShellDisabled reads VITE_OCLIVE_DISABLE_DIRECTORY_SHELL', () => {
    vi.stubEnv('VITE_OCLIVE_DISABLE_DIRECTORY_SHELL', '1')
    expect(isDirectoryShellDisabled()).toBe(true)
    vi.unstubAllEnvs()
  })

  it('keeps same-process plugin Vue disabled without an explicit unsafe dev opt-in', () => {
    vi.stubEnv('VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE', '')
    expect(isUnsafeInlinePluginVueEnabled()).toBe(false)
    vi.stubEnv('VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE', '1')
    expect(isUnsafeInlinePluginVueEnabled()).toBe(import.meta.env.DEV)
    vi.unstubAllEnvs()
  })
})
