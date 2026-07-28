import type { DirectoryPluginBootstrap } from '@oclive/shared/api'
import { pluginBridgeInvoke, readPluginAssetText } from '@oclive/shared/api'
import DirectoryShellApp from '@oclive/shared/components/DirectoryShellApp.vue'
import { i18n } from '@oclive/shared/i18n/index'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { createPluginFrameBridge } from '@oclive/shared/utils/pluginFrameBridge'
import { isUnsafeInlinePluginVueEnabled } from '@oclive/shared/utils/vueComponentSecurity'
import { invoke } from '@tauri-apps/api/core'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined'
    && Object.hasOwn(window, '__TAURI_INTERNALS__')
  )
}

/** Dev/troubleshooting: skip full-shell directory plugin and force main UI (set `VITE_OCLIVE_DISABLE_DIRECTORY_SHELL=1` in `.env`). */
export function isDirectoryShellDisabled(): boolean {
  return import.meta.env.VITE_OCLIVE_DISABLE_DIRECTORY_SHELL === '1'
}

export interface DirectoryShellIdentity {
  pluginId: string
  assetRel: string
}

/**
 * Resolve the immutable bridge identity from the canonical custom-protocol URL.
 * The browser frame never gets to supply either value.
 */
export function parseDirectoryShellIdentity(
  shellUrl: string,
  expectedPluginId: string,
): DirectoryShellIdentity | null {
  try {
    const url = new URL(shellUrl)
    const isNativeCustomProtocol
      = url.protocol === 'ocliveplugin:' && url.hostname === 'localhost'
    const isMappedHttpsProtocol
      = url.protocol === 'https:' && url.hostname === 'ocliveplugin.localhost'
    if (!isNativeCustomProtocol && !isMappedHttpsProtocol)
      return null
    const segments = url.pathname.split('/').filter(Boolean).map((segment) => {
      const decoded = decodeURIComponent(segment)
      if (!decoded || decoded === '.' || decoded === '..' || decoded.includes('/') || decoded.includes('\\'))
        throw new Error('invalid plugin shell path segment')
      return decoded
    })
    const [pluginId, ...assetSegments] = segments
    if (pluginId !== expectedPluginId || assetSegments.length === 0)
      return null
    return { pluginId, assetRel: assetSegments.join('/') }
  }
  catch {
    return null
  }
}

/**
 * Install a full-viewport opaque-origin frame. Tauri initialization scripts are
 * main-frame-only, so all host authority stays in this parent-side broker.
 */
export function mountIsolatedDirectoryShell(
  shellUrl: string,
  identity: DirectoryShellIdentity,
): boolean {
  const root = document.getElementById('app')
  if (!root)
    return false

  const broker = createPluginFrameBridge(pluginBridgeInvoke, {
    emit: (event, data) => hostEventBus.emit(event, data),
    subscribe: (event, handler) => {
      hostEventBus.on(event, handler)
      return () => hostEventBus.off(event, handler)
    },
  })
  const frame = document.createElement('iframe')
  frame.id = 'oclive-directory-shell-frame'
  frame.src = shellUrl
  frame.title = `plugin shell ${identity.pluginId}`
  frame.setAttribute('sandbox', 'allow-scripts')
  frame.referrerPolicy = 'no-referrer'
  frame.style.cssText = 'position:fixed;inset:0;width:100%;height:100%;border:0;background:#fff;z-index:2147483647'

  window.addEventListener('message', broker.handleMessage)
  root.replaceChildren(frame)
  const source = frame.contentWindow
  if (!source) {
    window.removeEventListener('message', broker.handleMessage)
    broker.dispose()
    frame.remove()
    return false
  }
  const registration = broker.register(source, identity)
  frame.addEventListener('load', () => {
    if (!registration.activate())
      console.warn('[oclive] directory shell navigation revoked bridge authority')
  })
  window.addEventListener('beforeunload', () => {
    registration.unregister()
    broker.dispose()
  }, { once: true })
  return true
}

/**
 * When a full-shell directory plugin is configured, release builds mount its
 * HTML in an opaque-origin full-viewport frame. Same-process `shell.vueEntry`
 * mounting is unsafe DEV-only.
 *
 * @returns true if full shell was handled (Vue mounted or HTML navigation started); caller must not mount app root.
 */
export async function tryReplaceWithDirectoryShell(): Promise<boolean> {
  if (!isTauriRuntime() || isDirectoryShellDisabled())
    return false
  try {
    const boot = await invoke<DirectoryPluginBootstrap>('get_directory_plugin_bootstrap', {
      roleId: null,
    })
    const shellUrl
      = typeof boot?.shellUrl === 'string' && boot.shellUrl.length > 0
        ? boot.shellUrl
        : null
    const shellPid
      = typeof boot?.shellPluginId === 'string' && boot.shellPluginId.trim().length > 0
        ? boot.shellPluginId.trim()
        : null
    if (!shellUrl || !shellPid) {
      return false
    }

    // A directory shell's Vue source would otherwise execute with the host
    // page's full authority. Production always navigates to the constrained
    // custom-protocol HTML surface; inline Vue is an explicit dev-only escape.
    const forceIframe
      = boot.forceIframeMode === true || !isUnsafeInlinePluginVueEnabled()
    const vueEntry
      = typeof boot.shellVueEntry === 'string' ? boot.shellVueEntry.trim() : ''

    if (!forceIframe && vueEntry.length > 0) {
      try {
        await readPluginAssetText(shellPid, vueEntry)
      }
      catch (e) {
        console.warn(
          '[oclive] directory shell vue entry unreadable; falling back to main app',
          { shellPid, vueEntry, error: e },
        )
        return false
      }
      const pinia = createPinia()
      const app = createApp(DirectoryShellApp, {
        pluginId: shellPid,
        vueEntry,
        bridgeAssetRel: vueEntry.replace(/\\/g, '/'),
        htmlFallbackUrl: shellUrl,
        developerMode: boot.developerMode === true,
      })
      app.use(pinia)
      app.use(i18n)
      app.mount('#app')
      return true
    }

    const identity = parseDirectoryShellIdentity(shellUrl, shellPid)
    if (!identity) {
      console.warn('[oclive] directory shell URL identity rejected', shellUrl)
      return false
    }
    try {
      await readPluginAssetText(identity.pluginId, identity.assetRel)
    }
    catch (e) {
      console.warn(
        '[oclive] directory shell HTML unreadable; falling back to main app',
        { shellPid, assetRel: identity.assetRel, error: e },
      )
      return false
    }
    return mountIsolatedDirectoryShell(shellUrl, identity)
  }
  catch (e) {
    console.warn('[oclive] directory shell bootstrap skipped', e)
  }
  return false
}
