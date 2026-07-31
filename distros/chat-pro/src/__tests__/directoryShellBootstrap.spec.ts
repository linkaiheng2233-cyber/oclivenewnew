// @vitest-environment jsdom

import {
  isDirectoryShellDisabled,
  mountIsolatedDirectoryShell,
  parseDirectoryShellIdentity,
} from '@oclive/shared/utils/directoryShellBootstrap'
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

  it('derives bridge authority only from the canonical shell URL', () => {
    expect(parseDirectoryShellIdentity(
      'https://ocliveplugin.localhost/plugin.a/ui/index.html',
      'plugin.a',
    )).toEqual({ pluginId: 'plugin.a', assetRel: 'ui/index.html' })
    expect(parseDirectoryShellIdentity(
      'ocliveplugin://localhost/plugin.a/ui/index.html',
      'plugin.a',
    )).toEqual({ pluginId: 'plugin.a', assetRel: 'ui/index.html' })
    expect(parseDirectoryShellIdentity(
      'https://ocliveplugin.localhost/plugin.b/ui/index.html',
      'plugin.a',
    )).toBeNull()
    expect(parseDirectoryShellIdentity(
      'https://evil.test/plugin.a/ui/index.html',
      'plugin.a',
    )).toBeNull()
    expect(parseDirectoryShellIdentity(
      'https://ocliveplugin.localhost/plugin.a/%2e%2e/secrets',
      'plugin.a',
    )).toBeNull()
  })

  it('mounts full-shell HTML as an opaque-origin script-only frame', () => {
    document.body.innerHTML = '<div id="app"><p class="host-secret">host</p></div>'

    expect(mountIsolatedDirectoryShell(
      'https://ocliveplugin.localhost/plugin.a/ui/index.html',
      { pluginId: 'plugin.a', assetRel: 'ui/index.html' },
    )).toBe(true)

    const frame = document.querySelector<HTMLIFrameElement>('#oclive-directory-shell-frame')
    expect(frame).not.toBeNull()
    expect(frame?.getAttribute('sandbox')).toBe('allow-scripts')
    expect(frame?.referrerPolicy).toBe('no-referrer')
    expect(document.querySelector('.host-secret')).toBeNull()
    window.dispatchEvent(new Event('beforeunload'))
  })
})
