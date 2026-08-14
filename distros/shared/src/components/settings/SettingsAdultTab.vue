<script setup lang="ts">
import {
  cancelAdultBeatQueuesForRole,
  cancelAllAdultBeatQueues,
  notifyAdultBeatQueueCapacityChanged,
} from '@oclive/shared/lib/adultBeatQueue'
import {
  ADULT_BACKGROUND_QUEUE_CAP_MAX,
  ADULT_BACKGROUND_QUEUE_CAP_MIN,
  ADULT_PACING_INTERVAL_MAX_MS,
  ADULT_PACING_INTERVAL_MIN_MS,
  useAdultInteractionStore,
} from '@oclive/shared/stores/adultInteractionStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import HelpHint from '../shared/HelpHint.vue'
import UiButton from '../ui/UiButton.vue'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiListRow from '../ui/UiListRow.vue'
import UiSection from '../ui/UiSection.vue'

const { t } = useI18n()
const adultStore = useAdultInteractionStore()
const roleStore = useRoleStore()

const confirmOpen = ref(false)
const pendingRoleId = ref<string | null>(null)
const intervalDraft = ref(adultStore.pacingIntervalMs)
const queueCapDraft = ref(adultStore.backgroundQueueCap)
const pacingError = ref('')
const queueError = ref('')

const adultRoles = computed(() => {
  const roles = roleStore.roles.filter(role => role.adultExtensionAvailable)
  if (
    roleStore.currentRoleId
    && roleStore.roleInfo.adultExtensionAvailable
    && !roles.some(role => role.id === roleStore.currentRoleId)
  ) {
    roles.push({
      id: roleStore.currentRoleId,
      name: roleStore.roleInfo.name,
      adultExtensionAvailable: true,
    })
  }
  return roles
})

const enabledRoleCount = computed(() =>
  adultRoles.value.filter(role => adultStore.roleIsEnabled(role.id)).length,
)

const invalidAdultExtensions = computed(() => {
  const errors = roleStore.roles
    .filter(role => role.adultExtensionError)
    .map(role => ({
      id: role.id,
      name: role.name,
      error: role.adultExtensionError,
    }))
  if (
    roleStore.currentRoleId
    && roleStore.roleInfo.adultExtensionError
    && !errors.some(role => role.id === roleStore.currentRoleId)
  ) {
    errors.push({
      id: roleStore.currentRoleId,
      name: roleStore.roleInfo.name,
      error: roleStore.roleInfo.adultExtensionError,
    })
  }
  return errors
})

function roleStatus(roleId: string): string {
  if (!adultStore.roleIsEnabled(roleId))
    return String(t('settings.adult.roleDisabled'))
  return String(t(
    adultStore.globalEnabled
      ? 'settings.adult.roleActive'
      : 'settings.adult.roleSaved',
  ))
}

function syncToggle(event: Event, checked: boolean): void {
  const target = event.target as HTMLInputElement | null
  if (target)
    target.checked = checked
}

function requestEnable(roleId?: string) {
  pendingRoleId.value = roleId ?? null
  if (!adultStore.confirmedAdult) {
    confirmOpen.value = true
    return
  }
  adultStore.setGlobalEnabled(true)
  if (roleId)
    adultStore.setRoleEnabled(roleId, true)
}

function confirmAdult() {
  adultStore.confirmAndEnableGlobal()
  if (pendingRoleId.value)
    adultStore.setRoleEnabled(pendingRoleId.value, true)
  pendingRoleId.value = null
  confirmOpen.value = false
}

function cancelAdult() {
  pendingRoleId.value = null
  confirmOpen.value = false
}

function onGlobalToggle(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  if (enabled) {
    requestEnable()
    syncToggle(event, adultStore.globalEnabled)
    return
  }
  if (!window.confirm(String(t('settings.adult.globalHelp')))) {
    syncToggle(event, adultStore.globalEnabled)
    return
  }
  void cancelAllAdultBeatQueues()
  adultStore.setGlobalEnabled(false)
  syncToggle(event, adultStore.globalEnabled)
}

