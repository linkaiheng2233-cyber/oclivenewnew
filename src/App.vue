<script setup lang="ts">
import type { LocalePreference } from './i18n'
import { listen } from '@tauri-apps/api/event'
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AutonomousSceneNotice from './components/AutonomousSceneNotice.vue'
import ChatInput from './components/chat/ChatInput.vue'
import ChatMessageList from './components/chat/ChatMessageList.vue'
import ChatPluginToolbarSlots from './components/ChatPluginToolbarSlots.vue'
import HotkeyHost from './components/hotkey/HotkeyHost.vue'
import PluginChatHeaderSlots from './components/PluginChatHeaderSlots.vue'
import PluginSidebarSlots from './components/PluginSidebarSlots.vue'
import PluginSlotEmbed from './components/PluginSlotEmbed.vue'
import RoleplayAsidePanel from './components/RoleplayAsidePanel.vue'
import RoleSelector from './components/role/RoleSelector.vue'
import SceneTravelBars from './components/SceneTravelBars.vue'
import ShortcutHelp from './components/ShortcutHelp.vue'
import TopBarMorePanel from './components/TopBarMorePanel.vue'
import Toast from './components/Toast.vue'
import TopBarSceneModeDialog from './components/scene/TopBarSceneModeDialog.vue'
import { useAppToast } from './composables/useAppToast'
import { useNarrativeScene } from './composables/useNarrativeScene'
import { useGlobalHotkeys } from './composables/useGlobalHotkeys'
import { usePluginEvents } from './composables/usePluginEvents'
import { usePluginManagerWindow } from './composables/usePluginManagerWindow'
import { useReturnFocusOnClose } from './composables/useReturnFocusOnClose'
import { useSceneTravelBars } from './composables/useSceneTravelBars'
import { useSceneDestination } from './composables/useSceneDestination'
import { usePackUiTheme } from './composables/useTheme'
import {
  getLocalePreference,
} from './i18n'
import { hostEventBus } from './lib/hostEventBus'
import { useChatStore } from './stores/chatStore'
import { useDebugStore } from './stores/debugStore'
import { useModelManagerWindow } from './composables/useModelManagerWindow'
import { usePluginMarketStore } from './stores/pluginMarketStore'
import { usePluginStore } from './stores/pluginStore'
import { usePluginTraceStore } from './stores/pluginTraceStore'
import { useRoleStore } from './stores/roleStore'
import { useUiStore } from './stores/uiStore'
import { buildRelationDropdownOptions } from './utils/relationOptions'
import {
  consumePendingProtocolInstalls,
  installPluginFromGit,

  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setErrorReporter,
  setRoleInteractionMode,
  setUserRelation,
} from './api'
import RoleDetailView from './views/RoleDetailView.vue'

const DebugPanel = defineAsyncComponent(() => import('./components/dev-tools/DebugPanel.vue'))
const MarketView = defineAsyncComponent(() => import('./views/MarketView.vue'))
const SettingsView = defineAsyncComponent(() => import('./views/SettingsView.vue'))
const ModelManagerPanel = defineAsyncComponent(() => import('./views/ModelManagerPanel.vue'))
const SimplePluginManagerPanel = defineAsyncComponent(() => import('./views/SimplePluginManagerPanel.vue'))

const roleStore = useRoleStore()
usePackUiTheme()
const chatStore = useChatStore()
const debugStore = useDebugStore()
const uiStore = useUiStore()
const pluginStore = usePluginStore()
const traceStore = usePluginTraceStore()
const pluginMarketStore = usePluginMarketStore()
const { t, locale } = useI18n()

function syncBrowserChromeFromLocale(): void {
  document.title = t('app.documentTitle')
  document.documentElement.setAttribute('lang', locale.value === 'en-US' ? 'en' : 'zh-CN')
}
const localePreference = ref<LocalePreference>(getLocalePreference())

const { toast, showToast } = useAppToast()
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

const messages = computed(() =>
  chatStore.messagesForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
)

/** 本场景最近一条助手旁白/内心（O(1) 读取，见 chatStore.lastAssistantAside） */
const latestRoleplayAside = computed(() => {
  const roleId = roleStore.currentRoleId
  const sceneId = uiStore.sceneId || 'default'
  return chatStore.lastAssistantAsideFor(roleId, sceneId)
})

const topMoreOpen = ref(false)
const settingsViewOpen = ref(false)

