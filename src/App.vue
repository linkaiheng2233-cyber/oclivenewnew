<script setup lang="ts">
import type { LocalePreference } from './i18n'
import type { JumpTimeResponse } from './api'
import { listen } from '@tauri-apps/api/event'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AutonomousSceneNotice from './components/AutonomousSceneNotice.vue'
import ChatInput from './components/ChatInput.vue'
import ChatMessageList from './components/ChatMessageList.vue'
import ChatPluginToolbarSlots from './components/ChatPluginToolbarSlots.vue'
import DebugPanel from './components/DebugPanel.vue'
import HotkeyHost from './components/HotkeyHost.vue'
import PluginChatHeaderSlots from './components/PluginChatHeaderSlots.vue'
import PluginSidebarSlots from './components/PluginSidebarSlots.vue'
import PluginSlotEmbed from './components/PluginSlotEmbed.vue'
import RoleplayAsidePanel from './components/RoleplayAsidePanel.vue'
import RoleSelector from './components/RoleSelector.vue'
import SceneTravelBars from './components/SceneTravelBars.vue'
import ShortcutHelp from './components/ShortcutHelp.vue'
import TopBarMorePanel from './components/TopBarMorePanel.vue'
import Toast from './components/Toast.vue'
import TopBarSceneModeDialog from './components/TopBarSceneModeDialog.vue'
import { useAppToast } from './composables/useAppToast'
import { useNarrativeScene } from './composables/useNarrativeScene'
import { useOcliveAppearance } from './composables/useOcliveAppearance'
import { usePluginManagerWindow } from './composables/usePluginManagerWindow'
import { useReturnFocusOnClose } from './composables/useReturnFocusOnClose'
import { useSceneDestination } from './composables/useSceneDestination'
import { usePackUiTheme } from './composables/useTheme'
import {
  getLocalePreference,
} from './i18n'
import { hostEventBus } from './lib/hostEventBus'
import { useChatStore } from './stores/chatStore'
import { useDebugStore } from './stores/debugStore'
import { usePluginStore } from './stores/pluginStore'
import { useRoleStore } from './stores/roleStore'
import { useUiStore } from './stores/uiStore'
import { buildRelationDropdownOptions } from './utils/relationOptions'
import {
  consumePendingProtocolInstalls,
  installPluginFromGit,

  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setErrorReporter,
  setRemoteLifeEnabled,
  setRoleInteractionMode,
  setUserRelation,
} from './api'
import MarketView from './views/MarketView.vue'
import RoleDetailView from './views/RoleDetailView.vue'
import SettingsView from './views/SettingsView.vue'
import SimplePluginManagerPanel from './views/SimplePluginManagerPanel.vue'

const roleStore = useRoleStore()
usePackUiTheme()
const chatStore = useChatStore()
const debugStore = useDebugStore()
const uiStore = useUiStore()
const pluginStore = usePluginStore()
const { t, locale } = useI18n()

function syncBrowserChromeFromLocale(): void {
  document.title = t('app.documentTitle')
  document.documentElement.setAttribute('lang', locale.value === 'en-US' ? 'en' : 'zh-CN')
}
const localePreference = ref<LocalePreference>(getLocalePreference())

const { toast, showToast } = useAppToast()
const { cycleTheme } = useOcliveAppearance()
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

/** 角色回复结束后，若本句含位移意图且有多场景，显示目的地条 */
const postReplySceneBarVisible = ref(false)
const postReplySceneSelectedId = ref('')
/** 邀请同行语义：选目的地后同行或仅叙事 */
const togetherTravelBarVisible = ref(false)
const togetherTravelSelectedId = ref('')
/** 顶栏改场景：叙事独行 / 同行 */
const topBarSceneDialogVisible = ref(false)
const pendingTopBarSceneId = ref('')
/** 顶栏场景确认弹关闭后恢复焦点到场景下拉 */
const topBarSceneOpenerFocus = ref<HTMLElement | null>(null)
const quickActionTravelEvent = 'com.oclive.mumu.quick-actions:travel'
const settingsSetRemoteLifeEvent = 'com.oclive.mumu.settings-panel:set_remote_life'
const settingsSetInteractionModeEvent
  = 'com.oclive.mumu.settings-panel:set_interaction_mode'