function onRoleToggle(roleId: string, event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  if (enabled) {
    requestEnable(roleId)
    syncToggle(event, adultStore.roleIsEnabled(roleId))
    return
  }
  void cancelAdultBeatQueuesForRole(roleId)
  adultStore.setRoleEnabled(roleId, false)
  syncToggle(event, adultStore.roleIsEnabled(roleId))
}

function savePacing() {
  const accepted = adultStore.setPacingOverride(
    adultStore.pacingOverrideEnabled,
    intervalDraft.value,
  )
  pacingError.value = accepted
    ? ''
    : String(t('settings.adult.intervalRangeError'))
  intervalDraft.value = adultStore.pacingIntervalMs
}

function onPacingOverrideToggle(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  const accepted = adultStore.setPacingOverride(
    enabled,
    enabled ? intervalDraft.value : adultStore.pacingIntervalMs,
  )
  pacingError.value = accepted
    ? ''
    : String(t('settings.adult.intervalRangeError'))
  syncToggle(event, adultStore.pacingOverrideEnabled)
}

function onQueueToggle(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  let accepted = false
  if (enabled && !adultStore.backgroundQueueWarningAccepted) {
    const warningAccepted = window.confirm(String(t('settings.adult.queueWarning')))
    if (!warningAccepted) {
      syncToggle(event, adultStore.backgroundQueueEnabled)
      return
    }
    accepted = adultStore.setBackgroundQueue(true, queueCapDraft.value, true)
  }
  else {
    accepted = adultStore.setBackgroundQueue(
      enabled,
      enabled ? queueCapDraft.value : adultStore.backgroundQueueCap,
      adultStore.backgroundQueueWarningAccepted,
    )
  }
  queueError.value = accepted
    ? ''
    : String(t('settings.adult.queueCapRangeError'))
  if (!enabled)
    void cancelAllAdultBeatQueues()
  syncToggle(event, adultStore.backgroundQueueEnabled)
  if (accepted)
    notifyAdultBeatQueueCapacityChanged()
}

function saveQueueSettings() {
  const accepted = adultStore.setBackgroundQueue(
    adultStore.backgroundQueueEnabled,
    queueCapDraft.value,
    adultStore.backgroundQueueWarningAccepted,
  )
  queueError.value = accepted
    ? ''
    : String(t('settings.adult.queueCapRangeError'))
  queueCapDraft.value = adultStore.backgroundQueueCap
  if (accepted)
    notifyAdultBeatQueueCapacityChanged()
}

async function resetAdultSettings() {
  if (!window.confirm(String(t('settings.adult.resetConfirm'))))
    return
  await cancelAllAdultBeatQueues()
  adultStore.resetAdultSettings()
  intervalDraft.value = adultStore.pacingIntervalMs
  queueCapDraft.value = adultStore.backgroundQueueCap
  pacingError.value = ''
  queueError.value = ''
  pendingRoleId.value = null
  confirmOpen.value = false
  notifyAdultBeatQueueCapacityChanged()
}
</script>

