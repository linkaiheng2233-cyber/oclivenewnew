import type { LocalePreference } from '@oclive/shared/i18n'
import {
  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
} from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useInteractionModeSettings } from '@oclive/shared/composables/useInteractionModeSettings'
import { useRoleSnapshotPoll } from '@oclive/shared/composables/useKernelStatus'
import { resolveOcliveShell } from '@oclive/shared/composables/useOcliveShell'
import { usePluginEvents } from '@oclive/shared/composables/usePluginEvents'
import { useProgressiveDisclosure } from '@oclive/shared/composables/useProgressiveDisclosure'
import { useReturnFocusOnClose } from '@oclive/shared/composables/useReturnFocusOnClose'
import { useSceneDestination } from '@oclive/shared/composables/useSceneDestination'
import { useSceneTravelBars } from '@oclive/shared/composables/useSceneTravelBars'
import { usePackUiTheme } from '@oclive/shared/composables/useTheme'
import { useVoiceAutoTts } from '@oclive/shared/composables/useVoiceAutoTts'
import { getLocalePreference } from '@oclive/shared/i18n'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { usePluginMarketStore } from '@oclive/shared/stores/pluginMarketStore'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { markPresetPickerDone } from '@oclive/shared/utils/presetRolePicker'
import { buildRelationDropdownOptions } from '@oclive/shared/utils/relationOptions'
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppBootstrap } from './useAppBootstrap'
import { useMainShellChat } from './useMainShellChat'
import { useMainShellHotkeys } from './useMainShellHotkeys'
import { useMainShellWindows } from './useMainShellWindows'

export const DebugPanel = defineAsyncComponent(() => import('@oclive/shared/components/dev-tools/DebugPanel.vue'))
export const MarketView = defineAsyncComponent(() => import('../views/MarketView.vue'))
export const SettingsView = defineAsyncComponent(() => import('../views/SettingsView.vue'))
export const ModelManagerPanel = defineAsyncComponent(() => import('../views/ModelManagerPanel.vue'))
export const SimplePluginManagerPanel = defineAsyncComponent(() => import('../views/SimplePluginManagerPanel.vue'))
export const RoleDetailView = defineAsyncComponent(() => import('../views/RoleDetailView.vue'))
export const SceneTravelBars = defineAsyncComponent(() => import('@oclive/shared/components/SceneTravelBars.vue'))
export const RoleplayAsidePanel = defineAsyncComponent(() => import('@oclive/shared/components/RoleplayAsidePanel.vue'))
export const TopBarMorePanel = defineAsyncComponent(() => import('@oclive/shared/components/TopBarMorePanel.vue'))
export const AutonomousSceneNotice = defineAsyncComponent(() => import('@oclive/shared/components/AutonomousSceneNotice.vue'))

