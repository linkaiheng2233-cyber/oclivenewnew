import type ChatMessageList from '../components/chat/ChatMessageList.vue'
import type { LocalePreference } from '../i18n'
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setRoleInteractionMode,
  setUserRelation,
} from '../api'
import { getLocalePreference } from '../i18n'
import { hostEventBus } from '../lib/hostEventBus'
import { useChatStore } from '../stores/chatStore'
import { useDebugStore } from '../stores/debugStore'
import { usePluginMarketStore } from '../stores/pluginMarketStore'
import { usePluginStore } from '../stores/pluginStore'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'
import { buildRelationDropdownOptions } from '../utils/relationOptions'
import { useAppBootstrap } from './useAppBootstrap'
import { useAppToast } from './useAppToast'
import { useChatSend } from './useChatSend'
import { useGlobalHotkeys } from './useGlobalHotkeys'
import { useRoleSnapshotPoll } from './useKernelStatus'
import { useModelManagerWindow } from './useModelManagerWindow'
import { useNarrativeScene } from './useNarrativeScene'
import { usePluginEvents } from './usePluginEvents'
import { usePluginManagerWindow } from './usePluginManagerWindow'
import { useReturnFocusOnClose } from './useReturnFocusOnClose'
import { useSceneDestination } from './useSceneDestination'
import { useSceneTravelBars } from './useSceneTravelBars'
import { usePackUiTheme } from './useTheme'
import { useProgressiveDisclosure } from './useProgressiveDisclosure'
import { markPresetPickerDone } from '../utils/presetRolePicker'

export const DebugPanel = defineAsyncComponent(() => import('../components/dev-tools/DebugPanel.vue'))
export const MarketView = defineAsyncComponent(() => import('../views/MarketView.vue'))
export const SettingsView = defineAsyncComponent(() => import('../views/SettingsView.vue'))
export const ModelManagerPanel = defineAsyncComponent(() => import('../views/ModelManagerPanel.vue'))
export const SimplePluginManagerPanel = defineAsyncComponent(() => import('../views/SimplePluginManagerPanel.vue'))
export const RoleDetailView = defineAsyncComponent(() => import('../views/RoleDetailView.vue'))
export const SceneTravelBars = defineAsyncComponent(() => import('../components/SceneTravelBars.vue'))
export const RoleplayAsidePanel = defineAsyncComponent(() => import('../components/RoleplayAsidePanel.vue'))
export const TopBarMorePanel = defineAsyncComponent(() => import('../components/TopBarMorePanel.vue'))
export const AutonomousSceneNotice = defineAsyncComponent(() => import('../components/AutonomousSceneNotice.vue'))

