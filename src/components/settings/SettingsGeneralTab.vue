<script setup lang="ts">
import type { EnvironmentDiagnostics } from '../../api'
import type { LocalePreference } from '../../i18n'
import { defineAsyncComponent, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getRemoteFallbackAppSettings,
  runEnvironmentDiagnostics,
  setRemoteFallbackToBuiltin,
} from '../../api'
import PluginSlotEmbed from '../PluginSlotEmbed.vue'
import HelpHint from '../shared/HelpHint.vue'
import UiButton from '../ui/UiButton.vue'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiSection from '../ui/UiSection.vue'
import UiSelect from '../ui/UiSelect.vue'
import { useAppToast } from '../../composables/useAppToast'
import { getLayoutWidths, resetLayoutWidths } from '../../composables/useLayoutWidths'
import { useOcliveAppearance } from '../../composables/useOcliveAppearance'
import { getLocalePreference, setLocalePreference } from '../../i18n'
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from '../../stores/pluginStore'
import { isSentryOptOut, setSentryOptOut } from '../../utils/telemetrySentry'

const ReplyPostProcessorStatus = defineAsyncComponent(() => import('../role/ReplyPostProcessorStatus.vue'))
const RoleIdentityControls = defineAsyncComponent(() => import('../role/RoleIdentityControls.vue'))
const KernelConnectionSettingsPanel = defineAsyncComponent(() => import('./KernelConnectionSettingsPanel.vue'))

type GeneralSubTab = 'simple' | 'advanced'

const props = defineProps<{
  visible: boolean
  embedded: boolean
  generalSubTab: GeneralSubTab
}>()

const emit = defineEmits<{
  'update:generalSubTab': [value: GeneralSubTab]
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()
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

const envDiagLoading = ref(false)
const envDiag = ref<EnvironmentDiagnostics | null>(null)

const remoteFallbackLoading = ref(false)
const remoteFallbackChecked = ref(true)
const remoteFallbackEnvLocked = ref(false)

const layoutLeftRailW = ref(getLayoutWidths().leftRail)
const layoutSidePanelW = ref(getLayoutWidths().sidePanel)

function setGeneralSubTab(next: GeneralSubTab) {
  emit('update:generalSubTab', next)
}

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
    if (v)
      void loadRemoteFallbackSettings()
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
  <form
    class="sv-body"
    :class="{ 'sv-body--embedded': embedded }"
    @submit.prevent
  >
    <p class="sv-lead" v-html="t('settings.generalLeadHtml')" />

    <nav class="sv-general-subnav" :aria-label="t('onboarding.settings.simpleTab')">
      <button
        type="button"
        class="sv-nav-btn"
        :aria-current="generalSubTab === 'simple' ? 'page' : undefined"
        @click="setGeneralSubTab('simple')"
      >
        {{ t("onboarding.settings.simpleTab") }}
      </button>
      <button
        type="button"
        class="sv-nav-btn"
        :aria-current="generalSubTab === 'advanced' ? 'page' : undefined"
        @click="setGeneralSubTab('advanced')"
      >
        {{ t("onboarding.settings.advancedTab") }}
      </button>
    </nav>

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

    <p v-if="generalSubTab === 'advanced'" class="sv-muted">
      {{ t("onboarding.settings.advancedLead") }}
    </p>

    <UiSection
      v-show="generalSubTab === 'advanced'"
      :title="t('settings.userIdentitySectionTitle')"
      :description="t('settings.userIdentitySectionLead')"
    >
      <template #extra>
        <HelpHint :text="t('settings.userIdentitySectionLeadSecondary')" />
      </template>
      <RoleIdentityControls variant="full" settings-layout />
    </UiSection>

    <UiSection v-show="generalSubTab === 'advanced'" :title="t('settings.postProcessorSectionTitle')">
      <ReplyPostProcessorStatus :show-title="false" />
    </UiSection>

    <UiSection
      v-show="generalSubTab === 'advanced'"
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
      v-show="generalSubTab === 'advanced'"
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

    <UiSection v-show="generalSubTab === 'advanced'" :title="t('kernel.diagnostics.title')">
      <KernelConnectionSettingsPanel :active="generalSubTab === 'advanced' && visible" />
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
        v-if="hasSentryDsn && generalSubTab === 'advanced'"
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
      <UiSection v-show="generalSubTab === 'advanced'" :title="t('settings.remoteFallbackSectionTitle')">
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
      <UiSection v-show="generalSubTab === 'advanced'" :title="t('settings.advancedTitle')">
        <p class="sv-muted" v-html="t('settings.advancedDesc')" />
        <PluginSlotEmbed
          :slot-name="SLOT_SETTINGS_ADVANCED"
          :aria-label="t('settings.advancedSlotAria')"
          :bootstrap-epoch="pluginStore.bootstrapEpoch"
        />
      </UiSection>
      <UiSection v-show="generalSubTab === 'advanced'" :title="t('settings.securityLabel')">
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
</template>

<style scoped>
.sv-body--embedded .sv-lead {
  margin-bottom: var(--tool-space-4, 16px);
}

.sv-body--embedded .sv-toggle-row {
  min-height: var(--tool-row-h, 32px);
  align-items: center;
}

.sv-lead {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}

.sv-general-subnav {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
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
