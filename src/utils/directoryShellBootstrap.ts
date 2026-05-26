import type { DirectoryPluginBootstrap } from '../api'
import { invoke } from '@tauri-apps/api/tauri'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import DirectoryShellApp from '../DirectoryShellApp.vue'
import { i18n } from '../i18n/index'
import { readPluginAssetText } from '../api'

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined'
    && Object.hasOwn(window, '__TAURI_INTERNALS__')
  )
}

/** 开发/排障：跳过整壳目录插件，强制挂载主界面（`.env` 设 `VITE_OCLIVE_DISABLE_DIRECTORY_SHELL=1`）。 */
export function isDirectoryShellDisabled(): boolean {
  return import.meta.env.VITE_OCLIVE_DISABLE_DIRECTORY_SHELL === '1'
}

/**
 * 若配置了整壳目录插件：优先在 **`shell.vueEntry` + 非强制 iframe** 时用宿主 Vue 挂载整壳；
 * 否则在 **`shellUrl`** 与当前页不同时执行 `location.replace(shellUrl)`（HTML 整壳）。
 *
 * @returns 若已处理整壳（Vue 已挂载或已发起 HTML 跳转）则为 true，调用方不应再挂载应用根组件。
 */
export async function tryReplaceWithDirectoryShell(): Promise<boolean> {
  if (!isTauriRuntime() || isDirectoryShellDisabled())
    return false
  try {
    const boot = await invoke<DirectoryPluginBootstrap>('get_directory_plugin_bootstrap', {
      role_id: null,
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

    const forceIframe = boot.forceIframeMode === true
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
