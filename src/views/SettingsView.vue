<script setup lang="ts">
import type { EnvironmentDiagnostics, KernelDiagnostics } from '../api'
import type { LocalePreference } from '../i18n'
import { nextTick, ref, Teleport, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {

  getKernelDiagnostics,
  getRemoteFallbackAppSettings,
  runEnvironmentDiagnostics,
  setRemoteFallbackToBuiltin,
  setRoleInteractionMode,
} from '../api'
import HotkeySettingsSection from '../components/hotkey/HotkeySettingsSection.vue'
import PluginSettingsPanelSlots from '../components/PluginSettingsPanelSlots.vue'
import PluginSlotEmbed from '../components/PluginSlotEmbed.vue'
import ReplyPostProcessorStatus from '../components/role/ReplyPostProcessorStatus.vue'
import RoleIdentityControls from '../components/role/RoleIdentityControls.vue'
import ChatStorageSettingsPanel from '../components/settings/ChatStorageSettingsPanel.vue'
import HelpHint from '../components/shared/HelpHint.vue'
import UiButton from '../components/ui/UiButton.vue'
import UiFieldRow from '../components/ui/UiFieldRow.vue'
import UiSection from '../components/ui/UiSection.vue'
import UiSelect from '../components/ui/UiSelect.vue'
import { useAppToast } from '../composables/useAppToast'
import { getLayoutWidths, resetLayoutWidths } from '../composables/useLayoutWidths'
import { useOcliveAppearance } from '../composables/useOcliveAppearance'
import { getLocalePreference, setLocalePreference } from '../i18n'
import { useKernelConnectionStore } from '../stores/kernelConnectionStore'
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from '../stores/pluginStore'
import { useRoleStore } from '../stores/roleStore'
import { isSentryOptOut, setSentryOptOut } from '../utils/telemetrySentry'

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
    /** When set, switches the active tab (e.g. open from StatusBar identity link). */
    focusTab?: SettingsTab | null
  }>(),
  { embedded: false, focusTab: null },
)

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()
const roleStore = useRoleStore()
const kernelConnectionStore = useKernelConnectionStore()
const { showToast } = useAppToast()
const { themeCycleLabel, cycleTheme, bumpScale, scaleLabel } = useOcliveAppearance()
const localePreference = ref<LocalePreference>(getLocalePreference())

const hasSentryDsn
  = typeof import.meta.env.VITE_SENTRY_DSN === 'string' && import.meta.env.VITE_SENTRY_DSN.length > 0
const sentryOptOut = ref(isSentryOptOut())

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

type SettingsTab = 'general' | 'plugins' | 'storage'

const tab = ref<SettingsTab>('general')

watch(
  () => props.focusTab,
  (next) => {
    if (next)
      tab.value = next
  },
)

watch(
  () => props.visible,
  (open) => {
    if (open && props.focusTab)
      tab.value = props.focusTab
  },
)

const envDiagLoading = ref(false)
const envDiag = ref<EnvironmentDiagnostics | null>(null)

const kernelDiagLoading = ref(false)
const kernelDiag = ref<KernelDiagnostics | null>(null)

const remoteFallbackLoading = ref(false)
const remoteFallbackChecked = ref(true)
const remoteFallbackEnvLocked = ref(false)

const settingsDialogRef = ref<HTMLElement | null>(null)

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

async function onInteractionModeChange(ev: Event): Promise<void> {
  const v = (ev.target as HTMLSelectElement).value as 'immersive' | 'pure_chat'
  try {
    const info = await setRoleInteractionMode(roleStore.currentRoleId, v)
    roleStore.applyRoleInfo(info)
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

async function loadRemoteFallbackSettings() {
  remoteFallbackLoading.value = true
  try {
    const s = await getRemoteFallbackAppSettings()
    remoteFallbackEnvLocked.value = s.remoteFallbackEnvOverrideActive
    remoteFallbackChecked.value = s.remoteFallbackToBuiltin.trim() !== '0'
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    remoteFallbackLoading.value = false
  }
}

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void loadRemoteFallbackSettings()
      void nextTick(() => {
        settingsDialogRef.value?.focus({ preventScroll: true })
      })
    }
  },
)