const settingsCycleThemeEvent = 'com.oclive.mumu.settings-panel:cycle_theme'
const settingsResetLayoutEvent = 'com.oclive.mumu.settings-panel:request_reset_layout'
const settingsResetLayoutResultEvent = 'com.oclive.mumu.settings-panel:reset_layout_result'
/** 虚拟时间跳转触发 autonomous_scene 规则时，左下角系统提示 */
const autonomousSceneNotice = ref<{
  visible: boolean
  fromLabel: string
  toLabel: string
}>({ visible: false, fromLabel: '', toLabel: '' })

const shortcutHelpOpen = ref(false)
let ctrlLongPressTimer: ReturnType<typeof setTimeout> | null = null

function clearCtrlLongPressTimer(): void {
  if (ctrlLongPressTimer != null) {
    window.clearTimeout(ctrlLongPressTimer)
    ctrlLongPressTimer = null
  }
}

function onCtrlHoldHintKeydown(e: KeyboardEvent): void {
  if (e.key !== 'Control' || e.repeat) {
    return
  }
  clearCtrlLongPressTimer()
  ctrlLongPressTimer = window.setTimeout(() => {
    ctrlLongPressTimer = null
    shortcutHelpOpen.value = true
  }, 1000)
}

function onCtrlHoldHintKeyup(e: KeyboardEvent): void {
  if (e.key === 'Control') {
    clearCtrlLongPressTimer()
  }
}

/** 宽屏左右分栏；窄屏改为上下堆叠，立绘用 stack 布局更易读 */
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

/** 顶栏：全部场景选项（展示名） */
const allSceneOptions = computed(() => {
  const labels = roleStore.roleInfo.sceneLabels ?? []
  const scenes = roleStore.roleInfo.scenes ?? []
  if (labels.length > 0) {
    return labels.map(s => ({ id: s.id, label: s.label }))
  }
  return scenes.map(id => ({ id, label: id }))
})

/** 除当前叙事场景外可切换的目的地（位移条） */
const sceneDestinationOptions = computed(() => {
  const cur = uiStore.sceneId
  return allSceneOptions.value.filter(s => s.id !== cur)
})

const messages = computed(() =>
  chatStore.messagesForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
)

/** 本场景最近一条助手消息拆出的旁白/内心（供左侧叙事区，与主气泡对白分离） */
const latestRoleplayAside = computed(() => {
  const list = messages.value
  for (let i = list.length - 1; i >= 0; i--) {
    const m = list[i]
    if (m.role === 'assistant') {
      const a = m.aside?.trim()
      if (a)
        return a
    }
  }
  return ''
})

const topMoreOpen = ref(false)
const settingsViewOpen = ref(false)

const {
  simplePluginManagerOpen,
  openPluginManagerPanel,
  openPluginMarket,
  pluginManagerMoreBtnLabel,
  settingsEntryMoreHelp,
} = usePluginManagerWindow({
  closeMoreMenu: () => {
    topMoreOpen.value = false
  },
})

useReturnFocusOnClose(settingsViewOpen)
useReturnFocusOnClose(simplePluginManagerOpen)
useReturnFocusOnClose(shortcutHelpOpen)

function openShortcutHelp(): void {
  shortcutHelpOpen.value = true
  topMoreOpen.value = false
}

function openSettingsView(): void {
  settingsViewOpen.value = true
  topMoreOpen.value = false
}

const sceneHistorySplitIndex = computed(() =>
  chatStore.sceneHistorySplitForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
)

/** 角色包 `ui.json` → layout；空字段视为 left / bottom */
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

/** 对齐 oclive-new 底部状态栏心形 */
const statusHeart = computed(() => {
  const f = roleStore.roleInfo.favorability
  if (f >= 60)
    return '💖'
  if (f >= 30)
    return '💕'
  return '🤍'
})