export function useMainShell() {
  const roleStore = useRoleStore()
  const adultStore = useAdultInteractionStore()
  usePackUiTheme()
  const chatStore = useChatStore()
  const debugStore = useDebugStore()
  const uiStore = useUiStore()
  const pluginStore = usePluginStore()
  const pluginMarketStore = usePluginMarketStore()
  const { t, locale } = useI18n()

  function syncBrowserChromeFromLocale(): void {
    document.title = t('app.documentTitle')
    document.documentElement.setAttribute('lang', locale.value === 'en-US' ? 'en' : 'zh-CN')
  }
  const localePreference = ref<LocalePreference>(getLocalePreference())

  const { toast, showToast } = useAppToast()
  const progressive = useProgressiveDisclosure()
  const { applyPureChatSceneIsolation, onInteractionModeSelect } = useInteractionModeSettings()
  useRoleSnapshotPoll()
  const {
    sceneTransition,
    applySceneDestination,
    sceneLabelForId,
  } = useSceneDestination(showToast)

  const leftPaneRef = ref<HTMLElement | null>(null)
  const roleSwitching = ref(false)
  const presetPickerOpen = ref(false)
  const presetPickerPicking = ref(false)

  const wideSplitLayout = ref(typeof window !== 'undefined' && window.innerWidth > 720)
  function refreshSplitLayout(): void {
    wideSplitLayout.value = typeof window !== 'undefined' && window.innerWidth > 720
  }

  let splitLayoutResizeRaf = 0
  function scheduleRefreshSplitLayout(): void {
    if (splitLayoutResizeRaf !== 0)
      return
    splitLayoutResizeRaf = requestAnimationFrame(() => {
      splitLayoutResizeRaf = 0
      refreshSplitLayout()
    })
  }

  const relationOptions = computed(() =>
    buildRelationDropdownOptions(
      roleStore.roleInfo.userRelations ?? [],
      roleStore.roleInfo.defaultRelation,
    ),
  )

  const connectivityPluginIndexDetail = computed(() => {
    const b = uiStore.connectivityBanner
    if (!b || b.kind !== 'plugin_index_offline' || !b.detail)
      return ''
    const d = b.detail
    return d.length > 200 ? `${d.slice(0, 200)}…` : d
  })

  const {
    topMoreOpen,
    settingsViewOpen,
    settingsFocusTab,
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
    modelManagerOpen,
    openModelManager,
    closeModelManager,
    closeAllSidePanels,
  } = useMainShellWindows({
    pluginMarketStore,
  })

  const {
    allSceneOptions,
    sceneDestinationOptions,
    postReplySceneBarVisible,
    postReplySceneSelectedId,
    togetherTravelBarVisible,
    togetherTravelSelectedId,
    topBarSceneDialogVisible,
    pendingTopBarSceneId,
    autonomousSceneNotice,
    resetPureChatSceneUi,
    dismissPostReplySceneBar,
    dismissTogetherTravelBar,
    confirmPostReplyScene,
    confirmTogetherTravel,
    onTopBarSceneChange,
    dismissTopBarSceneDialog,
    confirmTopBarScene,
    onPluginQuickActionTravel,
    onVirtualTimeJumpComplete,
    dismissAutonomousSceneNotice,
    offerSceneBarsAfterReply,
    clearSceneBarsBeforeSend,
  } = useSceneTravelBars({ applySceneDestination, sceneLabelForId })

  function closePluginSurfaces(): void {
    simplePluginManagerOpen.value = false
    pluginMarketStore.closeMarketPanel()
    pluginsPanelSubview.value = 'list'
    if (debugStore.visible)
      debugStore.toggle()
  }

  const {
    shortcutHelpOpen,
    openShortcutHelp,
    openSettingsView,
    openSettingsToGeneral,
    sidePanelOpen,
    sidePanelTab,
  } = useMainShellHotkeys({
    simplePluginManagerOpen,
    settingsViewOpen,
    topMoreOpen,
    marketPanelVisible: computed(() => pluginMarketStore.marketPanelVisible),
    modelManagerOpen,
    debugVisible: computed(() => debugStore.visible),
    pluginUiEnabled: computed(() => roleStore.interactionImmersive),
    debugUiEnabled: computed(() => roleStore.interactionImmersive),
    openPluginManagerPanel,
    openModelManager: () => openModelManager(),
    toggleDebug: () => debugStore.toggle(),
    closeMarketPanel: () => pluginMarketStore.closeMarketPanel(),
    closeModelManager,
    settingsFocusTab,
  })

  useReturnFocusOnClose(settingsViewOpen)
  useReturnFocusOnClose(simplePluginManagerOpen)
  useReturnFocusOnClose(modelManagerOpen)
  useReturnFocusOnClose(shortcutHelpOpen)

  function onHostOpenModelManager(): void {
    if (resolveOcliveShell() === 'theater') {
      closeModelManager()
      hostEventBus.emit('theater:settings', { action: 'model' })
      return
    }
    openModelManager(true)
  }

  function onHostOpenPluginManager(): void {
    openSimplePluginManager(true)
  }

  watch(
    () => roleStore.roleInfo.interactionMode,
    (mode) => {
      // Default state is pure_chat before refreshRoleInfo; cold start is handled by
      // completeRoleBootstrap.bootstrapChatForRole — do not race-load the wrong bucket.
      if (!roleStore.roleInfo.version)
        return
      if (mode === 'pure_chat') {
        applyPureChatSceneIsolation()
        resetPureChatSceneUi()
        closePluginSurfaces()
        if (settingsFocusTab.value === 'plugins')
          settingsFocusTab.value = 'general'
        const roleId = roleStore.currentRoleId
        if (roleId)
          void chatStore.enterPureChatScene(roleId)
      }
    },
  )

  const packLayoutResolved = computed(() => {
    const l = roleStore.roleInfo.packUiConfig?.layout ?? {
      sidebar: '',
      chatInput: '',
    }
    const sidebar = l.sidebar === 'right' ? 'right' : 'left'
    const chatInput = l.chatInput === 'top' ? 'top' : 'bottom'
    return { sidebar, chatInput }
  })
  const sidebarRight = computed(() => packLayoutResolved.value.sidebar === 'right')
  const chatInputTop = computed(() => packLayoutResolved.value.chatInput === 'top')
  const roleName = computed(() => roleStore.roleInfo.name || t('app.defaultRoleName'))
  const emotion = computed(() => roleStore.roleInfo.currentEmotion || 'neutral')
  const portraitAssetRelPath = computed(
    () => roleStore.roleInfo.portraitAssetPath ?? null,
  )

  const statusHeart = computed(() => {
    const f = roleStore.roleInfo.favorability
    if (f >= 60)
      return '💖'
    if (f >= 30)
      return '💕'
    return '🤍'
  })

  const favorClosenessLabel = computed(() => {
    const f = roleStore.roleInfo.favorability
    if (f >= 60)
      return t('onboarding.favorLabel.close')
    if (f >= 30)
      return t('onboarding.favorLabel.warm')
    return t('onboarding.favorLabel.distant')
  })

  async function onInteractionModeChange(ev: Event) {
    // Primary in-shell handler for InteractionModeBar (Settings → General is the other user entry).
    const v = (ev.target as HTMLSelectElement).value as 'immersive' | 'pure_chat'
    await onInteractionModeSelect(ev)
    if (v === 'pure_chat')
      resetPureChatSceneUi()
  }

  useAppBootstrap({
    showToast,
    t,
    openPluginManagerPanel,
    localePreference,
    syncBrowserChromeFromLocale,
    scheduleRefreshSplitLayout,
    refreshSplitLayout,
    onPresetPickerRequired: () => {
      presetPickerOpen.value = true
    },
  })

  async function onPresetRolePick(roleId: string) {
    if (presetPickerPicking.value)
      return
    presetPickerPicking.value = true
    try {
      markPresetPickerDone()
      presetPickerOpen.value = false
      await onSwitchRole(roleId)
    }
    catch (err) {
      presetPickerOpen.value = true
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      presetPickerPicking.value = false
    }
  }

  const {
    chatListRef,
    chatInputRef,
    messages,
    chatListLoading,
    latestRoleplayAside,
    sceneHistorySplitIndex,
    onSend,
    onAdultAction,
  } = useMainShellChat({
    roleStore,
    uiStore,
    showToast,
    t,
    clearSceneBarsBeforeSend,
    offerSceneBarsAfterReply,
    onTurnRecorded: () => progressive.recordTurn(),
  })

  usePluginEvents({
    showToast,
    onQuickActionTravel: onPluginQuickActionTravel,
    onPureChatMode: resetPureChatSceneUi,
    onVoiceAsrSubmit: ({ text, mode }) => {
      if (!text?.trim())
        return
      if (mode === 'fill') {
        hostEventBus.emit('chat:set_input_draft', { text: text.trim() })
        return
      }
      if (chatListLoading.value)
        return
      void onSend({ content: text.trim() })
    },
  })

  useVoiceAutoTts({ showToast })

  async function onSwitchRole(nextRoleId: string) {
    if (roleSwitching.value || !nextRoleId.trim() || nextRoleId === roleStore.currentRoleId)
      return
    const savedLeftScroll = leftPaneRef.value?.scrollTop ?? 0
    try {
      roleSwitching.value = true
      chatStore.cancelPendingSend()
      await roleStore.switchRole(nextRoleId)
      await chatStore.bootstrapChatForRole(nextRoleId)
      await pluginStore.syncDirectoryPluginBootstrap()
      hostEventBus.emitBuiltin('role:switched', { roleId: nextRoleId })
      await debugStore.loadDebugData()
      showToast('success', t('app.toast.roleSwitched', { id: nextRoleId }))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      window.setTimeout(() => {
        roleSwitching.value = false
        void nextTick(() => {
          const pane = leftPaneRef.value
          if (pane)
            pane.scrollTop = savedLeftScroll
        })
      }, 220)
    }
  }

  async function onChangeRelation(nextRelation: string) {
    const roleId = roleStore.currentRoleId
    try {
      const relationName
        = relationOptions.value.find(r => r.id === nextRelation)?.name ?? nextRelation
      const sceneId = uiStore.sceneId || 'default'
      const endedAdultInteraction
        = adultStore.sessionFor(roleId, sceneId).active
      const perScene = roleStore.roleInfo.identityBinding === 'per_scene'
      if (nextRelation === OCLIVE_DEFAULT_RELATION_SENTINEL) {
        if (perScene)
          await roleStore.setManifestDefaultRelation(uiStore.sceneId)
        else
          await roleStore.setManifestDefaultRelation()
      }
      else if (perScene) {
        await roleStore.setSceneUserRelation(uiStore.sceneId, nextRelation)
      }
      else {
        await roleStore.setGlobalUserRelation(nextRelation, sceneId)
      }
      if (roleStore.currentRoleId !== roleId)
        return
      if (endedAdultInteraction) {
        await chatStore.sendAdultAction(
          'exit',
          sceneId,
          `用户身份已经切换为“${relationName}”。原身份下的互动已经结束；请从普通聊天状态开始，按照角色人设自然回应这次身份变化。`,
        )
      }
      const scopeKey = perScene ? 'app.toast.relationSetPerScene' : 'app.toast.relationSetGlobal'
      showToast('success', t(scopeKey, { name: relationName }))
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function onPackImported(roleId: string) {
    try {
      roleStore.currentRoleId = roleId
      await loadRole(roleId)
      await pluginStore.refresh()
      await roleStore.refreshRoleInfo()
      await roleStore.loadRoles()
      await chatStore.bootstrapChatForRole(roleId)
      await debugStore.loadDebugData()
      if (roleStore.roleInfo.adultExtensionAvailable) {
        const accepted = window.confirm(
          adultStore.confirmedAdult
            ? String(t('settings.adult.importPrompt'))
            : `${String(t('settings.adult.legalTitle'))}\n\n${String(t('settings.adult.legalBody'))}\n\n${String(t('settings.adult.importPrompt'))}`,
        )
        if (accepted) {
          adultStore.confirmAndEnableGlobal()
          adultStore.setRoleEnabled(roleId, true)
        }
      }
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function onReloadPolicy() {
    try {
      const msg = await debugStore.reloadPolicy()
      showToast('success', msg)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function onDebugRefresh() {
    try {
      await debugStore.loadDebugData()
    }
    catch {
      // toastAsyncError already surfaced in debugStore.loadDebugData
    }
  }

  watch(
    () => messages.value.length,
    async (len, prev) => {
      if (prev !== undefined && len <= prev)
        return
      await nextTick()
      chatListRef.value?.scrollToBottom?.(false)
    },
    { flush: 'post' },
  )

  watch(
    () => debugStore.visible,
    (v) => {
      if (v)
        void onDebugRefresh()
    },
  )

  watch(locale, () => {
    syncBrowserChromeFromLocale()
  })

  onMounted(() => {
    hostEventBus.on('ui:open_model_manager', onHostOpenModelManager)
    hostEventBus.on('ui:open_plugin_manager', onHostOpenPluginManager)
    localePreference.value = getLocalePreference()
  })

  onBeforeUnmount(() => {
    hostEventBus.off('ui:open_model_manager', onHostOpenModelManager)
    hostEventBus.off('ui:open_plugin_manager', onHostOpenPluginManager)
    if (splitLayoutResizeRaf !== 0) {
      cancelAnimationFrame(splitLayoutResizeRaf)
      splitLayoutResizeRaf = 0
    }
  })

  function openSidePanelTab(tab: 'settings' | 'plugins' | 'models'): void {
    if (tab === 'settings') {
      openSettingsView()
      return
    }
    if (tab === 'plugins') {
      if (!roleStore.interactionImmersive) {
        showToast('info', t('app.toast.pluginsStoryModeOnly'))
        return
      }
      openSimplePluginManager(true)
      return
    }
    if (resolveOcliveShell() === 'theater') {
      closeModelManager()
      hostEventBus.emit('theater:settings', { action: 'model' })
      return
    }
    openModelManager(true)
  }

  function onSidePanelTabChange(tab: 'settings' | 'plugins' | 'models'): void {
    openSidePanelTab(tab)
  }

  return {
    t,
    localePreference,
    toast,
    showToast,
    roleStore,
    chatStore,
    debugStore,
    uiStore,
    pluginStore,
    pluginMarketStore,
    chatListRef,
    chatInputRef,
    leftPaneRef,
    roleSwitching,
    wideSplitLayout,
    relationOptions,
    connectivityPluginIndexDetail,
    messages,
    chatListLoading,
    latestRoleplayAside,
    topMoreOpen,
    settingsViewOpen,
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
    modelManagerOpen,
    openModelManager,
    closeModelManager,
    sidePanelOpen,
    sidePanelTab,
    openSidePanelTab,
    closeAllSidePanels,
    onSidePanelTabChange,
    allSceneOptions,
    sceneDestinationOptions,
    postReplySceneBarVisible,
    postReplySceneSelectedId,
    togetherTravelBarVisible,
    togetherTravelSelectedId,
    topBarSceneDialogVisible,
    pendingTopBarSceneId,
    autonomousSceneNotice,
    dismissPostReplySceneBar,
    dismissTogetherTravelBar,
    confirmPostReplyScene,
    confirmTogetherTravel,
    onTopBarSceneChange,
    dismissTopBarSceneDialog,
    confirmTopBarScene,
    onPluginQuickActionTravel,
    onVirtualTimeJumpComplete,
    dismissAutonomousSceneNotice,
    shortcutHelpOpen,
    openShortcutHelp,
    openSettingsView,
    openSettingsToGeneral,
    settingsFocusTab,
    sceneTransition,
    sceneLabelForId,
    sceneHistorySplitIndex,
    packLayoutResolved,
    sidebarRight,
    chatInputTop,
    roleName,
    emotion,
    portraitAssetRelPath,
    statusHeart,
    favorClosenessLabel,
    progressive,
    onInteractionModeChange,
    onSend,
    onAdultAction,
    onSwitchRole,
    onChangeRelation,
    onPackImported,
    onReloadPolicy,
    onDebugRefresh,
    presetPickerOpen,
    presetPickerPicking,
    onPresetRolePick,
  }
}
