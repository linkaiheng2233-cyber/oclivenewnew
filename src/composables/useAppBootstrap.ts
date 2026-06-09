import { listen } from '@tauri-apps/api/event'
import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import {
  consumePendingProtocolInstalls,
  installPluginFromGit,
  loadRole,
  setErrorReporter,
} from '../api'
import { hostEventBus } from '../lib/hostEventBus'
import { useDebugStore } from '../stores/debugStore'
import { usePluginStore } from '../stores/pluginStore'
import { useRoleStore } from '../stores/roleStore'
import { resolveDefaultRoleId } from '../utils/presetRolePicker'
import { useNarrativeScene } from './useNarrativeScene'
import type { AppToastFn } from './useAppToast'

async function disposeTauriListener(
  handle: (() => void) | Promise<(() => void)> | undefined,
): Promise<void> {
  if (!handle)
    return
  if (typeof handle === 'function') {
    handle()
    return
  }
  const unlisten = await handle
  unlisten()
}

export function useAppBootstrap(options: {
  showToast: AppToastFn
  t: ComposerTranslation
  openPluginManagerPanel: () => void
  localePreference: Ref<unknown>
  syncBrowserChromeFromLocale: () => void
  scheduleRefreshSplitLayout: () => void
  refreshSplitLayout: () => void
  onPresetPickerRequired?: () => void
}) {
  const roleStore = useRoleStore()
  const pluginStore = usePluginStore()
  const debugStore = useDebugStore()
  const { applyResolvedNarrativeScene } = useNarrativeScene()

  let unlistenPluginFs: (() => void) | Promise<(() => void)> | undefined
  let unlistenProtocolInstall: (() => void) | Promise<(() => void)> | undefined

  async function completeRoleBootstrap(roleId?: string) {
    const rid = (roleId ?? roleStore.currentRoleId).trim()
    if (!rid) {
      options.showToast('error', options.t('app.toast.noRolesScanned'))
      return
    }
    await loadRole(rid)
    await pluginStore.refresh()
    await roleStore.refreshRoleInfo()
    hostEventBus.emitBuiltin('role:switched', { roleId: rid })
    applyResolvedNarrativeScene()
    await debugStore.loadDebugData()
  }

  async function initialize() {
    try {
      await roleStore.loadRoles()
      if (roleStore.roles.length === 0) {
        options.showToast('error', options.t('app.toast.noRolesScanned'))
        return
      }
      if (roleStore.needsPresetPicker()) {
        options.onPresetPickerRequired?.()
        return
      }
      if (!roleStore.currentRoleId.trim()) {
        roleStore.$patch({ currentRoleId: resolveDefaultRoleId(roleStore.roles) })
      }
      await completeRoleBootstrap()
    }
    catch (err) {
      options.showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function runPendingProtocolInstallsFromQueue() {
    try {
      const pending = await consumePendingProtocolInstalls()
      for (const p of pending) {
        const git = p.gitUrl?.trim()
        if (!git)
          continue
        try {
          const r = await installPluginFromGit(git)
          options.showToast('success', options.t('app.toast.pluginInstalledFromWeb', { id: r.installedPluginId }))
          await pluginStore.refresh()
          options.openPluginManagerPanel()
        }
        catch (e) {
          options.showToast('error', e instanceof Error ? e.message : String(e))
        }
      }
    }
    catch (e) {
      console.warn('consume_pending_protocol_installs', e)
    }
  }

  onMounted(() => {
    options.syncBrowserChromeFromLocale()
    setErrorReporter((err) => {
      options.showToast('error', err.message)
    })
    window.addEventListener('resize', options.scheduleRefreshSplitLayout)
    options.refreshSplitLayout()
    void initialize()
    void listen('plugin:changed', () => {
      void pluginStore.onPluginFilesChanged().then(() => {
        options.showToast('success', options.t('app.toast.pluginFilesChanged'))
      })
    }).then((u) => {
      unlistenPluginFs = u
    }).catch((e) => {
      console.warn('listen plugin:changed failed', e)
    })

    void listen('protocol:pending_install', () => {
      void runPendingProtocolInstallsFromQueue()
    }).then((u) => {
      unlistenProtocolInstall = u
    }).catch((e) => {
      console.warn('listen protocol:pending_install failed', e)
    })

    void runPendingProtocolInstallsFromQueue()
  })

  onBeforeUnmount(() => {
    setErrorReporter(null)
    window.removeEventListener('resize', options.scheduleRefreshSplitLayout)
    void disposeTauriListener(unlistenPluginFs)
    void disposeTauriListener(unlistenProtocolInstall)
  })

  return { initialize, completeRoleBootstrap }
}