<template>
  <form class="sv-body adult-settings" @submit.prevent>
    <header class="adult-overview">
      <div class="adult-overview__copy">
        <h3>{{ t("settings.adult.title") }}</h3>
        <p class="sv-lead">
          {{ t("settings.adult.lead") }}
        </p>
      </div>
      <span
        class="adult-status"
        :class="{ 'adult-status--active': adultStore.globalEnabled }"
      >
        <span class="adult-status__dot" aria-hidden="true" />
        {{
          adultStore.globalEnabled
            ? t("settings.adult.enabled")
            : t("settings.adult.disabled")
        }}
      </span>
    </header>

    <div
      v-if="invalidAdultExtensions.length > 0"
      class="adult-extension-errors"
      role="alert"
    >
      <strong>{{ t("settings.adult.extensionErrorTitle") }}</strong>
      <p>{{ t("settings.adult.extensionErrorHelp") }}</p>
      <ul>
        <li v-for="role in invalidAdultExtensions" :key="role.id">
          <strong>{{ role.name }}</strong> ({{ role.id }}): {{ role.error }}
        </li>
      </ul>
    </div>

    <UiSection
      :title="t('settings.adult.accessTitle')"
      :description="t('settings.adult.accessHelp')"
    >
      <template #extra>
        <HelpHint :text="t('settings.adult.globalHelp')" />
      </template>
      <UiListRow
        :label="t('settings.adult.globalLabel')"
        :description="t('settings.adult.globalSummary')"
      >
        <template #control>
          <label class="adult-switch">
            <input
              type="checkbox"
              :checked="adultStore.globalEnabled"
              @change="onGlobalToggle"
            >
            <span class="adult-switch__track" aria-hidden="true">
              <span class="adult-switch__thumb" />
            </span>
            <span class="adult-switch__label">
              {{
                adultStore.globalEnabled
                  ? t("settings.adult.enabled")
                  : t("settings.adult.disabled")
              }}
            </span>
          </label>
        </template>
      </UiListRow>
    </UiSection>

    <UiSection
      :title="t('settings.adult.rolesTitle')"
      :description="t('settings.adult.rolesHelp', {
        enabled: enabledRoleCount,
        total: adultRoles.length,
      })"
    >
      <template #extra>
        <HelpHint :text="t('settings.adult.rolesHint')" />
      </template>
      <div v-if="adultRoles.length === 0" class="adult-empty">
        <span class="adult-empty__icon" aria-hidden="true">◇</span>
        <div>
          <strong>{{ t("settings.adult.rolesEmptyTitle") }}</strong>
          <p>{{ t("settings.adult.rolesEmpty") }}</p>
        </div>
      </div>
      <UiListRow
        v-for="role in adultRoles"
        :key="role.id"
        :label="role.name"
        :description="role.id"
      >
        <template #control>
          <label class="adult-switch">
            <input
              type="checkbox"
              :checked="adultStore.roleIsEnabled(role.id)"
              @change="onRoleToggle(role.id, $event)"
            >
            <span class="adult-switch__track" aria-hidden="true">
              <span class="adult-switch__thumb" />
            </span>
            <span
              class="adult-switch__label"
              :class="{
                'adult-switch__label--active':
                  adultStore.globalEnabled && adultStore.roleIsEnabled(role.id),
              }"
            >
              {{ roleStatus(role.id) }}
            </span>
          </label>
        </template>
      </UiListRow>
    </UiSection>

    <UiSection
      :title="t('settings.adult.pacingTitle')"
      :description="t('settings.adult.pacingHelp')"
    >
      <UiListRow
        :label="t('settings.adult.pacingOverride')"
        :description="t('settings.adult.pacingOverrideHelp')"
      >
        <template #control>
          <label class="adult-switch adult-switch--icon-only">
            <input
              type="checkbox"
              :checked="adultStore.pacingOverrideEnabled"
              @change="onPacingOverrideToggle"
            >
            <span class="adult-switch__track" aria-hidden="true">
              <span class="adult-switch__thumb" />
            </span>
            <span class="adult-switch__label">
              {{
                adultStore.pacingOverrideEnabled
                  ? t("settings.adult.enabled")
                  : t("settings.adult.disabled")
              }}
            </span>
          </label>
        </template>
      </UiListRow>
      <UiFieldRow>
        <template #label>
          <span class="adult-field-label">
            {{ t("settings.adult.intervalLabel") }}
            <HelpHint :text="t('settings.adult.intervalHelp')" pop-align="end" />
          </span>
        </template>
        <div class="adult-number-control">
          <input
            v-model.number="intervalDraft"
            class="ui-input"
            type="number"
            :min="ADULT_PACING_INTERVAL_MIN_MS"
            :max="ADULT_PACING_INTERVAL_MAX_MS"
            step="100"
            inputmode="numeric"
          >
          <span>{{ t("settings.adult.millisecondsUnit") }}</span>
        </div>
      </UiFieldRow>
      <p v-if="pacingError" class="adult-field-error" role="alert">
        {{ pacingError }}
      </p>
      <div class="adult-section-footer">
        <span>{{ t("settings.adult.intervalCurrent", { value: adultStore.pacingIntervalMs }) }}</span>
        <UiButton type="button" size="sm" variant="primary" @click="savePacing">
          {{ t("settings.adult.savePacing") }}
        </UiButton>
      </div>
    </UiSection>

    <UiSection
      :title="t('settings.adult.queueTitle')"
      :description="t('settings.adult.queueSectionHelp')"
    >
      <UiListRow
        :label="t('settings.adult.queueEnable')"
        :description="t('settings.adult.queueHelp')"
      >
        <template #control>
          <label class="adult-switch">
            <input
              type="checkbox"
              :checked="adultStore.backgroundQueueEnabled"
              @change="onQueueToggle"
            >
            <span class="adult-switch__track" aria-hidden="true">
              <span class="adult-switch__thumb" />
            </span>
            <span class="adult-switch__label">
              {{
                adultStore.backgroundQueueEnabled
                  ? t("settings.adult.enabled")
                  : t("settings.adult.disabled")
              }}
            </span>
          </label>
        </template>
      </UiListRow>
      <UiFieldRow>
        <template #label>
          <span class="adult-field-label">
            {{ t("settings.adult.queueCapLabel") }}
            <HelpHint :text="t('settings.adult.queueCapHelp')" pop-align="end" />
          </span>
        </template>
        <div class="adult-number-control">
          <input
            v-model.number="queueCapDraft"
            class="ui-input"
            type="number"
            :min="ADULT_BACKGROUND_QUEUE_CAP_MIN"
            :max="ADULT_BACKGROUND_QUEUE_CAP_MAX"
            step="1"
            inputmode="numeric"
          >
          <span>{{ t("settings.adult.beatsUnit") }}</span>
        </div>
      </UiFieldRow>
      <p v-if="queueError" class="adult-field-error" role="alert">
        {{ queueError }}
      </p>
      <div class="adult-note">
        <span class="adult-note__mark" aria-hidden="true">i</span>
        <span>{{ t("settings.adult.queueRecommendation") }}</span>
      </div>
      <div class="adult-section-footer">
        <span>{{ t("settings.adult.queueCurrent", { value: adultStore.backgroundQueueCap }) }}</span>
        <UiButton type="button" size="sm" variant="primary" @click="saveQueueSettings">
          {{ t("settings.adult.saveQueue") }}
        </UiButton>
      </div>
    </UiSection>

    <UiSection
      :title="t('settings.adult.resetTitle')"
      :description="t('settings.adult.resetHelp')"
    >
      <div class="adult-section-footer adult-section-footer--reset">
        <span>{{ t("settings.adult.resetMemoryNote") }}</span>
        <UiButton
          class="adult-reset-button"
          type="button"
          size="sm"
          variant="secondary"
          @click="resetAdultSettings"
        >
          {{ t("settings.adult.resetButton") }}
        </UiButton>
      </div>
    </UiSection>

    <div v-if="confirmOpen" class="adult-confirm-backdrop" role="presentation">
      <section
        class="adult-confirm"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('settings.adult.legalTitle'))"
      >
        <h3>{{ t("settings.adult.legalTitle") }}</h3>
        <p>{{ t("settings.adult.legalBody") }}</p>
        <div class="adult-confirm-actions">
          <UiButton type="button" variant="primary" @click="confirmAdult">
            {{ t("settings.adult.confirm") }}
          </UiButton>
          <UiButton type="button" variant="secondary" @click="cancelAdult">
            {{ t("settings.adult.cancel") }}
          </UiButton>
        </div>
      </section>
    </div>
  </form>