const {
  simplePluginManagerOpen,
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
  }
})
watch(modelManagerOpen, (open) => {
  if (open) {
    simplePluginManagerOpen.value = false
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

const { shortcutHelpOpen, openShortcutHelp, openSettingsView } = useGlobalHotkeys({
  simplePluginManagerOpen,
  settingsViewOpen,
  topMoreOpen,
  marketPanelVisible: computed(() => pluginMarketStore.marketPanelVisible),
  modelManagerOpen,
  debugVisible: computed(() => debugStore.visible),
  openPluginManagerPanel,
  openModelManager: () => openModelManager(),
  toggleDebug: () => debugStore.toggle(),
  closeMarketPanel: () => pluginMarketStore.closeMarketPanel(),
  closeModelManager,
})

usePluginEvents({
  showToast,
  onQuickActionTravel: onPluginQuickActionTravel,
  onPureChatMode: resetPureChatSceneUi,
})

useReturnFocusOnClose(settingsViewOpen)
useReturnFocusOnClose(simplePluginManagerOpen)
useReturnFocusOnClose(modelManagerOpen)
useReturnFocusOnClose(shortcutHelpOpen)

watch(
  () => traceStore.simpleManagerOpenNonce,
  () => {
    openSimplePluginManager(true)
  },
)

function onHostOpenModelManager(): void {
  openModelManager(true)
}

function onHostOpenPluginManager(): void {
  openSimplePluginManager(true)
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
    if (v === 'pure_chat')
      resetPureChatSceneUi()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
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
  clearSceneBarsBeforeSend()
  const userText = payload.content
  try {
    const res = await chatStore.sendMessage(userText, uiStore.sceneId)
    await roleStore.refreshRoleInfo()
    applyResolvedNarrativeScene()
    await debugStore.loadDebugData()
    if (res.reply_is_fallback) {
      const detail = res.llm_fallback_reason?.trim()
      showToast('info', detail || t('app.toast.fallbackReply'))
    }
    offerSceneBarsAfterReply(
      res.offer_together_travel ?? false,
      res.offer_destination_picker ?? false,
    )
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    chatInputRef.value?.focusInput?.()
  }
}

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

async function onReloadPolicy() {
  try {
    const msg = await debugStore.reloadPolicy()
    showToast('success', msg)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

/** 仅在新消息入列时贴底；避免 messages 浅更新时把用户上滑阅读打回底部。 */
watch(
  () => messages.value.length,
  async (len, prev) => {
    if (prev !== undefined && len <= prev)
      return
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

let unlistenPluginFs: (() => void) | Promise<(() => void)> | undefined
let unlistenProtocolInstall: (() => void) | Promise<(() => void)> | undefined

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
  hostEventBus.on('ui:open_model_manager', onHostOpenModelManager)
  hostEventBus.on('ui:open_plugin_manager', onHostOpenPluginManager)
  localePreference.value = getLocalePreference()
  syncBrowserChromeFromLocale()
  setErrorReporter((err) => {
    showToast('error', err.message)
  })
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
  hostEventBus.off('ui:open_model_manager', onHostOpenModelManager)
  hostEventBus.off('ui:open_plugin_manager', onHostOpenPluginManager)
  if (splitLayoutResizeRaf !== 0) {
    cancelAnimationFrame(splitLayoutResizeRaf)
    splitLayoutResizeRaf = 0
  }
  setErrorReporter(null)
  window.removeEventListener('resize', scheduleRefreshSplitLayout)
  void disposeTauriListener(unlistenPluginFs)
  void disposeTauriListener(unlistenProtocolInstall)
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
          @open-settings="openSettingsView"
          @open-shortcut-help="openShortcutHelp"
          @open-plugin-manager="openPluginManagerPanel"
          @open-plugin-market="openPluginMarket"
          @open-model-manager="() => openModelManager(true)"
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

      <ModelManagerPanel
        :visible="modelManagerOpen"
        @close="closeModelManager"
        @open-settings="
          () => {
            closeModelManager()
            openSettingsView()
          }
        "
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
/* 滚动交给 ChatMessageList 内 VirtualScrollContainer，避免与外层双滚动抢滚轮 */
.chat-scroll-wrap {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 12px 18px 0;
  background: var(--bg-primary);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}
.chat-scroll-wrap :deep(.chat-list-root) {
  flex: 1;
  min-height: 0;
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
