<script setup lang="ts">
import { inject, ref } from 'vue'
import AutonomousSceneNotice from '../../components/AutonomousSceneNotice.vue'
import ChatInput from '../../components/chat/ChatInput.vue'
import ChatMessageList from '../../components/chat/ChatMessageList.vue'
import ChatPluginToolbarSlots from '../../components/ChatPluginToolbarSlots.vue'
import HotkeyHost from '../../components/hotkey/HotkeyHost.vue'
import PluginChatHeaderSlots from '../../components/PluginChatHeaderSlots.vue'
import PluginSidebarSlots from '../../components/PluginSidebarSlots.vue'
import PluginSlotEmbed from '../../components/PluginSlotEmbed.vue'
import RoleIdentityControls from '../../components/role/RoleIdentityControls.vue'
import RoleSelector from '../../components/role/RoleSelector.vue'
import RoleplayAsidePanel from '../../components/RoleplayAsidePanel.vue'
import TopBarSceneModeDialog from '../../components/scene/TopBarSceneModeDialog.vue'
import SceneTravelBars from '../../components/SceneTravelBars.vue'
import ShortcutHelp from '../../components/ShortcutHelp.vue'
import Toast from '../../components/Toast.vue'
import UiResizeHandle from '../../components/ui/UiResizeHandle.vue'
import { MAIN_SHELL_KEY } from '../../composables/mainShellKey'
import {
  getLayoutWidths,
  setLeftRailWidth,
  setSidePanelWidth,
} from '../../composables/useLayoutWidths'
import {
  DebugPanel,
  MarketView,
} from '../../composables/useMainShell'
import RoleDetailView from '../../views/RoleDetailView.vue'
import ToolActivityBar from './ToolActivityBar.vue'
import ToolMoreMenu from './ToolMoreMenu.vue'
import ToolSidePanelHost from './ToolSidePanelHost.vue'
import ToolStatusBar from './ToolStatusBar.vue'

const shell = inject(MAIN_SHELL_KEY)
if (!shell) {
  throw new Error('ToolShell requires MAIN_SHELL_KEY provider')
}

const {
  t,
  toast,
  showToast,
  roleStore,
  chatStore,
  debugStore,
  uiStore,
  pluginStore,
  chatListRef,
  chatInputRef,
  leftPaneRef,
  roleSwitching,
  relationOptions,
  connectivityPluginIndexDetail,
  messages,
  chatListLoading,
  latestRoleplayAside,
  topMoreOpen,
  settingsViewOpen,
  simplePluginManagerOpen,
  pluginsPanelSubview,
  modelManagerOpen,
  closeModelManager,
  sidePanelOpen,
  sidePanelTab,
  closeAllSidePanels,
  onSidePanelTabChange,
  openSettingsView,
  settingsFocusTab,
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
  onVirtualTimeJumpComplete,
  dismissAutonomousSceneNotice,
  shortcutHelpOpen,
  openShortcutHelp,
  sceneTransition,
  sceneLabelForId,
  sceneHistorySplitIndex,
  sidebarRight,
  chatInputTop,
  roleName,
  emotion,
  statusHeart,
  onSend,
  onSwitchRole,
  onChangeRelation,
  onPackImported,
  onReloadPolicy,
  onDebugRefresh,
} = shell

const roleRailOpen = ref(false)

function onFocusChat() {
  closeAllSidePanels()
  topMoreOpen.value = false
}

function onToggleRoleRail() {
  roleRailOpen.value = !roleRailOpen.value
}

function onOpenSettings() {
  onSidePanelTabChange('settings')
}

function onOpenPlugins() {
  onSidePanelTabChange('plugins')
}

function onOpenModels() {
  onSidePanelTabChange('models')
}

function onCloseSidePanel() {
  closeAllSidePanels()
}

function onOpenSettingsFromModels() {
  closeModelManager()
  openSettingsView()
}

const layoutWidths = getLayoutWidths()
let leftRailWidth = layoutWidths.leftRail
let sidePanelWidth = layoutWidths.sidePanel