</template>

<style scoped>
.adult-settings {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-6, 24px);
  color: var(--tool-text, var(--text-primary));
}

.adult-settings :deep(.ui-section__head) {
  margin-bottom: var(--tool-space-3, 12px);
}

.adult-settings :deep(.ui-section__title-row) {
  display: flex;
  align-items: center;
  gap: var(--tool-space-2, 8px);
  flex-wrap: wrap;
}

.adult-settings :deep(.ui-section__title) {
  margin: 0;
  color: var(--tool-text, var(--text-primary));
  font-size: var(--tool-fs-md, 13px);
  font-weight: 600;
  line-height: 1.5;
}

.adult-settings :deep(.ui-section__desc) {
  margin: var(--tool-space-1, 4px) 0 0;
  color: var(--tool-text-muted, var(--text-secondary));
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.5;
}

.adult-settings :deep(.ui-section__body) {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-2, 8px);
}

.adult-settings :deep(.ui-field-row) {
  display: grid;
  grid-template-columns: minmax(116px, 0.8fr) minmax(0, 1.2fr);
  align-items: center;
  gap: var(--tool-space-3, 12px);
  min-height: var(--tool-row-h, 32px);
}

.adult-settings :deep(.ui-field-row__label) {
  min-width: 0;
  color: var(--tool-text, var(--text-primary));
  font-size: var(--tool-fs-md, 13px);
  line-height: 1.5;
}

