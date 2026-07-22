import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import {
  consumePendingProtocolInstalls,
  installPluginFromGit,
  loadRole,
  setErrorReporter,
} from '@oclive/shared/api'
import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'
import { startVoiceExpansionWarmOnStartup } from '@oclive/shared/composables/useVoiceExpansionWarm'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { bindAffectMetricsListener, useRoleStore } from '@oclive/shared/stores/roleStore'
import { markPresetPickerDone, resolveDefaultRoleId } from '@oclive/shared/utils/presetRolePicker'
import { getTheaterCastConfig } from '@oclive/theater/composables/theater/theaterCastConfig'
import { listen } from '@tauri-apps/api/event'
import { onBeforeUnmount, onMounted } from 'vue'

async function disposeTauriListener(
  handle: (() => void) | Promise<(() => void)> | undefined,
): Promise<void> {
  if (!handle)
    return
  if (typeof handle === 'function') {
    handle()
    return
  }
  try {
    const unlisten = await handle
    unlisten()
  }
  catch {
    // Registration failed before an unlisten handle existed.
  }
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
    startVoiceExpansionWarmOnStartup(id => pluginStore.isPluginDisabled(id))
    await roleStore.refreshRoleInfo()
    hostEventBus.emitBuiltin('role:switched', { roleId: rid })
    const chatStore = useChatStore()
    await chatStore.bootstrapChatForRole(rid)
    await debugStore.loadDebugData()
  }

  async function initialize() {
    try {
      await roleStore.loadRoles()
      if (roleStore.roles.length === 0) {
        options.showToast('error', options.t('app.toast.noRolesScanned'))
        return
      }

      // Theater: 0-config — skip first-run preset gallery; auto-wire saved cast A (fallback mumu).
      if (resolveOcliveShell() === 'theater') {
        markPresetPickerDone()
        const savedCastA = getTheaterCastConfig().castA.roleId
        const rid = roleStore.roles.find(r => r.id === savedCastA)?.id
          ?? roleStore.roles.find(r => r.id === 'mumu')?.id
          ?? roleStore.roles.find(r => r.id === '枫侵月')?.id
          ?? resolveDefaultRoleId(roleStore.roles)
        roleStore.$patch({ currentRoleId: rid })
        await completeRoleBootstrap(rid)
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
          showPluginInstallReviewHint(options.showToast, r)
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
    void bindAffectMetricsListener()
    options.syncBrowserChromeFromLocale()
    setErrorReporter((err) => {
      options.showToast('error', err.message)
    })
    window.addEventListener('resize', options.scheduleRefreshSplitLayout)
    options.refreshSplitLayout()
    void initialize()
    unlistenPluginFs = listen('plugin:changed', () => {
      void pluginStore.onPluginFilesChanged().then(() => {
        options.showToast('success', options.t('app.toast.pluginFilesChanged'))
      })
    })
    void Promise.resolve(unlistenPluginFs).catch((e) => {
      console.warn('listen plugin:changed failed', e)
    })

    unlistenProtocolInstall = listen('protocol:pending_install', () => {
      void runPendingProtocolInstallsFromQueue()
    })
    void Promise.resolve(unlistenProtocolInstall).catch((e) => {
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