async function onRemoteFallbackToggle(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (remoteFallbackEnvLocked.value) {
    return
  }
  const prev = remoteFallbackChecked.value
  remoteFallbackChecked.value = checked
  try {
    await setRemoteFallbackToBuiltin(checked)
    showToast('info', t('settings.remoteFallbackSavedToast'))
  }
  catch (err) {
    remoteFallbackChecked.value = prev
    showToast('error', err instanceof Error ? err.message : String(err))
  }
}

async function onRunKernelDiagnostics() {
  kernelDiagLoading.value = true
  kernelDiag.value = null
  try {
    kernelDiag.value = await getKernelDiagnostics()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    kernelDiagLoading.value = false
  }
}

async function onReconnectKernelFromSettings() {
  kernelDiagLoading.value = true
  try {
    const status = await kernelConnectionStore.reconnect()
    if (status) {
      kernelDiag.value = kernelDiag.value
        ? { ...kernelDiag.value, status }
        : await getKernelDiagnostics()
    }
    showToast('info', t('kernel.status.reconnect'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    kernelDiagLoading.value = false
  }
}

async function onRunEnvironmentDiagnostics() {
  envDiagLoading.value = true
  envDiag.value = null
  try {
    envDiag.value = await runEnvironmentDiagnostics()
    showToast('info', t('settings.envCheckDoneToast'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    envDiagLoading.value = false
  }
}

async function onToggleForceIframe(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  pluginStore.pluginState = {
    ...pluginStore.pluginState,
    force_iframe_mode: checked,
  }
  try {
    await pluginStore.persist()
    showToast('info', t('settings.iframeSavedInfo'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
    pluginStore.pluginState = {
      ...pluginStore.pluginState,
      force_iframe_mode: !checked,
    }
  }
}
</script>

<template>
  <component
    :is="embedded ? 'div' : Teleport"
    v-bind="embedded ? {} : { to: 'body' }"
  >
    <div
      v-if="visible"
      :class="embedded ? 'sv-embedded-root' : 'sv-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('settings.ariaDialog')"
      @click.self="!embedded && emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="settingsDialogRef"
        :class="embedded ? 'sv-embedded' : 'sv-dialog'"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <header v-if="!embedded" class="sv-head">
          <h2 class="sv-title">
            {{ t("settings.title") }}
          </h2>
          <button type="button" class="sv-close" :aria-label="t('settings.closeAria')" @click="emit('close')">
            ×
          </button>
        </header>

        <nav class="sv-nav" :class="{ 'sv-nav--vertical': embedded }" :aria-label="t('settings.ariaNav')">
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'general' ? 'page' : undefined"
            @click="tab = 'general'"
          >
            {{ t("settings.tabGeneral") }}
          </button>
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'plugins' ? 'page' : undefined"
            @click="tab = 'plugins'"
          >
            {{ t("settings.tabPlugins") }}
          </button>
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'storage' ? 'page' : undefined"
            @click="tab = 'storage'"
          >
            {{ t("settings.tabStorage") }}
          </button>
        </nav>

        <form v-show="tab === 'general'" class="sv-body" @submit.prevent>
          <p class="sv-lead" v-html="t('settings.generalLeadHtml')" />

          <UiSection :title="t('settings.shortcutsLabel')">
            <template #extra>
              <HelpHint :text="t('settings.shortcutsHelp')" />
            </template>
          </UiSection>

          <UiSection
            :title="t('settings.appearanceSectionTitle')"
            :description="t('settings.appearanceSectionHelp')"
          >
            <UiFieldRow :label="t('app.locale.label')">
              <UiSelect
                :model-value="localePreference"
                @change="onLocalePreferenceChange"
              >
                <option value="system">
                  {{ t("app.locale.system") }}
                </option>
                <option value="zh-CN">
                  {{ t("app.locale.zhCN") }}
                </option>
                <option value="en-US">
                  {{ t("app.locale.enUS") }}
                </option>
              </UiSelect>
            </UiFieldRow>
            <UiFieldRow :label="t('app.more.interactionMode')">
              <div class="sv-interaction-mode">
                <UiSelect
                  :model-value="roleStore.roleInfo.interactionMode"
                  @change="onInteractionModeChange"
                >
                  <option value="immersive">
                    {{ t("app.more.interactionImmersive") }}
                  </option>
                  <option value="pure_chat">
                    {{ t("app.more.interactionPureChat") }}
                  </option>
                </UiSelect>
                <HelpHint
                  :paragraphs="[
                    t('app.more.interactionImmersiveHint'),
                    t('app.more.interactionPureChatHint'),
                  ]"
                />
              </div>
            </UiFieldRow>
            <p class="sv-muted">
              {{ t("settings.immersiveOnlyNote") }}
            </p>
            <UiFieldRow :label="t('app.more.ui')">
              <div class="ui-btn-group">
                <UiButton size="sm" variant="secondary" @click="bumpScale(-1)">
                  A−
                </UiButton>
                <span class="ui-btn-group__sep">{{ scaleLabel }}</span>
                <UiButton size="sm" variant="secondary" @click="bumpScale(1)">
                  A+
                </UiButton>
                <UiButton size="sm" variant="secondary" @click="cycleTheme">
                  {{ themeCycleLabel }}
                </UiButton>
              </div>
            </UiFieldRow>
          </UiSection>

          <UiSection
            :title="t('settings.userIdentitySectionTitle')"
            :description="t('settings.userIdentitySectionLead')"
          >
            <template #extra>
              <HelpHint :text="t('settings.userIdentitySectionLeadSecondary')" />
            </template>
            <RoleIdentityControls variant="full" settings-layout />
          </UiSection>

          <UiSection :title="t('settings.postProcessorSectionTitle')">
            <ReplyPostProcessorStatus :show-title="false" />
          </UiSection>

          <UiSection
            :title="t('settings.layoutSectionTitle')"
            :description="t('settings.layoutSectionLead')"
          >
            <template #extra>
              <HelpHint :text="t('settings.layoutSectionHelp')" />
            </template>
            <p class="sv-muted sv-layout-readout">
              {{ t("settings.layoutCurrentWidths", { left: layoutLeftRailW, side: layoutSidePanelW }) }}
            </p>
            <UiButton size="sm" variant="secondary" @click="onResetPanelWidths">
              {{ t("settings.layoutResetWidths") }}
            </UiButton>
          </UiSection>

          <UiSection
            :title="t('settings.envCheckTitle')"
            :description="t('settings.envCheckLead')"
          >
            <template #extra>
              <HelpHint :text="t('settings.envCheckHelp')" />
            </template>
            <p class="sv-muted sv-small">
              {{ t("settings.envCheckOllamaPullNote") }}
            </p>
            <UiButton
              size="sm"
              variant="secondary"
              :disabled="envDiagLoading"
              @click="onRunEnvironmentDiagnostics"
            >
              {{ envDiagLoading ? t("settings.envCheckRunning") : t("settings.envCheckRun") }}
            </UiButton>
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
          </UiSection>

          <UiSection :title="t('kernel.diagnostics.title')">
            <div class="ui-btn-group">
              <UiButton
                size="sm"
                variant="secondary"
                :disabled="kernelDiagLoading"
                @click="onRunKernelDiagnostics"
              >
                {{ kernelDiagLoading ? t("settings.envCheckRunning") : t("kernel.diagnostics.refresh") }}
              </UiButton>
              <UiButton
                size="sm"
                variant="ghost"
                :disabled="kernelDiagLoading"
                @click="onReconnectKernelFromSettings"
              >
                {{ t("kernel.diagnostics.reconnect") }}
              </UiButton>
            </div>
            <div v-if="kernelDiag" class="sv-env-results" role="status">
              <p>
                <strong>{{ t("kernel.diagnostics.mode") }}</strong>
                {{ kernelDiag.status.mode }}
                · :{{ kernelDiag.status.port }}
                ·
                <span :class="kernelDiag.status.healthy ? 'sv-ok' : 'sv-bad'">
                  {{
                    kernelDiag.status.healthy
                      ? t("kernel.diagnostics.healthyYes")
                      : t("kernel.diagnostics.healthyNo")
                  }}
                </span>
              </p>
              <p v-if="kernelDiag.status.binaryPath">
                <strong>{{ t("kernel.diagnostics.binary") }}</strong>
                <code class="sv-code">{{ kernelDiag.status.binaryPath }}</code>
              </p>
              <p v-if="kernelDiag.status.kernelTier">
                <strong>{{ t("kernel.diagnostics.tier") }}</strong> {{ kernelDiag.status.kernelTier }}
              </p>
              <p>
                <strong>{{ t("kernel.diagnostics.sharedRuntime") }}</strong>
                <code class="sv-code">{{ kernelDiag.sharedRuntimePath }}</code>
                <span v-if="kernelDiag.sharedRuntimeModifiedMs" class="sv-muted">
                  ({{ t("kernel.diagnostics.sharedRuntimeMtime") }}:
                  {{ new Date(kernelDiag.sharedRuntimeModifiedMs).toLocaleString() }})
                </span>
              </p>
              <pre v-if="kernelDiag.healthJson" class="sv-code sv-pre">{{
                JSON.stringify(kernelDiag.healthJson, null, 2)
              }}</pre>
            </div>
          </UiSection>
          <details v-if="embedded" class="sv-advanced-fold">
            <summary class="sv-advanced-fold__summary">
              {{ t("settings.advancedFoldTitle") }}
            </summary>
            <div class="sv-advanced-fold__body">
              <UiSection
                v-if="hasSentryDsn"
                :title="t('settings.sentrySectionTitle')"
                :description="t('settings.sentrySectionLead')"
              >
                <template #extra>
                  <HelpHint :text="t('settings.sentryOptOutHelp')" />
                </template>
                <label class="sv-toggle-row">
                  <input type="checkbox" :checked="sentryOptOut" @change="onSentryOptOutChange">
                  <span class="sv-toggle-text">
                    <strong>{{ t("settings.sentryOptOutLabel") }}</strong>
                  </span>
                </label>
              </UiSection>
              <UiSection :title="t('settings.remoteFallbackSectionTitle')">
                <template #extra>
                  <HelpHint :text="t('settings.remoteFallbackHelp')" />
                </template>
                <p v-if="remoteFallbackEnvLocked" class="sv-muted">
                  {{ t("settings.remoteFallbackEnvLocked") }}
                </p>
                <label class="sv-toggle-row">
                  <input
                    type="checkbox"
                    :checked="remoteFallbackChecked"
                    :disabled="remoteFallbackLoading || remoteFallbackEnvLocked"
                    @change="onRemoteFallbackToggle"
                  >
                  <span class="sv-toggle-text">
                    <strong>{{ t("settings.remoteFallbackLabel") }}</strong>
                  </span>
                </label>
              </UiSection>
              <UiSection :title="t('settings.advancedTitle')">
                <p class="sv-muted" v-html="t('settings.advancedDesc')" />
                <PluginSlotEmbed
                  :slot-name="SLOT_SETTINGS_ADVANCED"
                  :aria-label="t('settings.advancedSlotAria')"
                  :bootstrap-epoch="pluginStore.bootstrapEpoch"
                />
              </UiSection>
              <UiSection :title="t('settings.securityLabel')">
                <label class="sv-toggle-row">
                  <input
                    type="checkbox"
                    :checked="pluginStore.pluginState.force_iframe_mode === true"
                    @change="onToggleForceIframe"
                  >
                  <span class="sv-toggle-text">
                    <strong>{{ t("settings.forceIframeTitle") }}</strong>
                    <span class="sv-muted sv-toggle-desc">
                      {{ t("settings.forceIframeDesc") }}
                    </span>
                  </span>
                </label>
              </UiSection>
            </div>
          </details>
          <template v-else>
            <UiSection
              v-if="hasSentryDsn"
              :title="t('settings.sentrySectionTitle')"
              :description="t('settings.sentrySectionLead')"
            >
              <template #extra>
                <HelpHint :text="t('settings.sentryOptOutHelp')" />
              </template>
              <label class="sv-toggle-row">
                <input type="checkbox" :checked="sentryOptOut" @change="onSentryOptOutChange">
                <span class="sv-toggle-text">
                  <strong>{{ t("settings.sentryOptOutLabel") }}</strong>
                </span>
              </label>
            </UiSection>
            <UiSection :title="t('settings.remoteFallbackSectionTitle')">
              <template #extra>
                <HelpHint :text="t('settings.remoteFallbackHelp')" />
              </template>
              <p v-if="remoteFallbackEnvLocked" class="sv-muted">
                {{ t("settings.remoteFallbackEnvLocked") }}
              </p>
              <label class="sv-toggle-row">
                <input
                  type="checkbox"
                  :checked="remoteFallbackChecked"
                  :disabled="remoteFallbackLoading || remoteFallbackEnvLocked"
                  @change="onRemoteFallbackToggle"
                >
                <span class="sv-toggle-text">
                  <strong>{{ t("settings.remoteFallbackLabel") }}</strong>
                </span>
              </label>
            </UiSection>
            <UiSection :title="t('settings.advancedTitle')">
              <p class="sv-muted" v-html="t('settings.advancedDesc')" />
              <PluginSlotEmbed
                :slot-name="SLOT_SETTINGS_ADVANCED"
                :aria-label="t('settings.advancedSlotAria')"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
            </UiSection>
            <UiSection :title="t('settings.securityLabel')">
              <label class="sv-toggle-row">
                <input
                  type="checkbox"
                  :checked="pluginStore.pluginState.force_iframe_mode === true"
                  @change="onToggleForceIframe"
                >
                <span class="sv-toggle-text">
                  <strong>{{ t("settings.forceIframeTitle") }}</strong>
                  <span class="sv-muted sv-toggle-desc">
                    {{ t("settings.forceIframeDesc") }}
                  </span>
                </span>
              </label>
            </UiSection>
          </template>
        </form>

        <form v-show="tab === 'plugins'" class="sv-body" @submit.prevent>
          <UiSection
            :title="t('settings.pluginsPanelTitle')"
            :description="t('settings.pluginsPanelHint1')"
          >
            <template #extra>
              <HelpHint :paragraphs="[t('settings.pluginsPanelHint1'), t('settings.pluginsPanelHint2')]" />
            </template>
            <PluginSettingsPanelSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
          </UiSection>

          <HotkeySettingsSection />
        </form>

        <div v-show="tab === 'storage'" class="sv-body">
          <ChatStorageSettingsPanel />
        </div>
      </div>
    </div>
  </component>
</template>

<style scoped>
.sv-embedded-root {
  flex: 1;
  width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.sv-embedded {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 0;
  padding: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.sv-embedded .sv-body {
  flex: 1;
  min-width: 0;
  overflow: visible;
  padding: var(--tool-space-4, 16px);
  max-width: none;
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.sv-embedded .sv-section {
  gap: var(--tool-space-3, 12px);
  padding-top: var(--tool-space-6, 24px);
  margin-bottom: 0;
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}

.sv-embedded .ui-section {
  margin-bottom: 0;
}

.sv-embedded .sv-body > .sv-lead + .sv-section,
.sv-embedded .sv-body > .sv-section:first-child {
  border-top: none;
  padding-top: 0;
}

.sv-embedded .sv-lead {
  margin-bottom: var(--tool-space-4, 16px);
}

.sv-embedded .sv-label {
  font-size: var(--tool-fs-md, 13px);
}

.sv-embedded .sv-toggle-row {
  min-height: var(--tool-row-h, 32px);
  align-items: center;
}

.sv-advanced-fold {
  border: none;
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  border-radius: 0;
  padding: var(--tool-space-4, 16px) 0 0;
  margin-bottom: 0;
}

.sv-advanced-fold__summary {
  cursor: pointer;
  font-size: var(--tool-fs-md, 13px);
  font-weight: 600;
  color: var(--tool-text, var(--text-primary));
  list-style: none;
}

.sv-advanced-fold__summary::-webkit-details-marker {
  display: none;
}

.sv-advanced-fold__body {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-6, 24px);
  margin-top: var(--tool-space-4, 16px);
}

.sv-layout-readout {
  font-variant-numeric: tabular-nums;
}

.sv-interaction-mode {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--tool-space-2, 8px);
  width: 100%;
}

.sv-interaction-mode .ui-select {
  flex: 1 1 10rem;
  min-width: 0;
}

.sv-nav--vertical {
  position: sticky;
  top: 0;
  align-self: flex-start;
  flex: 0 0 132px;
  flex-shrink: 0;
  flex-direction: column;
  align-items: stretch;
  gap: var(--tool-space-1, 4px);
  padding: var(--tool-space-3, 12px);
  border-bottom: none;
  border-right: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-sidebar, var(--tool-bg, var(--bg-secondary)));
}

.sv-nav--vertical .sv-nav-btn {
  width: 100%;
  text-align: left;
  border-radius: var(--tool-radius, 4px);
  font-size: var(--tool-fs-md, 13px);
}

.sv-nav--vertical .sv-nav-btn[aria-current="page"] {
  border-color: transparent;
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 12%, transparent);
  color: var(--tool-text, var(--text-primary));
}

.sv-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10040;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sv-dialog {
  position: relative;
  width: min(640px, 100%);
  max-height: min(90vh, 800px);
  overflow: auto;
  padding: 16px 18px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.sv-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
}
.sv-title {
  margin: 0;
  font-size: 18px;
}
.sv-close {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sv-nav {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid var(--border-light);
  padding-bottom: 8px;
}
.sv-nav-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-nav-btn[aria-current="page"] {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.sv-body {
  flex: 1;
  min-height: 0;
}
.sv-lead {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sv-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-row-h {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.sv-label {
  font-weight: 600;
  font-size: 14px;
}
.sv-h3 {
  margin: 0;
  font-size: 15px;
}
.sv-muted {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.sv-toggle-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
  font-size: 13px;
  line-height: 1.45;
}
.sv-toggle-row input {
  margin-top: 3px;
  flex-shrink: 0;
}
.sv-toggle-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.sv-toggle-desc {
  display: block;
  font-weight: 400;
}
.sv-v2-launch {
  margin-top: 4px;
}
.sv-v2-launch-btn {
  padding: 8px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.sv-v2-launch-btn:hover {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 45%, var(--border-light));
}
.sv-env-btn {
  align-self: flex-start;
  padding: 8px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.sv-env-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
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
.sv-field-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  font-size: 13px;
}
.sv-field-label {
  min-width: 6rem;
  color: var(--text-secondary);
}
.sv-select {
  min-width: 10rem;
  max-width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-light);
  border-radius: 6px;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 13px;
}
.sv-appearance-row {
  align-items: center;
}
.sv-appearance-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}
.sv-appearance-scale {
  min-width: 2.5rem;
  text-align: center;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