.adult-settings :deep(.ui-field-row__control) {
  display: flex;
  min-width: 0;
  align-items: center;
}

:global(html:not([data-shell="tool"])) .adult-settings :deep(.ui-section) {
  padding: var(--tool-space-4, 16px);
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, var(--radius-app));
  background: var(--tool-chrome-editor, var(--bg-elevated));
  box-shadow: var(--shadow-sm);
}

.adult-overview {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--tool-space-4, 16px);
  padding: 0 0 var(--tool-space-4, 16px);
  border-bottom: 1px solid var(--tool-divider, var(--border-light));
}

.adult-overview__copy {
  min-width: 0;
}

.adult-overview h3 {
  margin: 0;
  font-size: 16px;
  line-height: 1.45;
}

.adult-overview .sv-lead {
  max-width: 58rem;
  margin: 5px 0 0;
  color: var(--tool-text-muted, var(--text-secondary));
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.55;
}

.adult-status {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
  min-height: 24px;
  padding: 0 9px;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: 999px;
  color: var(--tool-text-muted, var(--text-secondary));
  background: var(--tool-chrome-sidebar, var(--bg-secondary));
  font-size: var(--tool-fs-sm, 12px);
  font-weight: 600;
}

.adult-status__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.55;
}

.adult-status--active {
  border-color: color-mix(in srgb, var(--tool-accent, var(--accent)) 42%, var(--tool-border, var(--border-light)));
  color: var(--tool-accent, var(--accent));
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 9%, transparent);
}

