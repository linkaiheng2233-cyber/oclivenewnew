import type { DirectoryPluginBootstrap } from '@oclive/shared/api'
import { readPluginAssetText } from '@oclive/shared/api'
import DirectoryShellApp from '@oclive/shared/components/DirectoryShellApp.vue'
import { i18n } from '@oclive/shared/i18n/index'
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

/**
 * When a full-shell directory plugin is configured, release builds navigate to
 * `shellUrl`. Same-process `shell.vueEntry` mounting is unsafe DEV-only.
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

    async function shellHtmlReachable(url: string): Promise<boolean> {
      try {
        const r = await fetch(url, { method: 'GET', cache: 'no-store' })
        return r.ok
      }
      catch {
        return false
      }
    }

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

    const here = window.location.href.split('#')[0]
    const target = shellUrl.split('#')[0]
    if (here !== target) {
      const ok = await shellHtmlReachable(shellUrl)
      if (!ok) {
        console.warn(
          '[oclive] directory shell HTML unreachable; falling back to main app',
          shellUrl,
        )
        return false
      }
      window.location.replace(shellUrl)
      return true
    }
  }
  catch (e) {
    console.warn('[oclive] directory shell bootstrap skipped', e)
  }
  return false
}