function onLeftRailResize(deltaX: number) {
  leftRailWidth = setLeftRailWidth(leftRailWidth + deltaX)
}

function onSidePanelResize(deltaX: number) {
  sidePanelWidth = setSidePanelWidth(sidePanelWidth - deltaX)
}
</script>

<template>
  <main class="tool-layout">
    <div class="tool-frame">
      <ToolActivityBar
        :role-rail-open="roleRailOpen"
        :settings-active="settingsViewOpen"
        :plugins-active="simplePluginManagerOpen"
        :models-active="modelManagerOpen"
        :more-open="topMoreOpen"
        @focus-chat="onFocusChat"
        @toggle-role-rail="onToggleRoleRail"
        @open-settings="onOpenSettings"
        @open-plugins="onOpenPlugins"
        @open-models="onOpenModels"
        @toggle-more="topMoreOpen = !topMoreOpen"
      />

      <div class="tool-body">
        <div class="tool-body__main">
          <header class="tool-top-bar">
            <RoleSelector
              variant="topbar"
              :sections="['role', 'relation']"
              :current-role-id="roleStore.currentRoleId"
              :current-relation="roleStore.relationSelectValue"
              :roles="roleStore.roles"
              :relations="relationOptions"
              :loading="chatListLoading"
              @change-role="onSwitchRole"
              @change-relation="onChangeRelation"
            />
            <ToolMoreMenu
              v-model="topMoreOpen"
              :all-scene-options="allSceneOptions"
              @open-shortcut-help="openShortcutHelp"
              @scene-change="onTopBarSceneChange"
              @notify="(p) => showToast(p.type, p.message)"
              @virtual-time-jump-complete="onVirtualTimeJumpComplete"
            />
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

          <div class="tool-workspace">
            <aside
              v-show="roleRailOpen"
              ref="leftPaneRef"
              class="tool-left-rail"
              :class="{ 'tool-left-rail--right': sidebarRight }"
            >
              <RoleDetailView
                class="character-block"
                layout="sidebar"
                :role-id="roleStore.currentRoleId"
                :name="roleName"
                :emotion="emotion"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
              <RoleplayAsidePanel :text="latestRoleplayAside" />
              <PluginSidebarSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
              <div class="tool-left-rail__status" :aria-label="t('app.sidebar.favorability')">
                {{ t("app.sidebar.favorability") }} {{ Math.round(roleStore.roleInfo.favorability) }} {{ statusHeart }}
              </div>
              <RoleIdentityControls variant="compact" />
              <div
                v-if="roleStore.interactionImmersive && roleStore.roleInfo.currentLife?.label"
                class="tool-left-rail__life"
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

            <UiResizeHandle
              v-if="roleRailOpen"
              edge="left"
              :aria-label="t('settings.layoutResizeLeftRail')"
              @resize="onLeftRailResize"
            />

            <div class="tool-main" :class="{ 'tool-main--input-top': chatInputTop }">
              <PluginChatHeaderSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
              <div class="tool-chat-scroll chat-list">
                <transition name="fade">
                  <ChatMessageList
                    ref="chatListRef"
                    :key="`${roleStore.currentRoleId}-${uiStore.sceneId}`"
                    :messages="messages"
                    :history-split-index="sceneHistorySplitIndex"
                    :loading="chatListLoading"
                    :role-switching="roleSwitching"
                  />
                </transition>
              </div>
              <section class="tool-input-area">
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

          <ToolStatusBar :status-heart="statusHeart" :scene-label-for-id="sceneLabelForId" />
        </div>

        <UiResizeHandle
          v-if="sidePanelOpen"
          edge="right"
          :aria-label="t('settings.layoutResizeSidePanel')"
          @resize="onSidePanelResize"
        />

        <ToolSidePanelHost
          :open="sidePanelOpen"
          :active-tab="sidePanelTab"
          :plugins-subview="pluginsPanelSubview"
          :settings-focus-tab="settingsFocusTab"
          @close="onCloseSidePanel"
          @update:active-tab="onSidePanelTabChange"
          @update:plugins-subview="pluginsPanelSubview = $event"
          @open-settings-from-models="onOpenSettingsFromModels"
        />
      </div>

      <DebugPanel
        :visible="debugStore.visible"
        :loading="chatStore.isLoading"
        :favorability="roleStore.roleInfo.favorability"
        :personality="roleStore.roleInfo.personality ?? []"
        :events="debugStore.events"
        :memories="debugStore.memories"
        @reload="onReloadPolicy"
        @refresh="onDebugRefresh"
        @close="debugStore.toggle"
        @notify="(p) => showToast(p.type, p.message)"
        @imported="onPackImported"
      />

      <Toast :show="toast.show" :type="toast.type" :message="toast.message" />
      <ShortcutHelp v-model="shortcutHelpOpen" :bootstrap-epoch="pluginStore.bootstrapEpoch" />
      <MarketView />

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
.tool-layout {
  flex: 1;
  min-height: 0;
  width: 100%;
  display: flex;
  background: var(--tool-bg, var(--shell-page-bg));
  box-sizing: border-box;
  overflow: hidden;
}

