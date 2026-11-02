<script setup lang="ts">
import type {
  EnvironmentDiagnostics,
  InstallationRepairIssue,
  InstallationRepairReport,
  RepairIssueScope,
} from '@oclive/shared/api'
import type { LocalePreference } from '@oclive/shared/i18n'
import {
  getRemoteFallbackAppSettings,
  openPathInFileManager,
  runEnvironmentDiagnostics,
  runEnvironmentRepair,
  setRemoteFallbackToBuiltin,
} from '@oclive/shared/api'
import { ApiInvokeError, toErrorPresentation } from '@oclive/shared/api/helpers'
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { useDistroUxProfile } from '@oclive/shared/composables/useDistroUxProfile'
import { useEasterEggSkin } from '@oclive/shared/composables/useEasterEggSkin'
import { useInteractionModeSettings } from '@oclive/shared/composables/useInteractionModeSettings'
import { getLayoutWidths, resetLayoutWidths } from '@oclive/shared/composables/useLayoutWidths'
import { useOcliveAppearance } from '@oclive/shared/composables/useOcliveAppearance'
import { useUserIdentityState } from '@oclive/shared/composables/useUserIdentityState'
import { getLocalePreference, setLocalePreference } from '@oclive/shared/i18n'
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { isChatStreamEnabled, setChatStreamEnabled } from '@oclive/shared/utils/chatStreamSettings'
import { isSentryOptOut, setSentryOptOut } from '@oclive/shared/utils/telemetrySentry'
import { isUnsafeInlinePluginVueEnabled } from '@oclive/shared/utils/vueComponentSecurity'
import { computed, defineAsyncComponent, inject, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ErrorNotice from '../ErrorNotice.vue'
import KeybindingsSettingsSection from '../hotkey/KeybindingsSettingsSection.vue'
import PluginSlotEmbed from '../PluginSlotEmbed.vue'
import RolePackBar from '../RolePackBar.vue'
import HelpHint from '../shared/HelpHint.vue'
import UiButton from '../ui/UiButton.vue'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiSection from '../ui/UiSection.vue'
import UiSelect from '../ui/UiSelect.vue'
import UiToggle from '../ui/UiToggle.vue'

const props = defineProps<{
  visible: boolean
  embedded: boolean
  generalSubTab: GeneralSubTab
  basicWorkspace?: boolean
  page?: string
}>()
const ReplyPostProcessorStatus = defineAsyncComponent(() => import('../role/ReplyPostProcessorStatus.vue'))
const RoleIdentityControls = defineAsyncComponent(() => import('../role/RoleIdentityControls.vue'))
const KernelConnectionSettingsPanel = defineAsyncComponent(() => import('./KernelConnectionSettingsPanel.vue'))

type GeneralSubTab = 'simple' | 'advanced'

const { t } = useI18n()
const pluginStore = usePluginStore()
const roleStore = useRoleStore()
const uiStore = useUiStore()
const shell = inject(MAIN_SHELL_KEY, null)
const { showToast } = useAppToast()
const { allowModeSwitch, ensureDistroUxProfileLoaded } = useDistroUxProfile()
const { onInteractionModeSelect } = useInteractionModeSettings()
const { hasCatalog } = useUserIdentityState()
const { themeCycleLabel, cycleTheme, bumpScale, scaleLabel } = useOcliveAppearance()
const { skinUnlocked, win98Enabled, toggleWin98 } = useEasterEggSkin()
const inlinePluginVueLocked = !isUnsafeInlinePluginVueEnabled()
const activePage = computed(() => props.page ?? (props.generalSubTab === 'advanced' ? 'advanced' : 'appearance'))
const settingsFailure = ref<{ error: unknown, summary: string, retry: () => Promise<void> } | null>(null)
const settingsPageRef = ref<HTMLElement | null>(null)
const remoteFallbackSaving = ref(false)
const remoteFallbackReady = ref(false)
const iframeSaving = ref(false)

function reportFailure(error: unknown, summary: string, retry: () => Promise<void>, page = 'connection') {
  if (!(error instanceof ApiInvokeError)) {
    const presentation = toErrorPresentation(error)
    recordDiagnostic({ level: 'error', source: 'settings', summary: t(summary), details: presentation.details || presentation.summary })
  }
  if (!props.visible || activePage.value !== page)
    return
  settingsFailure.value = { error, summary, retry }
}

function focusSettingsPage(): void {
  // The notice's controls disappear on retry/dismiss. Keep keyboard focus in
  // this page before removing them, without moving it after an async result.
  if (settingsPageRef.value?.contains(document.activeElement))
    settingsPageRef.value.focus({ preventScroll: true })
}

function retrySettingsFailure(): void {
  const retry = settingsFailure.value?.retry
  if (!retry)
    return
  focusSettingsPage()
  void retry()
}

function dismissSettingsFailure(): void {
  focusSettingsPage()
  settingsFailure.value = null
}
watch(activePage, () => {
  settingsFailure.value = null
})

function onWin98SkinChange(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (checked !== win98Enabled.value)
    toggleWin98()
}
const localePreference = ref<LocalePreference>(getLocalePreference())

onMounted(() => {
  void ensureDistroUxProfileLoaded()
})

async function onSettingsInteractionModeChange(ev: Event): Promise<void> {
  if (!uiStore.advancedFeaturesEnabled)
    return
  // Primary user entry alongside in-shell InteractionModeBar (FluentShell).
  await onInteractionModeSelect(ev)
}

const hasSentryDsn
  = typeof import.meta.env.VITE_SENTRY_DSN === 'string' && import.meta.env.VITE_SENTRY_DSN.length > 0
const sentryOptOut = ref(isSentryOptOut())
const chatStreamEnabled = ref(isChatStreamEnabled())

function onChatStreamEnabledChange(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked
  void saveChatStreamEnabled(enabled)
}

async function saveChatStreamEnabled(enabled: boolean) {
  settingsFailure.value = null
  const previous = chatStreamEnabled.value
  chatStreamEnabled.value = enabled
  try {
    setChatStreamEnabled(enabled)
  }
  catch (error) {
    chatStreamEnabled.value = previous
    reportFailure(error, 'settings.chatSaveFailed', () => saveChatStreamEnabled(enabled), 'chat')
  }
}

async function onSentryOptOutChange(e: Event) {
  const optOut = (e.target as HTMLInputElement).checked
  setSentryOptOut(optOut)
  sentryOptOut.value = optOut
  if (optOut) {
    try {
      const Sentry = await import('@sentry/vue')
      await Sentry.close(2000)
    }
    catch {
      // Sentry might not be loaded (no DSN / opt-out since startup); ignore.
    }
    showToast('info', t('settings.sentryDisabledToast'))
  }
  else {
    showToast('info', t('settings.sentryReenableRestartToast'))
  }
}

const envDiagLoading = ref(false)
const envDiag = ref<EnvironmentDiagnostics | null>(null)
const envRepairLoading = ref(false)
const envRepair = ref<InstallationRepairReport | null>(null)
const groupedRepairIssues = computed(() => {
  const groups = new Map<RepairIssueScope, InstallationRepairIssue[]>()
  for (const issue of envRepair.value?.issues ?? []) {
    const entries = groups.get(issue.scope) ?? []
    entries.push(issue)
    groups.set(issue.scope, entries)
  }
  return [...groups].map(([scope, issues]) => ({ scope, issues }))
})

const remoteFallbackLoading = ref(false)
const remoteFallbackChecked = ref(true)
const remoteFallbackEnvLocked = ref(false)

const layoutLeftRailW = ref(getLayoutWidths().leftRail)
const layoutSidePanelW = ref(getLayoutWidths().sidePanel)

function onResetPanelWidths() {
  const widths = resetLayoutWidths()
  layoutLeftRailW.value = widths.leftRail
  layoutSidePanelW.value = widths.sidePanel
  showToast('success', t('settings.layoutResetWidthsDone'))
}

function onLocalePreferenceChange(ev: Event): void {
  const v = (ev.target as HTMLSelectElement).value as LocalePreference
  setLocalePreference(v)
  localePreference.value = v
}

async function loadRemoteFallbackSettings() {
  if (remoteFallbackLoading.value || remoteFallbackSaving.value)
    return
  settingsFailure.value = null
  remoteFallbackLoading.value = true
  remoteFallbackReady.value = false
  try {
    const s = await getRemoteFallbackAppSettings()
    remoteFallbackEnvLocked.value = s.remoteFallbackEnvOverrideActive
    remoteFallbackChecked.value = s.remoteFallbackToBuiltin.trim() !== '0'
    remoteFallbackReady.value = true
  }
  catch (err) {
    reportFailure(err, 'settings.fallbackLoadFailed', loadRemoteFallbackSettings)
  }
  finally {
    remoteFallbackLoading.value = false
  }
}

watch(
  () => props.visible && activePage.value === 'connection',
  (v) => {
    if (v)
      void loadRemoteFallbackSettings()
  },
  { immediate: true },
)

async function onRemoteFallbackToggle(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await saveRemoteFallback(checked)
}

async function saveRemoteFallback(checked: boolean) {
  if (remoteFallbackEnvLocked.value || !remoteFallbackReady.value || remoteFallbackSaving.value)
    return
  settingsFailure.value = null
  remoteFallbackSaving.value = true
  const prev = remoteFallbackChecked.value
  remoteFallbackChecked.value = checked
  try {
    await setRemoteFallbackToBuiltin(checked)
    showToast('info', t('settings.remoteFallbackSavedToast'))
  }
  catch (err) {
    remoteFallbackChecked.value = prev
    reportFailure(err, 'settings.fallbackSaveFailed', () => saveRemoteFallback(checked))
  }
  finally {
    remoteFallbackSaving.value = false
  }
}

async function onRunEnvironmentDiagnostics() {
  if (envDiagLoading.value || envRepairLoading.value)
    return
  settingsFailure.value = null
  envDiagLoading.value = true
  envDiag.value = null
  try {
    envDiag.value = await runEnvironmentDiagnostics()
    showToast('info', t('settings.envCheckDoneToast'))
  }
  catch (err) {
    reportFailure(err, 'settings.checkFailed', onRunEnvironmentDiagnostics)
  }
  finally {
    envDiagLoading.value = false
  }
}

async function onRunEnvironmentRepair() {
  if (envDiagLoading.value || envRepairLoading.value)
    return
  settingsFailure.value = null
  envRepairLoading.value = true
  envRepair.value = null
  try {
    envRepair.value = await runEnvironmentRepair()
    showToast(
      envRepair.value.success ? 'success' : 'error',
      envRepair.value.success
        ? t('settings.envRepairDoneToast')
        : t('settings.envRepairIssuesToast', { count: envRepair.value.issues.length }),
    )
  }
  catch (err) {
    reportFailure(err, 'settings.repairFailed', onRunEnvironmentRepair)
  }
  finally {
    envRepairLoading.value = false
  }
}

async function onOpenRepairReport() {
  const path = envRepair.value?.reportPath.trim()
  if (!path)
    return
  try {
    await openPathInFileManager(path)
  }
  catch (err) {
    reportFailure(err, 'settings.openReportFailed', onOpenRepairReport)
  }
}

async function onToggleForceIframe(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await saveForceIframe(checked)
}

async function saveForceIframe(checked: boolean) {
  if (inlinePluginVueLocked || iframeSaving.value)
    return
  settingsFailure.value = null
  iframeSaving.value = true
  const previous = pluginStore.pluginState.force_iframe_mode
  pluginStore.pluginState = {
    ...pluginStore.pluginState,
    force_iframe_mode: checked,
  }
  try {
    await pluginStore.persist()
    showToast('info', t('settings.iframeSavedInfo'))
  }
  catch (err) {
    reportFailure(err, 'settings.iframeSaveFailed', () => saveForceIframe(checked), 'advanced')
    pluginStore.pluginState = {
      ...pluginStore.pluginState,
      force_iframe_mode: previous,
    }
  }
  finally {
    iframeSaving.value = false
  }
}
</script>

<template>
  <form ref="settingsPageRef" class="sv-body sv-general-page" :class="{ 'sv-body--embedded': embedded }" tabindex="-1" @submit.prevent>
    <ErrorNotice v-if="settingsFailure" :error="settingsFailure.error" :summary="t(settingsFailure.summary)" dismissible @dismiss="dismissSettingsFailure">
      <p class="sv-failure-reason">
        {{ toErrorPresentation(settingsFailure.error).summary }}
      </p>
      <p class="sv-muted">
        {{ t('settings.actionFailedHelp') }}
      </p>
      <UiButton size="sm" variant="secondary" :disabled="envDiagLoading || envRepairLoading || remoteFallbackLoading || remoteFallbackSaving || iframeSaving" @click="retrySettingsFailure">
        {{ t('settings.retryAction') }}
      </UiButton>
    </ErrorNotice>

    <UiSection v-if="activePage === 'roles'" :title="t('settings.rolePacksTitle')" :description="t('settings.rolePacksHelp')">
      <RolePackBar @notify="(payload) => showToast(payload.type, payload.message, payload.details)" @imported="(roleId) => shell?.onRolePackImported?.(roleId)" />
    </UiSection>
    <UiSection v-if="activePage === 'roles' && hasCatalog" :title="t('settings.userIdentitySectionTitle')">
      <RoleIdentityControls variant="full" settings-layout />
    </UiSection>

    <UiSection v-if="activePage === 'appearance'" :title="t('settings.appearanceSectionTitle')" :description="t('settings.appearanceSectionHelp')">
      <UiFieldRow :label="t('app.locale.label')">
        <UiSelect :model-value="localePreference" :aria-label="t('app.locale.label')" @change="onLocalePreferenceChange">
          <option value="system">
            {{ t('app.locale.system') }}
          </option><option value="zh-CN">
            {{ t('app.locale.zhCN') }}
          </option><option value="en-US">
            {{ t('app.locale.enUS') }}
          </option>
        </UiSelect>
      </UiFieldRow>
      <UiFieldRow :label="t('settings.themeLabel')">
        <UiButton size="sm" variant="secondary" @click="cycleTheme">
          {{ themeCycleLabel }}
        </UiButton>
      </UiFieldRow>
      <UiFieldRow :label="t('settings.scaleLabel')">
        <div class="sv-scale-controls">
          <UiButton size="sm" variant="secondary" :aria-label="t('settings.smaller')" @click="bumpScale(-1)">
            A−
          </UiButton><output>{{ scaleLabel }}</output><UiButton size="sm" variant="secondary" :aria-label="t('settings.larger')" @click="bumpScale(1)">
            A+
          </UiButton>
        </div>
      </UiFieldRow>
      <UiToggle v-if="skinUnlocked" :model-value="win98Enabled" :label="t('settings.skinWin98Label')" :description="t('settings.skinWin98Help')" @change="onWin98SkinChange" />
    </UiSection>
    <UiSection v-if="activePage === 'appearance' && !basicWorkspace" :title="t('settings.layoutSectionTitle')" :description="t('settings.layoutSectionLead')">
      <template #extra>
        <HelpHint :text="t('settings.layoutSectionHelp')" />
      </template>
      <p class="sv-muted sv-layout-readout">
        {{ t('settings.layoutCurrentWidths', { left: layoutLeftRailW, side: layoutSidePanelW }) }}
      </p>
      <UiButton size="sm" variant="secondary" @click="onResetPanelWidths">
        {{ t('settings.layoutResetWidths') }}
      </UiButton>
    </UiSection>

    <UiSection v-if="activePage === 'chat'" :title="t('settings.chatStreamSectionTitle')" :description="t('settings.chatStreamSectionHelp')">
      <UiToggle :model-value="chatStreamEnabled" :label="t('settings.chatStreamEnabledLabel')" :description="t('settings.chatStreamEnabledHint')" @change="onChatStreamEnabledChange" />
    </UiSection>
    <UiSection v-if="activePage === 'chat'" :title="t('settings.postProcessorSectionTitle')">
      <ReplyPostProcessorStatus :show-title="false" />
    </UiSection>

    <UiSection v-if="activePage === 'connection'" :title="t('kernel.diagnostics.title')">
      <KernelConnectionSettingsPanel :active="activePage === 'connection' && visible" />
    </UiSection>
    <UiSection
      v-if="activePage === 'connection'"
      :title="t('settings.envCheckTitle')"
      :description="t('settings.envCheckLead')"
    >
      <template #extra>
        <HelpHint :text="t('settings.envCheckHelp')" />
      </template>
      <p class="sv-muted sv-small">
        {{ t("settings.envCheckOllamaPullNote") }}
      </p>
      <div class="ui-btn-group sv-env-actions">
        <UiButton
          size="sm"
          variant="secondary"
          :disabled="envDiagLoading || envRepairLoading"
          @click="onRunEnvironmentDiagnostics"
        >
          {{ envDiagLoading ? t("settings.envCheckRunning") : t("settings.envCheckRun") }}
        </UiButton>
        <UiButton
          size="sm"
          variant="primary"
          :disabled="envDiagLoading || envRepairLoading"
          @click="onRunEnvironmentRepair"
        >
          {{ envRepairLoading ? t("settings.envRepairRunning") : t("settings.envRepairRun") }}
        </UiButton>
      </div>
      <div v-if="envDiag" class="sv-env-results" role="status">
        <p class="sv-env-line">
          <strong>{{ t("settings.envCheckOllama", { url: envDiag.ollamaBaseUrl }) }}</strong>
          —
          <span :class="envDiag.ollamaReachable ? 'sv-ok' : 'sv-bad'">
            {{
              envDiag.ollamaReachable
                ? t("settings.envCheckOllamaOk")
                : t("settings.envCheckOllamaFail")
            }}
          </span>
          <span v-if="envDiag.ollamaDetail" class="sv-muted sv-detail">
            {{ t("settings.envCheckDetail") }} {{ envDiag.ollamaDetail }}
          </span>
        </p>
        <p class="sv-env-line">
          <strong>{{ t("settings.envCheckRoles") }}</strong>
          —
          <span
            :class="
              envDiag.rolesDirExists && envDiag.rolesDirReadable ? 'sv-ok' : 'sv-bad'
            "
          >
            {{
              !envDiag.rolesDirExists
                ? t("settings.envCheckRolesMissing")
                : !envDiag.rolesDirReadable
                  ? t("settings.envCheckRolesUnreadable")
                  : t("settings.envCheckRolesOk")
            }}
          </span>
          <code class="sv-code">{{ envDiag.rolesDir }}</code>
        </p>
        <p class="sv-muted sv-small">
          {{ t("settings.envCheckRolesHint") }}
        </p>
        <p class="sv-env-line">
          <strong>{{ t("settings.envCheckAppData") }}</strong>
          —
          <span :class="envDiag.appDataWritable ? 'sv-ok' : 'sv-bad'">
            {{
              envDiag.appDataWritable
                ? t("settings.envCheckAppDataOk")
                : t("settings.envCheckAppDataFail")
            }}
          </span>
          <span v-if="envDiag.appDataDetail" class="sv-muted sv-detail">
            {{ t("settings.envCheckDetail") }} {{ envDiag.appDataDetail }}
          </span>
          <code class="sv-code">{{ envDiag.appDataDir }}</code>
        </p>
      </div>
      <div v-if="envRepair" class="sv-env-results" role="status">
        <p class="sv-env-line">
          <strong>{{ t("settings.envRepairResult") }}</strong>
          —
          <span :class="envRepair.success ? 'sv-ok' : 'sv-bad'">
            {{
              envRepair.success
                ? t("settings.envRepairSuccess")
                : t("settings.envRepairNeedsAttention")
            }}
          </span>
        </p>
        <p class="sv-muted sv-small">
          {{
            t("settings.envRepairCounts", {
              roles: envRepair.roleCount,
              plugins: envRepair.pluginCount,
            })
          }}
        </p>
        <ul class="sv-repair-list">
          <li v-for="action in envRepair.actions" :key="action.code">
            <code>{{ action.code }}</code>
            <span :class="action.status === 'failed' ? 'sv-bad' : 'sv-ok'">
              {{ t(`settings.envRepairActionStatus.${action.status}`) }}
            </span>
            <span>{{ action.summary }}</span>
            <span v-if="action.detail" class="sv-muted sv-detail">{{ action.detail }}</span>
          </li>
        </ul>
        <div v-if="envRepair.issues.length" class="sv-repair-issues">
          <p class="sv-env-line sv-bad">
            <strong>{{ t("settings.envRepairIssues", { count: envRepair.issues.length }) }}</strong>
          </p>
          <div
            v-for="group in groupedRepairIssues"
            :key="group.scope"
            class="sv-repair-group"
          >
            <p class="sv-env-line">
              <strong>{{ t(`settings.envRepairScope.${group.scope}`) }}</strong>
            </p>
            <ul class="sv-repair-list">
              <li v-for="issue in group.issues" :key="`${issue.code}:${issue.path}`">
                <code>{{ issue.code }}</code>
                <span :class="issue.severity === 'error' ? 'sv-bad' : 'sv-warn'">
                  {{ t(`settings.envRepairSeverity.${issue.severity}`) }}
                </span>
                <span class="sv-muted">
                  {{ t(`settings.envRepairCategory.${issue.category}`) }}
                </span>
                <span>{{ issue.summary }}</span>
                <span class="sv-muted">
                  {{
                    issue.repairable
                      ? t("settings.envRepairCanRetry")
                      : t("settings.envRepairManual")
                  }}
                </span>
                <span v-if="issue.detail" class="sv-muted sv-detail">{{ issue.detail }}</span>
                <code v-if="issue.path" class="sv-code">{{ issue.path }}</code>
              </li>
            </ul>
          </div>
        </div>
        <div v-if="envRepair.reportPath" class="sv-report-row">
          <code class="sv-code">{{ envRepair.reportPath }}</code>
          <UiButton size="sm" variant="secondary" @click="onOpenRepairReport">
            {{ t("settings.envRepairOpenReport") }}
          </UiButton>
        </div>
      </div>
    </UiSection>
    <UiSection v-if="activePage === 'connection'" :title="t('settings.remoteFallbackSectionTitle')">
      <UiToggle :model-value="remoteFallbackChecked" :label="t('settings.remoteFallbackLabel')" :description="t('settings.remoteFallbackHelp')" :disabled="remoteFallbackLoading || !remoteFallbackReady || remoteFallbackEnvLocked" :busy="remoteFallbackSaving" @change="onRemoteFallbackToggle" />
      <p v-if="remoteFallbackEnvLocked" class="sv-muted">
        {{ t('settings.remoteFallbackEnvLocked') }}
      </p>
    </UiSection>

    <UiSection v-if="activePage === 'advanced' && !basicWorkspace" :title="t('settings.advancedFeaturesTitle')" :description="t('settings.advancedFeaturesHelp')">
      <UiToggle :model-value="uiStore.advancedFeaturesEnabled" :label="t('settings.advancedFeaturesLabel')" @update:model-value="uiStore.setAdvancedFeaturesEnabled" />
    </UiSection>
    <UiSection v-if="activePage === 'advanced' && !basicWorkspace && uiStore.advancedFeaturesEnabled" :title="t('settings.interactionModeSectionTitle')" :description="t('settings.interactionModeSectionLead')">
      <template #extra>
        <HelpHint :text="t('settings.interactionModePersistNote')" />
      </template>
      <UiFieldRow v-if="allowModeSwitch" :label="t('settings.interactionModeFieldLabel')">
        <UiSelect :model-value="roleStore.roleInfo.interactionMode" :aria-label="t('settings.interactionModeFieldLabel')" @change="onSettingsInteractionModeChange">
          <option value="pure_chat">
            {{ t('app.more.interactionPureChat') }}
          </option><option value="immersive">
            {{ t('app.more.interactionImmersive') }}
          </option>
        </UiSelect>
      </UiFieldRow>
      <p v-else class="sv-muted">
        {{ t('settings.interactionModeLockedNote') }}
      </p>
    </UiSection>
    <KeybindingsSettingsSection v-if="activePage === 'advanced'" />
    <UiSection v-if="activePage === 'advanced' && hasSentryDsn" :title="t('settings.sentrySectionTitle')" :description="t('settings.sentrySectionLead')">
      <UiToggle :model-value="sentryOptOut" :label="t('settings.sentryOptOutLabel')" :description="t('settings.sentryOptOutHelp')" @change="onSentryOptOutChange" />
    </UiSection>
    <UiSection v-if="activePage === 'advanced'" :title="t('settings.securityLabel')">
      <UiToggle :model-value="inlinePluginVueLocked || pluginStore.pluginState.force_iframe_mode === true" :label="t('settings.forceIframeTitle')" :description="t('settings.forceIframeDesc')" :disabled="inlinePluginVueLocked" :busy="iframeSaving" @change="onToggleForceIframe" />
      <p v-if="inlinePluginVueLocked" class="sv-muted">
        {{ t('settings.lockedSwitch') }}
      </p>
    </UiSection>
    <UiSection v-if="activePage === 'advanced' && !basicWorkspace && uiStore.advancedFeaturesEnabled" :title="t('settings.advancedTitle')">
      <p class="sv-muted">
        <span>{{ t('settings.advancedDescPrefix') }}</span><code>{{ t('settings.advancedDescCode') }}</code><span>{{ t('settings.advancedDescSuffix') }}</span>
      </p>
      <PluginSlotEmbed :slot-name="SLOT_SETTINGS_ADVANCED" :aria-label="t('settings.advancedSlotAria')" :bootstrap-epoch="pluginStore.bootstrapEpoch" />
    </UiSection>
  </form>
</template>

<style scoped>
.sv-muted { margin:0; color:var(--text-secondary); font-size:12px; line-height:1.8; }
.sv-layout-readout { font-variant-numeric:tabular-nums; }
.sv-scale-controls { display:flex; align-items:center; gap:10px; }
.sv-scale-controls output { min-width:48px; font-size:13px; text-align:center; font-variant-numeric:tabular-nums; }
.sv-failure-reason { font-size:13px; margin:8px 0; line-height:1.7; }
.sv-general-page>.error-notice :deep(.ui-btn) { margin-top:10px; }
.sv-env-results {
  margin-top: 4px;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sv-env-actions,
.sv-report-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.sv-report-row .sv-code {
  flex: 1 1 18rem;
}

.sv-repair-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding-left: 18px;
  font-size: 11px;
  line-height: 1.45;
}

.sv-repair-list li > :not(:last-child) {
  margin-right: 6px;
}

.sv-repair-issues {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sv-repair-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sv-env-line {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-primary);
}

.sv-small {
  font-size: 11px;
}

.sv-detail {
  display: block;
  margin-top: 2px;
  word-break: break-word;
}

.sv-code {
  display: block;
  margin-top: 4px;
  font-size: 11px;
  padding: 4px 6px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--border-light) 35%, transparent);
  word-break: break-all;
}

.sv-ok {
  color: var(--success, #15803d);
}

.sv-bad {
  color: var(--danger, #b91c1c);
}

.sv-warn {
  color: var(--warning, #a16207);
}
</style>
