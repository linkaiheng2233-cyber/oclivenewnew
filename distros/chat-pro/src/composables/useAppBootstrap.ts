import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import {
  consumePendingProtocolInstalls,
  installPluginFromGit,
  loadRole,
  setErrorReporter,
} from '@oclive/shared/api'
import { ApiInvokeError, toErrorPresentation } from '@oclive/shared/api/helpers'
import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'
import { showPluginInstallReviewHint } from '@oclive/shared/composables/usePluginInstallReviewHint'
import { startVoiceExpansionWarmOnStartup } from '@oclive/shared/composables/useVoiceExpansionWarm'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { bindAffectMetricsListener, useRoleStore } from '@oclive/shared/stores/roleStore'
import { markPresetPickerDone, resolveDefaultRoleId } from '@oclive/shared/utils/presetRolePicker'
import { getTheaterCastConfig } from '@oclive/theater/composables/theater/theaterCastConfig'
import { listen } from '@tauri-apps/api/event'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

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

export function createProtocolInstallQueue() {
  return {
    pending: [] as Awaited<ReturnType<typeof consumePendingProtocolInstalls>>,
    running: null as Promise<void> | null,
  }
}

export async function installPendingProtocolPlugins(options: {
  queue?: ReturnType<typeof createProtocolInstallQueue>
  advancedFeaturesEnabled?: () => boolean
  showToast: AppToastFn
  t: ComposerTranslation
  refreshPlugins: () => Promise<void>
  openPluginManagerPanel: () => void
  confirmInstall?: (gitUrl: string) => boolean | Promise<boolean>
}): Promise<void> {
  if (options.advancedFeaturesEnabled?.() === false)
    return
  const queue = options.queue ?? createProtocolInstallQueue()
  if (queue.running) {
    await queue.running
    return installPendingProtocolPlugins({ ...options, queue })
  }
  const run = (async () => {
    try {
      queue.pending.push(...await consumePendingProtocolInstalls())
      while (queue.pending.length > 0) {
        if (options.advancedFeaturesEnabled?.() === false)
          return
        const item = queue.pending.shift()!
        const git = item.gitUrl?.trim()
        if (!git)
          continue
        try {
          const source = new URL(git)
          if (git.length > 4096 || source.protocol !== 'https:' || !source.hostname
            || source.username || source.password || source.search || source.hash) {
            options.showToast('error', options.t('app.toast.pluginLinkInvalid'))
            continue
          }
          // A protocol event supplies a request, never consent to install its code.
          const confirmed = await (options.confirmInstall
            ? options.confirmInstall(git)
            : window.confirm(options.t('app.toast.pluginProtocolConfirm', { source: git })))
          if (!confirmed)
            continue
          if (options.advancedFeaturesEnabled?.() === false) {
            queue.pending.unshift(item)
            return
          }
          const result = await installPluginFromGit(git)
          // Already-issued installs may finish; hidden tools must not reopen their UI.
          await options.refreshPlugins()
          if (options.advancedFeaturesEnabled?.() === false)
            continue
          options.showToast(
            'success',
            options.t('app.toast.pluginInstalledFromWeb', {
              id: result.installedPluginId,
            }),
          )
          showPluginInstallReviewHint(options.showToast, result)
          options.openPluginManagerPanel()
        }
        catch (error) {
          if (options.advancedFeaturesEnabled?.() === false)
            continue
          options.showToast(
            'error',
            error instanceof Error ? error.message : String(error),
          )
        }
      }
    }
    catch (error) {
      console.warn('consume_pending_protocol_installs', error)
    }
  })()
  queue.running = run
  try {
    await run
  }
  finally {
    if (queue.running === run)
      queue.running = null
  }
}

export function useAppBootstrap(options: {
  deferRoleActivation?: boolean
  advancedFeaturesEnabled?: () => boolean
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
  const kernelConnection = useKernelConnectionStore()
  const protocolInstallQueue = createProtocolInstallQueue()
  const startupState = ref<'loading' | 'ready' | 'error'>('loading')
  const startupError = ref('')
  const startupErrorDetails = ref('')
  let disposed = false

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
    if (options.advancedFeaturesEnabled?.() !== false)
      startVoiceExpansionWarmOnStartup(id => pluginStore.isPluginDisabled(id))
    await roleStore.refreshRoleInfo()
    hostEventBus.emitBuiltin('role:switched', { roleId: rid })
    const chatStore = useChatStore()
    await chatStore.bootstrapChatForRole(rid)
    if (options.advancedFeaturesEnabled?.() !== false)
      await debugStore.loadDebugData()
  }

  async function initialize() {
    startupState.value = 'loading'
    startupError.value = ''
    startupErrorDetails.value = ''
    try {
      if (options.deferRoleActivation)
        roleStore.currentRoleId = ''
      const chatStore = useChatStore()
      await chatStore.hydrateFromStorage()
      chatStore.migrateAllLegacyMessageBuckets()
      await roleStore.loadRoles(!options.deferRoleActivation)
      if (options.deferRoleActivation) {
        startupState.value = 'ready'
        return
      }
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
      const failure = toErrorPresentation(err)
      startupError.value = failure.summary
      startupErrorDetails.value = failure.details
      startupState.value = 'error'
      options.showToast('error', failure.summary, failure.details)
    }
    finally {
      if (startupState.value === 'loading')
        startupState.value = 'ready'
    }
  }

  async function runPendingProtocolInstallsFromQueue() {
    await installPendingProtocolPlugins({
      queue: protocolInstallQueue,
      advancedFeaturesEnabled: () => !disposed && options.advancedFeaturesEnabled?.() !== false,
      showToast: options.showToast,
      t: options.t,
      refreshPlugins: () => pluginStore.refresh(),
      openPluginManagerPanel: options.openPluginManagerPanel,
    })
  }

  watch(() => options.advancedFeaturesEnabled?.() ?? true, (enabled) => {
    if (enabled)
      void runPendingProtocolInstallsFromQueue()
  })

  onMounted(() => {
    void bindAffectMetricsListener()
    options.syncBrowserChromeFromLocale()
    setErrorReporter((err) => {
      // Cold-start probes can fail before the kernel is ready. The connection indicator
      // owns that transient state; action-specific catches still report actual failures.
      if (err.code === 'KERNEL_OFFLINE'
        && (startupState.value === 'loading'
          || kernelConnection.phase === 'checking'
          || kernelConnection.status?.mode === 'reconnecting')) {
        return
      }
      const failure = toErrorPresentation(new ApiInvokeError(err))
      options.showToast('error', failure.summary, failure.details)
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
    disposed = true
    setErrorReporter(null)
    window.removeEventListener('resize', options.scheduleRefreshSplitLayout)
    void disposeTauriListener(unlistenPluginFs)
    void disposeTauriListener(unlistenProtocolInstall)
  })

  return { initialize, completeRoleBootstrap, startupState, startupError, startupErrorDetails }
}
