<script setup lang="ts">
import type { useMainShell } from '../../composables/useMainShell'
import ChatInput from '@oclive/shared/components/chat/ChatInput.vue'
import ChatMessageList from '@oclive/shared/components/chat/ChatMessageList.vue'
import ErrorNotice from '@oclive/shared/components/ErrorNotice.vue'
import KernelStatusBar from '@oclive/shared/components/KernelStatusBar.vue'
import RenderErrorBoundary from '@oclive/shared/components/RenderErrorBoundary.vue'
import CharacterInfo from '@oclive/shared/components/role/CharacterInfo.vue'
import RoleIdentityControls from '@oclive/shared/components/role/RoleIdentityControls.vue'
import RolePackBar from '@oclive/shared/components/RolePackBar.vue'
import ShortcutHelp from '@oclive/shared/components/ShortcutHelp.vue'
import StartupWarningsBanner from '@oclive/shared/components/StartupWarningsBanner.vue'
import SupportDiagnostics from '@oclive/shared/components/SupportDiagnostics.vue'
import Toast from '@oclive/shared/components/Toast.vue'
import UiButton from '@oclive/shared/components/ui/UiButton.vue'
import UiCloseButton from '@oclive/shared/components/ui/UiCloseButton.vue'
import Win98TitleBar from '@oclive/shared/components/win98/Win98TitleBar.vue'
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { useEasterEggSkin } from '@oclive/shared/composables/useEasterEggSkin'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { useOcliveAppearance } from '@oclive/shared/composables/useOcliveAppearance'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { computed, inject, ref } from 'vue'
import ModelSetupGuide from '../../components/ModelSetupGuide.vue'
import MumuFeatureIcon from '../../components/MumuFeatureIcon.vue'
import { ModelManagerPanel, SettingsView } from '../../composables/useMainShell'
import { useRoleWorkspace } from '../../composables/useRoleWorkspace'

const shell = inject(MAIN_SHELL_KEY) as ReturnType<typeof useMainShell> | undefined
if (!shell)
  throw new Error('RoleWorkspaceShell requires MAIN_SHELL_KEY provider')
const {
  t,
  roleStore,
  chatStore,
  toast,
  showToast,
  startupState,
  startupError,
  startupErrorDetails,
  retryStartup,
  roleSwitching,
  sidebarRight,
  chatInputTop,
  roleName,
  emotion,
  portraitAssetRelPath,
  relationOptions,
  messages,
  chatListLoading,
  chatListRef,
  chatInputRef,
  activeChatKey,
  onAdultAction,
  onChangeRelation,
  openSettingsView,
  settingsViewOpen,
  settingsFocusTab,
  openModelManager,
  closeModelManager,
  modelManagerOpen,
  shortcutHelpOpen,
  openShortcutHelp,
  pluginStore,
} = shell
const {
  page,
  busy,
  error,
  capabilityError,
  recent,
  recentErrors,
  draft,
  activeSession,
  enterRole,
  openLibrary,
  onImported,
  refreshLibrary,
  submitMessage,
} = useRoleWorkspace(shell)
const helpOpen = ref(false)
const rolePanelOpen = ref(false)
const browserFixture = import.meta.env.MODE === 'e2e'
const helpCard = ref<HTMLElement | null>(null)
const helpOpener = ref<HTMLButtonElement | null>(null)
useModalFocusRestore(helpOpen, helpCard, { trapFocus: true })
const blocked = computed(() => busy.value || roleSwitching.value || startupState.value !== 'ready')
const runtimeReady = computed(() => page.value === 'chat' && !!activeSession.value && !!roleStore.currentRoleId)
const ui = useUiStore()
const initialProvider = ref<'local' | 'cloud' | undefined>()
const { effectiveTheme, toggleDarkTheme } = useOcliveAppearance()
const { win98Enabled } = useEasterEggSkin()
const isDark = computed(() => effectiveTheme.value === 'dark')

function openConnection(provider?: 'local' | 'cloud'): void {
  if (!runtimeReady.value || blocked.value) {
    showToast('info', t('workspace.setupSelectRoleFirst'))
    return
  }
  initialProvider.value = provider
  openModelManager(true)
}

function onModelSaved(roleId: string): void {
  if (roleId === roleStore.currentRoleId)
    ui.modelSetupDismissed = true
}

function openRecords(): void {
  if (!roleStore.currentRoleId) {
    showToast('info', t('workspace.selectRoleFirst'))
    return
  }
  settingsFocusTab.value = 'storage'
  settingsViewOpen.value = true
}