async function onInteractionModeChange(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value as 'immersive' | 'pure_chat'
  try {
    const info = await setRoleInteractionMode(roleStore.currentRoleId, v)
    roleStore.applyRoleInfo(info)
    if (v === 'pure_chat') {
      postReplySceneBarVisible.value = false
      postReplySceneSelectedId.value = ''
      togetherTravelBarVisible.value = false
      togetherTravelSelectedId.value = ''
      topBarSceneDialogVisible.value = false
      pendingTopBarSceneId.value = ''
      autonomousSceneNotice.value = {
        visible: false,
        fromLabel: '',
        toLabel: '',
      }
    }
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

async function onPluginSetRemoteLife(payload: unknown): Promise<void> {
  const enabledRaw = (payload as { enabled?: boolean } | null)?.enabled
  if (typeof enabledRaw !== 'boolean')
    return
  try {
    const info = await setRemoteLifeEnabled(roleStore.currentRoleId, enabledRaw)
    roleStore.applyRoleInfo(info)
    showToast('success', enabledRaw ? t('app.toast.remoteLifeOn') : t('app.toast.remoteLifeOff'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

async function onPluginSetInteractionMode(payload: unknown): Promise<void> {
  const mode = (payload as { mode?: string } | null)?.mode
  if (mode !== 'immersive' && mode !== 'pure_chat')
    return
  try {
    const info = await setRoleInteractionMode(roleStore.currentRoleId, mode)
    roleStore.applyRoleInfo(info)
    if (mode === 'pure_chat') {
      postReplySceneBarVisible.value = false
      postReplySceneSelectedId.value = ''
      togetherTravelBarVisible.value = false
      togetherTravelSelectedId.value = ''
      topBarSceneDialogVisible.value = false
      pendingTopBarSceneId.value = ''
      autonomousSceneNotice.value = {
        visible: false,
        fromLabel: '',
        toLabel: '',
      }
    }
    showToast(
      'success',
      mode === 'immersive'
        ? t('app.toast.interactionImmersive')
        : t('app.toast.interactionPureChat'),
    )
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

function onPluginCycleTheme(): void {
  cycleTheme()
}

async function onPluginResetLayout(): Promise<void> {
  try {
    await pluginStore.resetToRolePackDefault()
    const message = t('app.toast.layoutResetOk')
    hostEventBus.emit(settingsResetLayoutResultEvent, { ok: true, message })
    showToast('success', message)
  }
  catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    hostEventBus.emit(settingsResetLayoutResultEvent, {
      ok: false,
      message: t('app.toast.layoutResetFailPrefix') + message,
    })
    showToast('error', message)
  }
}

async function initialize() {
  try {
    await roleStore.loadRoles()
    if (!roleStore.currentRoleId.trim()) {
      showToast('error', t('app.toast.noRolesScanned'))
      return
    }
    await loadRole(roleStore.currentRoleId)
    await pluginStore.refresh()
    await roleStore.refreshRoleInfo()
    hostEventBus.emitBuiltin('role:switched', { roleId: roleStore.currentRoleId })
    applyResolvedNarrativeScene()
    await debugStore.loadDebugData()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

async function onSend(payload: { content: string }) {
  postReplySceneBarVisible.value = false
  postReplySceneSelectedId.value = ''
  togetherTravelBarVisible.value = false
  togetherTravelSelectedId.value = ''
  const userText = payload.content
  try {
    const res = await chatStore.sendMessage(userText, uiStore.sceneId)
    await roleStore.refreshRoleInfo()
    applyResolvedNarrativeScene()
    await debugStore.loadDebugData()
    if (res.reply_is_fallback) {
      showToast('info', t('app.toast.fallbackReply'))
    }
    const offerTogether = res.offer_together_travel ?? false
    const offerPicker = res.offer_destination_picker ?? false
    // 问卷：邀请同行条优先于「仅选目的地」条（与后端 movement_ui_flags 一致）
    if (offerTogether && sceneDestinationOptions.value.length > 0) {
      togetherTravelBarVisible.value = true
    }
    else if (offerPicker && sceneDestinationOptions.value.length > 0) {
      postReplySceneBarVisible.value = true
    }
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    chatInputRef.value?.focusInput?.()
  }
}

async function confirmPostReplyScene(together: boolean) {
  const id = postReplySceneSelectedId.value.trim()
  postReplySceneBarVisible.value = false
  postReplySceneSelectedId.value = ''
  await applySceneDestination(id, together)
}

function dismissPostReplySceneBar() {
  postReplySceneBarVisible.value = false
  postReplySceneSelectedId.value = ''
}

async function confirmTogetherTravel(together: boolean) {
  const id = togetherTravelSelectedId.value.trim()
  togetherTravelBarVisible.value = false
  togetherTravelSelectedId.value = ''
  await applySceneDestination(id, together)
}

function dismissTogetherTravelBar() {
  togetherTravelBarVisible.value = false
  togetherTravelSelectedId.value = ''
}

function onTopBarSceneChange(ev: Event) {
  const sel = ev.target as HTMLSelectElement
  const next = sel.value
  if (next === uiStore.sceneId)
    return
  const a = document.activeElement
  topBarSceneOpenerFocus.value = a instanceof HTMLElement ? a : null
  pendingTopBarSceneId.value = next
  topBarSceneDialogVisible.value = true
  sel.value = uiStore.sceneId
}

function dismissTopBarSceneDialog() {
  topBarSceneDialogVisible.value = false
  pendingTopBarSceneId.value = ''
  const el = topBarSceneOpenerFocus.value
  topBarSceneOpenerFocus.value = null
  void nextTick(() => el?.focus({ preventScroll: true }))
}

async function confirmTopBarScene(together: boolean) {
  const id = pendingTopBarSceneId.value.trim()
  topBarSceneDialogVisible.value = false
  pendingTopBarSceneId.value = ''
  const el = topBarSceneOpenerFocus.value
  topBarSceneOpenerFocus.value = null
  void nextTick(() => el?.focus({ preventScroll: true }))
  await applySceneDestination(id, together)
}

function onPluginQuickActionTravel(payload: unknown): void {
  const sceneId = (payload as { sceneId?: string } | null)?.sceneId
  const togetherRaw = (payload as { together?: boolean } | null)?.together
  const id = typeof sceneId === 'string' ? sceneId.trim() : ''
  if (!id)
    return
  if (!allSceneOptions.value.some(s => s.id === id))
    return
  const together = togetherRaw === true
  void applySceneDestination(id, together)
}

async function onSwitchRole(nextRoleId: string) {
  const savedLeftScroll = leftPaneRef.value?.scrollTop ?? 0
  try {
    roleSwitching.value = true
    await roleStore.switchRole(nextRoleId)
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
        if (pane) {
          pane.scrollTop = savedLeftScroll
        }
      })
    }, 220)
  }
}

async function onChangeRelation(nextRelation: string) {
  try {
    const perScene = roleStore.roleInfo.identityBinding === 'per_scene'
    if (nextRelation === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) {
        await roleStore.setManifestDefaultIdentity(uiStore.sceneId)
      }
      else {
        await roleStore.setManifestDefaultIdentity()
      }
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

function onVirtualTimeJumpComplete(res: JumpTimeResponse): void {
  if (res.autonomous_scene_from && res.autonomous_scene_to) {
    autonomousSceneNotice.value = {
      visible: true,
      fromLabel: sceneLabelForId(res.autonomous_scene_from),
      toLabel: sceneLabelForId(res.autonomous_scene_to),
    }
  }
}

function dismissAutonomousSceneNotice(): void {
  autonomousSceneNotice.value = { visible: false, fromLabel: '', toLabel: '' }
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

function onHotkey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (simplePluginManagerOpen.value) {
      e.preventDefault()
      simplePluginManagerOpen.value = false
      return
    }
    if (shortcutHelpOpen.value) {
      e.preventDefault()
      shortcutHelpOpen.value = false
      return
    }
    if (pluginStore.marketPanelVisible) {
      e.preventDefault()
      pluginStore.closeMarketPanel()
      return
    }
    if (settingsViewOpen.value) {
      e.preventDefault()
      settingsViewOpen.value = false
      return
    }
    if (topMoreOpen.value) {
      e.preventDefault()
      topMoreOpen.value = false
      return
    }
    if (debugStore.visible) {
      e.preventDefault()
      debugStore.toggle()
      return
    }
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'f') {
    e.preventDefault()
    openPluginManagerPanel()
    return
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 's') {
    e.preventDefault()
    openSettingsView()
    return
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'd') {
    e.preventDefault()
    debugStore.toggle()
  }
}

watch(
  messages,
  async () => {
    await nextTick()
    chatListRef.value?.scrollToBottom?.()
  },
  { flush: 'post' },
)

watch(
  () => debugStore.visible,
  (v) => {
    if (v)
      void debugStore.loadDebugData()
  },
)

let unlistenPluginFs: (() => void) | undefined
let unlistenProtocolInstall: (() => void) | undefined

async function runPendingProtocolInstallsFromQueue(): Promise<void> {
  try {
    const pending = await consumePendingProtocolInstalls()
    for (const p of pending) {
      const git = p.gitUrl?.trim()
      if (!git)
        continue
      try {
        const r = await installPluginFromGit(git)
        showToast('success', t('app.toast.pluginInstalledFromWeb', { id: r.installedPluginId }))
        await pluginStore.refresh()
        openPluginManagerPanel()
      }
      catch (e) {
        showToast('error', e instanceof Error ? e.message : String(e))
      }
    }
  }
  catch (e) {
    console.warn('consume_pending_protocol_installs', e)
  }
}

watch(locale, () => {
  syncBrowserChromeFromLocale()
})

onMounted(() => {
  localePreference.value = getLocalePreference()
  syncBrowserChromeFromLocale()
  setErrorReporter((err) => {
    showToast('error', err.message)
  })
  hostEventBus.on(quickActionTravelEvent, onPluginQuickActionTravel)
  hostEventBus.on(settingsSetRemoteLifeEvent, onPluginSetRemoteLife)
  hostEventBus.on(settingsSetInteractionModeEvent, onPluginSetInteractionMode)
  hostEventBus.on(settingsCycleThemeEvent, onPluginCycleTheme)
  hostEventBus.on(settingsResetLayoutEvent, onPluginResetLayout)
  window.addEventListener('keydown', onHotkey)
  window.addEventListener('keydown', onCtrlHoldHintKeydown)
  window.addEventListener('keyup', onCtrlHoldHintKeyup)
  window.addEventListener('resize', scheduleRefreshSplitLayout)
  refreshSplitLayout()
  initialize()
  void listen('plugin:changed', () => {
    void pluginStore.onPluginFilesChanged().then(() => {
      showToast('success', t('app.toast.pluginFilesChanged'))
    })
  }).then((u) => {
    unlistenPluginFs = u
  })

  void listen('protocol:pending_install', () => {
    void runPendingProtocolInstallsFromQueue()
  }).then((u) => {
    unlistenProtocolInstall = u
  })

  void runPendingProtocolInstallsFromQueue()
})

onBeforeUnmount(() => {
  if (splitLayoutResizeRaf !== 0) {
    cancelAnimationFrame(splitLayoutResizeRaf)
    splitLayoutResizeRaf = 0
  }
  setErrorReporter(null)
  window.removeEventListener('keydown', onHotkey)
  hostEventBus.off(quickActionTravelEvent, onPluginQuickActionTravel)
  hostEventBus.off(settingsSetRemoteLifeEvent, onPluginSetRemoteLife)
  hostEventBus.off(settingsSetInteractionModeEvent, onPluginSetInteractionMode)
  hostEventBus.off(settingsCycleThemeEvent, onPluginCycleTheme)
  hostEventBus.off(settingsResetLayoutEvent, onPluginResetLayout)
  window.removeEventListener('keydown', onCtrlHoldHintKeydown)
  window.removeEventListener('keyup', onCtrlHoldHintKeyup)
  window.removeEventListener('resize', scheduleRefreshSplitLayout)
  clearCtrlLongPressTimer()
  unlistenPluginFs?.()
  unlistenProtocolInstall?.()
})
</script>

<template>
  <main class="layout">
    <div class="app-frame">
      <!-- 对齐 oclive-new：顶栏角色 + 时间/场景 -->
      <header class="top-bar">
        <TopBarMorePanel
          v-model="topMoreOpen"
          v-model:locale-preference="localePreference"
          :relation-options="relationOptions"
          :all-scene-options="allSceneOptions"
          :settings-entry-more-help="settingsEntryMoreHelp"
          :plugin-manager-more-btn-label="pluginManagerMoreBtnLabel"
          @open-settings="openSettingsView"
          @open-shortcut-help="openShortcutHelp"
          @open-plugin-manager="openPluginManagerPanel"
          @open-plugin-market="openPluginMarket"
          @scene-change="onTopBarSceneChange"
          @interaction-mode-change="onInteractionModeChange"
          @change-role="onSwitchRole"
          @change-relation="onChangeRelation"
          @notify="(p) => showToast(p.type, p.message)"
          @virtual-time-jump-complete="onVirtualTimeJumpComplete"
        >
          <template #leading>
            <RoleSelector
              variant="topbar"
              :sections="['role']"
              :current-role-id="roleStore.currentRoleId"
              :current-relation="roleStore.relationSelectValue"
              :roles="roleStore.roles"
              :relations="relationOptions"
              :loading="chatStore.isLoading"
              @change-role="onSwitchRole"
              @change-relation="onChangeRelation"
            />
          </template>
        </TopBarMorePanel>
      </header>

      <div
        v-if="uiStore.connectivityBanner?.kind === 'plugin_index_offline'"
        class="connectivity-banner"
        role="status"
      >
        <div class="connectivity-banner__inner">
          <p class="connectivity-banner__title">
            {{ t("app.connectivity.pluginIndexOffline") }}
          </p>
          <p v-if="connectivityPluginIndexDetail" class="connectivity-banner__detail">
            {{ connectivityPluginIndexDetail }}
          </p>
          <button
            type="button"
            class="connectivity-banner__dismiss"
            @click="uiStore.dismissConnectivityBanner()"
          >
            {{ t("app.connectivity.dismiss") }}
          </button>
        </div>
      </div>

      <div
        v-if="roleStore.interactionImmersive && sceneTransition.visible"
        class="scene-transition-overlay"
        role="status"
        aria-live="polite"
      >
        {{ t("app.sceneTransition.going", { label: sceneTransition.label }) }}
      </div>

      <TopBarSceneModeDialog
        v-if="roleStore.interactionImmersive"
        :visible="topBarSceneDialogVisible"
        :pending-scene-label="sceneLabelForId(pendingTopBarSceneId)"
        @confirm="confirmTopBarScene"
        @dismiss="dismissTopBarSceneDialog"
      />

      <div class="main-content">
        <div
          class="split-row"
          :class="{
            'split-row--narrow': !wideSplitLayout,
            'split-row--sidebar-right': sidebarRight,
          }"
        >
          <aside ref="leftPaneRef" class="left-pane">
            <RoleDetailView
              class="character-block"
              :layout="wideSplitLayout ? 'sidebar' : 'stack'"
              :role-id="roleStore.currentRoleId"
              :name="roleName"
              :emotion="emotion"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
            />
            <RoleplayAsidePanel :text="latestRoleplayAside" />
            <PluginSidebarSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
            <div class="left-pane-status" :aria-label="t('app.sidebar.favorability')">
              {{ t("app.sidebar.favorability") }} {{ Math.round(roleStore.roleInfo.favorability) }} {{ statusHeart }}
            </div>
            <div
              v-if="roleStore.interactionImmersive && roleStore.roleInfo.currentLife?.label"
              class="left-pane-life"
              :aria-label="t('app.sidebar.scheduleInference')"
            >
              {{ t("app.sidebar.lifeNow", { label: roleStore.roleInfo.currentLife?.label }) }}
            </div>
            <AutonomousSceneNotice
              v-if="roleStore.interactionImmersive"
              :visible="autonomousSceneNotice.visible"
              :from-label="autonomousSceneNotice.fromLabel"
              :to-label="autonomousSceneNotice.toLabel"
              @dismiss="dismissAutonomousSceneNotice"
            />
          </aside>
          <div class="right-pane" :class="{ 'right-pane--input-top': chatInputTop }">
            <PluginChatHeaderSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
            <div class="chat-scroll-wrap chat-list">
              <transition name="fade">
                <ChatMessageList
                  ref="chatListRef"
                  :key="`${roleStore.currentRoleId}-${uiStore.sceneId}`"
                  :messages="messages"
                  :history-split-index="sceneHistorySplitIndex"
                  :loading="chatStore.isLoading"
                  :role-switching="roleSwitching"
                />
              </transition>
            </div>
            <section class="input-area">
              <ChatPluginToolbarSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
              <SceneTravelBars
                v-if="roleStore.interactionImmersive"
                :together-visible="togetherTravelBarVisible"
                :post-reply-visible="postReplySceneBarVisible"
                :destination-options="sceneDestinationOptions"
                :together-selected-id="togetherTravelSelectedId"
                :post-reply-selected-id="postReplySceneSelectedId"
                @update:together-selected-id="togetherTravelSelectedId = $event"
                @update:post-reply-selected-id="postReplySceneSelectedId = $event"
                @confirm-together="confirmTogetherTravel"
                @dismiss-together="dismissTogetherTravelBar"
                @confirm-post-reply="confirmPostReplyScene"
                @dismiss-post-reply="dismissPostReplySceneBar"
              />
              <ChatInput ref="chatInputRef" :loading="chatStore.isLoading" @send="onSend" />
            </section>
          </div>
        </div>
      </div>

      <DebugPanel
        :visible="debugStore.visible"
        :loading="chatStore.isLoading"
        :favorability="roleStore.roleInfo.favorability"
        :personality="roleStore.roleInfo.personality ?? []"
        :events="debugStore.events"
        :memories="debugStore.memories"
        @reload="onReloadPolicy"
        @refresh="debugStore.loadDebugData"
        @close="debugStore.toggle"
        @notify="(p) => showToast(p.type, p.message)"
        @imported="onPackImported"
      />

      <Toast :show="toast.show" :type="toast.type" :message="toast.message" />
      <ShortcutHelp v-model="shortcutHelpOpen" :bootstrap-epoch="pluginStore.bootstrapEpoch" />

      <MarketView />
      <SimplePluginManagerPanel
        :visible="simplePluginManagerOpen"
        @close="simplePluginManagerOpen = false"
        @open-market="openPluginMarket"
      />

      <SettingsView
        :visible="settingsViewOpen"
        @close="settingsViewOpen = false"
      />

      <div class="app-floating-slot" aria-hidden="true">
        <PluginSlotEmbed
          slot-name="overlay.floating"
          :aria-label="t('app.floatingSlot')"
          :bootstrap-epoch="pluginStore.bootstrapEpoch"
        />
      </div>
      <HotkeyHost />
    </div>
  </main>
</template>

<style scoped>
/* 占满视口：宽度随窗口拉伸，避免两侧大块留白 */
.layout {
  flex: 1;
  min-height: 0;
  width: 100%;
  display: flex;
  justify-content: stretch;
  align-items: stretch;
  padding: 6px 8px;
  background: var(--shell-page-bg);
  box-sizing: border-box;
  overflow: hidden;
}
/* 单卡外壳：圆角与阴影保留，横向铺满可用区域 */
.app-frame {
  width: 100%;
  max-width: 100%;
  height: 100%;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg-primary);
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-app), var(--frame-inset-highlight);
  overflow: hidden;
}
.connectivity-banner {
  flex-shrink: 0;
  padding: 8px 14px;
  background: color-mix(in srgb, var(--accent, #3b82f6) 10%, var(--bg-elevated));
  border-bottom: 1px solid var(--border-light);
}
.connectivity-banner__inner {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 8px 12px;
}
.connectivity-banner__title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.4;
  color: var(--text-primary);
}
.connectivity-banner__detail {
  margin: 0;
  width: 100%;
  font-size: 11px;
  line-height: 1.35;
  color: var(--text-secondary);
  word-break: break-word;
}
.connectivity-banner__dismiss {
  flex-shrink: 0;
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.connectivity-banner__dismiss:hover {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 35%, var(--border-light));
}
.app-floating-slot {
  position: fixed;
  right: 12px;
  bottom: 12px;
  z-index: 10020;
  max-width: min(400px, calc(100vw - 24px));
  pointer-events: none;
}
.app-floating-slot :deep(.pse) {
  pointer-events: auto;
}
.top-bar {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0;
  padding: 10px 14px 12px;
  background: color-mix(in srgb, var(--bg-secondary) 92%, var(--rail-accent-runtime-bg) 8%);
  border-bottom: 1px solid var(--border-light);
  border-left: 3px solid var(--rail-accent-runtime);
  box-shadow: 0 1px 0 color-mix(in srgb, var(--accent) 12%, transparent);
}
.time-section {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--text-secondary);
}
.scene-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.scene-row-label {
  color: var(--text-secondary);
  font-weight: 600;
  white-space: nowrap;
}
.scene-select {
  min-width: 120px;
  max-width: 200px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--bg-elevated);
}
.scene-select:focus {
  outline: none;
}
.scene-select:focus-visible {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.scene-row-hint {
  font-size: 11px;
  opacity: 0.9;
  max-width: 140px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
/* 左：立绘 + 好感；右：历史 + 输入（历史区域显著变宽） */
.split-row {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
}
.split-row--narrow {
  flex-direction: column;
}
/* 宽屏：立绘在右；窄屏：对话在上、立绘在下 */
.split-row--sidebar-right:not(.split-row--narrow) {
  flex-direction: row-reverse;
}
.split-row--sidebar-right:not(.split-row--narrow) .left-pane {
  border-right: none;
  border-left: 1px solid var(--border-light);
  box-shadow: inset 1px 0 0 color-mix(in srgb, var(--border-light) 65%, transparent);
}
.split-row--sidebar-right.split-row--narrow {
  flex-direction: column-reverse;
}
.left-pane {
  flex: 0 0 clamp(248px, 28vw, 360px);
  max-width: 40%;
  min-width: 220px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  border-right: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-secondary) 96%, var(--accent) 4%);
  box-shadow: inset -1px 0 0 color-mix(in srgb, var(--border-light) 65%, transparent);
}
.split-row--narrow .left-pane {
  flex: 0 0 auto;
  width: 100%;
  max-width: none;
  min-width: 0;
  border-right: none;
  border-bottom: 1px solid var(--border-light);
  max-height: min(52vh, 520px);
}
.character-block {
  flex-shrink: 0;
}
.left-pane-status {
  flex-shrink: 0;
  margin-top: auto;
  padding: 10px 12px 14px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  border-top: 1px solid var(--border-light);
  background: var(--bg-status);
}
.left-pane-life {
  flex-shrink: 0;
  padding: 0 12px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  line-height: 1.4;
}
.right-pane {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
.right-pane--input-top {
  flex-direction: column-reverse;
}
/* 聊天记录仅在右侧栏滚动；底部多留空，避免气泡+阴影被输入区视觉上压住 */
.chat-scroll-wrap {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px 18px max(52px, calc(32px + env(safe-area-inset-bottom, 0px)));
  scroll-padding-bottom: 44px;
  background: var(--bg-primary);
  -webkit-overflow-scrolling: touch;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}
.input-area {
  flex-shrink: 0;
  position: relative;
  z-index: 1;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
  /* 略收阴影，减少「盖住最后一泡」的错觉 */
  box-shadow: 0 -2px 14px color-mix(in srgb, var(--text-primary) 8%, transparent);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 220ms ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0.82;
}
.scene-transition-overlay {
  flex-shrink: 0;
  width: 100%;
  padding: 10px 16px;
  text-align: center;
  font-size: 14px;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-secondary) 88%, transparent);
  border-bottom: 1px solid var(--border-light);
  box-shadow: var(--shadow-sm);
  z-index: 2;
}
</style>