.adult-extension-errors {
  padding: var(--tool-space-3, 12px);
  border: 1px solid color-mix(in srgb, var(--danger, #c33) 58%, transparent);
  border-left-width: 4px;
  border-radius: var(--tool-radius, 6px);
  color: var(--tool-text, var(--text-primary));
  background: color-mix(in srgb, var(--danger, #c33) 10%, transparent);
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.5;
}

.adult-extension-errors p {
  margin: 4px 0;
}

.adult-extension-errors ul {
  margin: 6px 0 0;
  padding-left: 20px;
}

.adult-empty {
  display: flex;
  align-items: flex-start;
  gap: var(--tool-space-3, 12px);
  padding: var(--tool-space-3, 12px);
  border: 1px dashed var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, 6px);
  color: var(--tool-text-muted, var(--text-secondary));
  background: color-mix(in srgb, var(--tool-chrome-sidebar, var(--bg-secondary)) 62%, transparent);
}

.adult-empty__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: 50%;
  font-size: 13px;
}

.adult-empty strong {
  color: var(--tool-text, var(--text-primary));
  font-size: var(--tool-fs-md, 13px);
}

.adult-empty p {
  margin: 4px 0 0;
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.45;
}

.adult-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 100px;
  cursor: pointer;
  user-select: none;
}

.adult-switch input {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  clip-path: inset(50%);
  white-space: nowrap;
}

.adult-switch__track {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 30px;
  height: 16px;
  padding: 2px;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: 999px;
  box-sizing: border-box;
  background: var(--tool-chrome-sidebar, var(--bg-secondary));
  transition: var(--tool-transition, var(--control-transition));
}

.adult-switch__thumb {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--tool-text-muted, var(--text-secondary));
  transition: transform 0.16s ease, background 0.16s ease;
}

.adult-switch input:checked + .adult-switch__track {
  border-color: var(--tool-accent, var(--accent));
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 25%, transparent);
}

.adult-switch input:checked + .adult-switch__track .adult-switch__thumb {
  transform: translateX(14px);
  background: var(--tool-accent, var(--accent));
}

.adult-switch input:focus-visible + .adult-switch__track {
  outline: none;
  box-shadow: 0 0 0 2px var(--focus-ring-color);
}

.adult-switch__label {
  min-width: 3.5em;
  color: var(--tool-text-muted, var(--text-secondary));
  font-size: var(--tool-fs-sm, 12px);
}

.adult-switch__label--active {
  color: var(--tool-accent, var(--accent));
}

.adult-field-label {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.adult-number-control {
  display: flex;
  align-items: center;
  gap: 8px;
  width: min(220px, 100%);
  color: var(--tool-text-muted, var(--text-secondary));
  font-size: var(--tool-fs-sm, 12px);
}

.adult-number-control .ui-input {
  width: 132px;
  font-variant-numeric: tabular-nums;
}

.adult-field-error {
  margin: 0;
  color: var(--danger, #c33);
  font-size: var(--tool-fs-sm, 12px);
}

.adult-section-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--tool-space-3, 12px);
  min-height: 32px;
  padding-top: var(--tool-space-2, 8px);
  border-top: 1px solid var(--tool-divider, var(--border-light));
  color: var(--tool-text-muted, var(--text-secondary));
  font-size: var(--tool-fs-sm, 12px);
}

.adult-section-footer--reset {
  padding-top: 0;
  border-top: 0;
}

.adult-note {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 9px 10px;
  border-left: 2px solid var(--tool-accent, var(--accent));
  color: var(--tool-text-muted, var(--text-secondary));
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 6%, transparent);
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.5;
}

.adult-note__mark {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-top: 1px;
  border: 1px solid currentColor;
  border-radius: 50%;
  font-size: 10px;
  font-weight: 700;
}

.adult-confirm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10100;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgb(0 0 0 / 55%);
}
.adult-confirm {
  width: min(520px, 100%);
  padding: 20px;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, var(--radius-app));
  color: var(--tool-text, var(--text-primary));
  background: var(--tool-chrome-editor, var(--bg-primary));
  box-shadow: var(--shadow-app, var(--shadow-md));
}

.adult-confirm h3 {
  margin: 0;
  font-size: 16px;
}

.adult-confirm p {
  color: var(--tool-text-muted, var(--text-secondary));
  line-height: 1.7;
}

.adult-confirm-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

@media (max-width: 520px) {
  .adult-overview,
  .adult-section-footer {
    align-items: stretch;
    flex-direction: column;
  }

  .adult-status {
    align-self: flex-start;
  }

  .adult-switch {
    min-width: 0;
  }

  .adult-settings :deep(.ui-field-row) {
    grid-template-columns: 1fr;
    align-items: stretch;
    gap: var(--tool-space-1, 4px);
  }
}
</style>