export function useMainShell() {
  const roleStore = useRoleStore()
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
  useRoleSnapshotPoll()
  const { applyResolvedNarrativeScene } = useNarrativeScene()
  const {
    sceneTransition,
    applySceneDestination,
    sceneLabelForId,
  } = useSceneDestination(showToast)

  const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null)
  const chatInputRef = ref<{ focusInput?: () => void } | null>(null)
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

  const messages = computed(() =>
    chatStore.messagesForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
  )

  const chatListLoading = computed(() =>
    chatStore.isLoading
    || chatStore.isMessagesLoadingFor(roleStore.currentRoleId, uiStore.sceneId),
  )

  const latestRoleplayAside = computed(() => {
    const roleId = roleStore.currentRoleId
    const sceneId = uiStore.sceneId || 'default'
    return chatStore.lastAssistantAsideFor(roleId, sceneId)
  })

  const topMoreOpen = ref(false)
  const settingsViewOpen = ref(false)
  const settingsFocusTab = ref<'general' | 'plugins' | 'storage' | null>(null)

  const {
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
  } = usePluginManagerWindow({
    closeMoreMenu: () => {
      topMoreOpen.value = false
    },
  })

  const {
    modelManagerOpen,
    openModelManager,
    closeModelManager,
  } = useModelManagerWindow({
    closeMoreMenu: () => {
      topMoreOpen.value = false
    },
  })

  watch(simplePluginManagerOpen, (open) => {
    if (open) {
      modelManagerOpen.value = false
      settingsViewOpen.value = false
    }
  })
  watch(modelManagerOpen, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      settingsViewOpen.value = false
      pluginMarketStore.closeMarketPanel()
    }
  })
  watch(settingsViewOpen, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      modelManagerOpen.value = false
      pluginMarketStore.closeMarketPanel()
    }
  })
  watch(() => pluginMarketStore.marketPanelVisible, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      modelManagerOpen.value = false
    }
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

  const { shortcutHelpOpen, openShortcutHelp, openSettingsView } = useGlobalHotkeys({
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
  })

  function openSettingsToGeneral(): void {
    settingsFocusTab.value = 'general'
    openSettingsView()
  }

  usePluginEvents({
    showToast,
    onQuickActionTravel: onPluginQuickActionTravel,
    onPureChatMode: resetPureChatSceneUi,
  })

  useReturnFocusOnClose(settingsViewOpen)
  useReturnFocusOnClose(simplePluginManagerOpen)
  useReturnFocusOnClose(modelManagerOpen)
  useReturnFocusOnClose(shortcutHelpOpen)

  function onHostOpenModelManager(): void {
    openModelManager(true)
  }

  function onHostOpenPluginManager(): void {
    openSimplePluginManager(true)
  }

  const sceneHistorySplitIndex = computed(() => {
    if (!roleStore.interactionImmersive)
      return 0
    return chatStore.sceneHistorySplitForRoleScene(roleStore.currentRoleId, uiStore.sceneId)
  })

  watch(
    () => roleStore.roleInfo.interactionMode,
    (mode) => {
      if (mode === 'pure_chat') {
        resetPureChatSceneUi()
        closePluginSurfaces()
        if (settingsFocusTab.value === 'plugins')
          settingsFocusTab.value = 'general'
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
    const v = (ev.target as HTMLSelectElement).value as 'immersive' | 'pure_chat'
    try {
      const info = await setRoleInteractionMode(roleStore.currentRoleId, v)
      roleStore.applyRoleInfo(info)
      if (v === 'pure_chat')
        resetPureChatSceneUi()
      showToast(
        'info',
        v === 'pure_chat'
          ? t('app.toast.interactionPureChat')
          : t('app.toast.interactionImmersive'),
      )
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
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

  const { onSend } = useChatSend({
    showToast,
    t,
    chatInputRef,
    clearSceneBarsBeforeSend,
    offerSceneBarsAfterReply,
    onTurnRecorded: (msg) => progressive.recordTurn(msg),
  })

  async function onSwitchRole(nextRoleId: string) {
    const savedLeftScroll = leftPaneRef.value?.scrollTop ?? 0
    try {
      roleSwitching.value = true
      await roleStore.switchRole(nextRoleId)
      await chatStore.loadMessagesForRoleScene(nextRoleId, uiStore.sceneId || 'default')
      await pluginStore.syncDirectoryPluginBootstrap()
      hostEventBus.emitBuiltin('role:switched', { roleId: nextRoleId })
      applyResolvedNarrativeScene()
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
    try {
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
        const info = await setUserRelation(roleStore.currentRoleId, nextRelation)
        roleStore.applyRoleInfo(info)
      }
      const relationName
        = relationOptions.value.find(r => r.id === nextRelation)?.name ?? nextRelation
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
      applyResolvedNarrativeScene()
      await debugStore.loadDebugData()
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
    openModelManager(true)
  }

  function closeAllSidePanels(): void {
    settingsViewOpen.value = false
    simplePluginManagerOpen.value = false
    closeModelManager()
  }

  const sidePanelOpen = computed(
    () => settingsViewOpen.value || simplePluginManagerOpen.value || modelManagerOpen.value,
  )

  const sidePanelTab = computed<'settings' | 'plugins' | 'models'>(() => {
    if (settingsViewOpen.value)
      return 'settings'
    if (simplePluginManagerOpen.value)
      return 'plugins'
    return 'models'
  })

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