function sessionLabel(snippet: string): string {
  return snippet.trim().replace(/\s+/g, ' ').slice(0, 36) || t('workspace.untitled')
}

function notify(payload: { type: 'success' | 'error' | 'warning' | 'info', message: string, details?: string }): void {
  showToast(payload.type, payload.message, payload.details)
}

function openRecoveryFromNotice(): void {
  // Use a persistent opener so closing Help restores a useful keyboard position.
  helpOpener.value?.focus({ preventScroll: true })
  helpOpen.value = true
}
</script>

<template>
  <main class="role-app">
    <div v-if="browserFixture" class="workspace-error" role="status">
      {{ t('workspace.testPreview') }}
    </div>
    <Win98TitleBar />
    <div class="role-app-body">
      <nav class="role-nav" :aria-label="t('workspace.library')">
        <button class="role-brand" type="button" :disabled="blocked" @click="openLibrary">
          <span class="brand-mark">oc</span><span>OCLive<small>Chat Pro</small></span>
        </button>
        <button class="nav-button" :class="{ active: page === 'roles' }" type="button" :disabled="blocked" @click="openLibrary">
          <MumuFeatureIcon name="library" />{{ t('workspace.library') }}
        </button>
        <div class="nav-role-list">
          <p class="nav-label">
            {{ t('workspace.myRoles') }}
          </p>
          <section v-for="role in roleStore.roles" :key="role.id" class="nav-role">
            <div class="nav-role-title" :class="{ active: runtimeReady && roleStore.currentRoleId === role.id }">
              <button type="button" :disabled="blocked" @click="enterRole(role.id)">
                <span class="role-initial" aria-hidden="true"><CharacterInfo :role-id="role.id" :name="role.name" emotion="neutral" layout="thumbnail" /></span>
                <span class="nav-role-name" :title="role.name">{{ role.name }}</span>
              </button>
              <button type="button" :title="t('workspace.newChat')" :aria-label="`${role.name} · ${t('workspace.newChat')}`" :disabled="blocked" @click="enterRole(role.id, undefined, true)">
                +
              </button>
            </div>
            <div v-if="runtimeReady && roleStore.currentRoleId === role.id" class="nav-conversations">
              <p v-if="recentErrors[role.id]" class="inline-error">
                {{ t('workspace.recentFailed') }}
              </p>
              <button v-for="session in (recent[role.id] ?? []).slice(0, 5)" :key="session.session_id" type="button" :class="{ selected: activeSession?.session_id === session.session_id }" :disabled="blocked" @click="enterRole(role.id, session)">
                {{ sessionLabel(session.last_message_snippet) }}
              </button>
              <button v-if="activeSession?.message_count === 0" class="selected" type="button" disabled>
                {{ t('workspace.untitled') }}
              </button>
              <button class="all-records" type="button" @click="openRecords">
                {{ t('workspace.records') }}
              </button>
            </div>
          </section>
        </div>
        <div class="nav-global">
          <button class="nav-button" type="button" :disabled="blocked" @click="openConnection()">
            <MumuFeatureIcon name="model" />{{ t('workspace.model') }}
          </button>
          <button class="nav-button" type="button" @click="openSettingsView">
            <MumuFeatureIcon name="settings" />{{ t('workspace.settings') }}
          </button>
          <button ref="helpOpener" class="nav-button" type="button" @click="helpOpen = true">
            <MumuFeatureIcon name="help" />{{ t('workspace.help') }}
          </button>
          <small class="local-note">{{ t('workspace.local') }}</small>
        </div>
      </nav>

      <section class="workspace-content">
        <StartupWarningsBanner />
        <header class="workspace-header">
          <div>
            <h1>{{ runtimeReady ? t('workspace.runtime', { name: roleName }) : t('workspace.library') }}</h1>
            <p>{{ runtimeReady ? (roleStore.roleInfo.effectiveOllamaModel || t('workspace.noModel')) : t('workspace.libraryHint') }}</p>
          </div>
          <div class="workspace-header-actions">
            <button
              class="theme-toggle"
              type="button"
              role="switch"
              :aria-label="t('workspace.darkMode')"
              :aria-checked="isDark"
              :disabled="win98Enabled"
              :title="win98Enabled ? t('workspace.classicThemeHint') : t(isDark ? 'workspace.switchToLight' : 'workspace.switchToDark')"
              @click="toggleDarkTheme"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path v-if="isDark" d="M20.8 13.1A8.8 8.8 0 0 1 10.9 3.2 9 9 0 1 0 20.8 13.1Z" />
                <template v-else><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.4 1.4M17.6 17.6 19 19M5 19l1.4-1.4M17.6 6.4 19 5" /></template>
              </svg>
              <span>{{ t(isDark ? 'workspace.darkMode' : 'workspace.lightMode') }}</span>
            </button>
            <KernelStatusBar />
          </div>
        </header>

        <ModelSetupGuide v-if="startupState === 'ready'" :role-id="roleStore.currentRoleId" :ready="runtimeReady && !blocked" @configure="openConnection" />

        <div v-if="startupState === 'loading'" class="workspace-empty" role="status">
          <p>{{ t('workspace.loading') }}</p>
        </div>
        <div v-else-if="startupState === 'error'" class="workspace-empty" role="alert">
          <ErrorNotice :summary="startupError || t('workspace.startupFailed')" :details="startupErrorDetails" />
          <button class="primary" type="button" @click="retryStartup">
            {{ t('workspace.retry') }}
          </button>
        </div>
        <RenderErrorBoundary v-else>
          <ErrorNotice v-if="error" :error="error">
            <button type="button" @click="error = null">
              {{ t('workspace.close') }}
            </button>
          </ErrorNotice>
          <ErrorNotice v-if="capabilityError && !error" :error="capabilityError" :summary="t('workspace.capabilityFailed')">
            <button type="button" @click="refreshLibrary">
              {{ t('workspace.retry') }}
            </button>
          </ErrorNotice>
          <div v-if="page === 'roles'" class="library-page" role="region" :aria-label="t('workspace.library')" tabindex="0">
            <div class="library-tools">
              <h2>{{ t('workspace.myRoles') }} <small>{{ roleStore.roles.length }}</small></h2>
              <div class="library-actions">
                <UiButton :disabled="blocked" @click="retryStartup">
                  {{ t('workspace.retry') }}
                </UiButton>
                <RolePackBar v-if="roleStore.roles.length" :disabled="blocked" :can-export="false" @imported="onImported" @notify="notify" />
              </div>
            </div>
            <div v-if="roleStore.roles.length === 0" class="workspace-empty">
              <MumuFeatureIcon class="empty-role-icon" name="library" size="empty" />
              <h2>{{ t('workspace.emptyTitle') }}</h2><p>{{ t('workspace.emptyHint') }}</p>
              <RolePackBar :disabled="blocked" :can-export="false" @imported="onImported" @notify="notify" />
            </div>
            <div v-else class="role-card-grid">
              <article v-for="role in roleStore.roles" :key="role.id" class="role-card">
                <div class="card-portrait">
                  <CharacterInfo :role-id="role.id" :name="role.name" emotion="neutral" layout="preview" />
                </div>
                <div class="role-card-copy">
                  <h2>{{ role.name }}</h2>
                  <p class="role-description" :title="role.description">
                    {{ role.description }}
                  </p>
                  <small v-if="role.version">{{ t('workspace.version') }} {{ role.version }}</small>
                  <h3>{{ t('workspace.recent') }}</h3>
                  <ErrorNotice v-if="recentErrors[role.id]" :error="recentErrors[role.id]" :summary="t('workspace.recentFailed')">
                    <button type="button" @click="refreshLibrary">
                      {{ t('workspace.retry') }}
                    </button>
                  </ErrorNotice>
                  <p v-else-if="!recent[role.id]?.length" class="muted">
                    {{ t('workspace.noRecent') }}
                  </p>
                  <button v-for="session in (recent[role.id] ?? []).slice(0, 2)" :key="session.session_id" class="card-conversation" type="button" :disabled="blocked" @click="enterRole(role.id, session)">
                    {{ sessionLabel(session.last_message_snippet) }}<span aria-hidden="true">↗</span>
                  </button>
                  <button class="primary enter-role" type="button" :disabled="blocked" @click="enterRole(role.id)">
                    {{ t('workspace.enter') }} <span aria-hidden="true">→</span>
                  </button>
                </div>
              </article>
            </div>
          </div>

          <div v-else-if="runtimeReady" class="runtime-grid" :class="{ 'runtime-grid--sidebar-right': sidebarRight }" :aria-busy="blocked">
            <aside class="runtime-role" :class="{ 'runtime-role--open': rolePanelOpen }">
              <CharacterInfo :role-id="roleStore.currentRoleId" :name="roleName" :emotion="emotion" :portrait-asset-rel-path="portraitAssetRelPath" layout="sidebar" />
              <details class="identity-section" open>
                <summary>{{ t('workspace.identity') }}</summary>
                <RoleIdentityControls variant="compact" />
                <label v-if="relationOptions.length">{{ t('workspace.relationship') }}
                  <select :value="roleStore.relationSelectValue" :disabled="blocked || chatListLoading" @change="onChangeRelation(($event.target as HTMLSelectElement).value)">
                    <option v-for="relation in relationOptions" :key="relation.id" :value="relation.id">{{ relation.name }}</option>
                  </select>
                </label>
              </details>
              <details class="pack-section">
                <summary>{{ t('workspace.rolePack') }}</summary><RolePackBar :disabled="blocked" @imported="onImported" @notify="notify" />
              </details>
            </aside>
            <section class="runtime-chat" :class="{ 'runtime-chat--input-top': chatInputTop }">
              <div class="conversation-header">
                <button class="compact-role-controls" type="button" :aria-expanded="rolePanelOpen" @click="rolePanelOpen = !rolePanelOpen">
                  {{ t('workspace.identity') }}
                </button>
                <div><h2>{{ sessionLabel(activeSession?.last_message_snippet ?? '') }}</h2><small>{{ t('workspace.memoryContinues') }}</small></div>
                <button type="button" :disabled="blocked" @click="enterRole(roleStore.currentRoleId, undefined, true)">
                  {{ t('workspace.newChat') }}
                </button>
                <button type="button" @click="openRecords">
                  {{ t('workspace.records') }}
                </button>
              </div>
              <div class="workspace-messages">
                <ChatMessageList ref="chatListRef" :key="activeChatKey" :messages="messages" :history-split-index="0" :loading="chatListLoading" :role-switching="blocked" />
              </div>
              <div class="workspace-composer">
                <button v-if="chatStore.isLoading" class="stop-button" type="button" @click="chatStore.cancelPendingSend()">
                  ■ {{ t('workspace.stop') }}
                </button>
                <ChatInput ref="chatInputRef" :key="activeChatKey" v-model="draft" :disabled="blocked" :loading="chatListLoading" :role-name="roleName" @send="submitMessage" @adult-action="onAdultAction" />
              </div>
            </section>
          </div>
        </RenderErrorBoundary>
      </section>
    </div>

    <ModelManagerPanel v-if="modelManagerOpen" :visible="modelManagerOpen" :initial-provider="initialProvider" @saved="onModelSaved" @close="closeModelManager" @open-settings="() => { closeModelManager(); openSettingsView() }" />
    <SettingsView v-if="settingsViewOpen" basic-workspace :visible="settingsViewOpen" :focus-tab="settingsFocusTab" :storage-role-id="roleStore.currentRoleId" @close="settingsViewOpen = false" />
    <ShortcutHelp v-model="shortcutHelpOpen" :bootstrap-epoch="pluginStore.bootstrapEpoch" />
    <Teleport to="body">
      <div v-if="helpOpen" class="workspace-modal-backdrop" @click.self="helpOpen = false">
        <section ref="helpCard" class="workspace-help" role="dialog" aria-modal="true" aria-labelledby="workspace-help-title" tabindex="-1" @keydown.esc="helpOpen = false">
          <header class="workspace-help-header">
            <MumuFeatureIcon name="help" size="heading" />
            <div class="workspace-help-title-copy">
              <h2 id="workspace-help-title">
                {{ t('workspace.helpTitle') }}
              </h2>
              <p>{{ t('workspace.helpHint') }}</p>
            </div>
            <UiCloseButton class="workspace-help-close" :label="t('workspace.close')" @click="helpOpen = false" />
          </header>
          <SupportDiagnostics :active="helpOpen" />
          <footer class="workspace-help-footer">
            <small>{{ t('workspace.local') }}</small>
            <button type="button" @click="openShortcutHelp">
              {{ t('workspace.shortcuts') }}
            </button>
          </footer>
        </section>
      </div>
    </Teleport>
    <Toast :show="toast.show" :type="toast.type" :message="toast.message" :details="toast.details" recovery-action @open-recovery="openRecoveryFromNotice" />
  </main>
</template>

<style scoped src="./role-workspace.css"></style>

<style>
@import '@oclive/shared/styles/win98/shell-role-workspace.css';
</style>
