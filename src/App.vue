<script setup lang="ts">
import AutonomousSceneNotice from './components/AutonomousSceneNotice.vue'
import ChatInput from './components/chat/ChatInput.vue'
import ChatMessageList from './components/chat/ChatMessageList.vue'
import ChatPluginToolbarSlots from './components/ChatPluginToolbarSlots.vue'
import HotkeyHost from './components/hotkey/HotkeyHost.vue'
import KernelStatusBar from './components/KernelStatusBar.vue'
import PluginChatHeaderSlots from './components/PluginChatHeaderSlots.vue'
import PluginSidebarSlots from './components/PluginSidebarSlots.vue'
import PluginSlotEmbed from './components/PluginSlotEmbed.vue'
import RoleSelector from './components/role/RoleSelector.vue'
import RoleplayAsidePanel from './components/RoleplayAsidePanel.vue'
import TopBarSceneModeDialog from './components/scene/TopBarSceneModeDialog.vue'
import SceneTravelBars from './components/SceneTravelBars.vue'
import ShortcutHelp from './components/ShortcutHelp.vue'
import Toast from './components/Toast.vue'
import TopBarMorePanel from './components/TopBarMorePanel.vue'
import {
  DebugPanel,
  MarketView,
  ModelManagerPanel,
  SettingsView,
  SimplePluginManagerPanel,
  useMainShell,
} from './composables/useMainShell'
import RoleDetailView from './views/RoleDetailView.vue'

const {
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
  openPluginManagerPanel,
  openSimplePluginManager,
  openPluginMarket,
  modelManagerOpen,
  openModelManager,
  closeModelManager,
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
  sceneTransition,
  sceneLabelForId,
  sceneHistorySplitIndex,
  sidebarRight,
  chatInputTop,
  roleName,
  emotion,
  statusHeart,
  onInteractionModeChange,
  onSend,
  onSwitchRole,
  onChangeRelation,
  onPackImported,
  onReloadPolicy,
} = useMainShell()
</script>

<template>
  <main class="layout">
    <div class="app-frame">
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
              :loading="chatListLoading"
              @change-role="onSwitchRole"
              @change-relation="onChangeRelation"
            />
            <KernelStatusBar class="top-bar-kernel-status" />
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
                  :loading="chatListLoading"
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
.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
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