.tool-frame {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.tool-body {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.tool-body__main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.tool-top-bar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--tool-space-2, 8px);
  min-height: var(--tool-topbar-h, 36px);
  padding: 0 var(--tool-space-3, 12px);
  border-bottom: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.tool-workspace {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.tool-left-rail {
  flex: 0 0 var(--tool-left-rail-w, 220px);
  width: var(--tool-left-rail-w, 220px);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: auto;
  border-right: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-sidebar, var(--tool-bg, var(--bg-secondary)));
}

.tool-left-rail--right {
  order: 2;
  border-right: none;
  border-left: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}

.tool-left-rail__status {
  flex-shrink: 0;
  margin-top: auto;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px);
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  text-align: center;
  border-top: 1px solid var(--tool-border, var(--border-light));
}

.tool-left-rail__life {
  flex-shrink: 0;
  padding: 0 var(--tool-space-3, 12px) var(--tool-space-3, 12px);
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  text-align: center;
}

.tool-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.tool-main--input-top {
  flex-direction: column-reverse;
}

.tool-chat-scroll {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: var(--tool-space-4, 16px) var(--tool-space-6, 24px) 0;
}

.tool-chat-scroll :deep(.chat-list-root) {
  flex: 1;
  min-height: 0;
}

.tool-input-area {
  flex-shrink: 0;
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.connectivity-banner {
  flex-shrink: 0;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px);
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 8%, var(--tool-elevated, var(--bg-elevated)));
  border-bottom: 1px solid var(--tool-border, var(--border-light));
}

.connectivity-banner__inner {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: var(--tool-space-2, 8px);
}

.connectivity-banner__title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: var(--tool-fs-md, 13px);
  font-weight: 600;
  color: var(--tool-text, var(--text-primary));
}

.connectivity-banner__detail {
  margin: 0;
  width: 100%;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  word-break: break-word;
}

.connectivity-banner__dismiss {
  flex-shrink: 0;
  padding: 2px 8px;
  font-size: var(--tool-fs-sm, 12px);
  border-radius: var(--tool-radius, 4px);
  border: 1px solid var(--tool-border, var(--border-light));
  background: var(--tool-elevated, var(--bg-primary));
  cursor: pointer;
}

.scene-transition-overlay {
  flex-shrink: 0;
  padding: var(--tool-space-2, 8px) var(--tool-space-4, 16px);
  text-align: center;
  font-size: var(--tool-fs-md, 13px);
  color: var(--tool-text, var(--text-primary));
  background: var(--tool-bg, var(--bg-secondary));
  border-bottom: 1px solid var(--tool-border, var(--border-light));
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 120ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0.82;
}

.app-floating-slot {
  position: fixed;
  right: 12px;
  bottom: calc(var(--tool-statusbar-h, 24px) + 12px);
  z-index: 10020;
  max-width: min(400px, calc(100vw - 24px));
  pointer-events: none;
}

.app-floating-slot :deep(.pse) {
  pointer-events: auto;
}

.character-block {
  flex-shrink: 0;
}
</style>
