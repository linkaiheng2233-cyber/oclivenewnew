<script setup lang="ts">
import { getUserIdentityState } from '@oclive/shared/api'
import {
  cancelAdultBeatQueuesForRole,
  cancelAllAdultBeatQueues,
  notifyAdultBeatQueueCapacityChanged,
} from '@oclive/shared/lib/adultBeatQueue'
import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const adultStore = useAdultInteractionStore()
const roleStore = useRoleStore()

const confirmOpen = ref(false)
const pendingRoleId = ref<string | null>(null)
const intervalDraft = ref(adultStore.pacingIntervalMs)
const queueCapDraft = ref(adultStore.backgroundQueueCap)

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

async function currentIdentityAllowsAdult(roleId: string): Promise<boolean> {
  if (roleId !== roleStore.currentRoleId)
    return true
  const state = await getUserIdentityState(roleId)
  const current = state.identities.find(identity => identity.id === state.current_identity_id)
  if (current?.adult_eligible !== false)
    return true
  window.alert(String(t('settings.adult.minorIdentityBlocked')))
  return false
}

async function requestEnable(roleId?: string) {
  if (roleId && !await currentIdentityAllowsAdult(roleId))
    return
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
    return
  }
  if (!window.confirm(String(t('settings.adult.globalHelp'))))
    return
  void cancelAllAdultBeatQueues()
  adultStore.setGlobalEnabled(false)
}

function onRoleToggle(roleId: string, event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  if (enabled) {
    requestEnable(roleId)
    return
  }
  void cancelAdultBeatQueuesForRole(roleId)
  adultStore.setRoleEnabled(roleId, false)
}

function savePacing() {
  adultStore.setPacingOverride(adultStore.pacingOverrideEnabled, intervalDraft.value)
  intervalDraft.value = adultStore.pacingIntervalMs
}

function onQueueToggle(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  if (enabled && !adultStore.backgroundQueueWarningAccepted) {
    const accepted = window.confirm(String(t('settings.adult.queueWarning')))
    if (!accepted)
      return
    adultStore.setBackgroundQueue(true, queueCapDraft.value, true)
  }
  else {
    adultStore.setBackgroundQueue(
      enabled,
      queueCapDraft.value,
      adultStore.backgroundQueueWarningAccepted,
    )
  }
  if (!enabled)
    void cancelAllAdultBeatQueues()
  notifyAdultBeatQueueCapacityChanged()
}

function saveQueueSettings() {
  adultStore.setBackgroundQueue(
    adultStore.backgroundQueueEnabled,
    queueCapDraft.value,
    adultStore.backgroundQueueWarningAccepted,
  )
  queueCapDraft.value = adultStore.backgroundQueueCap
  notifyAdultBeatQueueCapacityChanged()
}
</script>

<template>
  <div class="sv-body adult-settings">
    <h3>{{ t("settings.adult.title") }}</h3>
    <p class="sv-lead">
      {{ t("settings.adult.lead") }}
    </p>

    <section class="adult-card">
      <label class="adult-toggle-row">
        <span>
          <strong>{{ t("settings.adult.globalLabel") }}</strong>
          <small>{{ t("settings.adult.globalHelp") }}</small>
        </span>
        <input
          type="checkbox"
          :checked="adultStore.globalEnabled"
          @change="onGlobalToggle"
        >
      </label>
    </section>

    <section class="adult-card">
      <h4>{{ t("settings.adult.rolesTitle") }}</h4>
      <p v-if="adultRoles.length === 0" class="adult-muted">
        {{ t("settings.adult.rolesEmpty") }}
      </p>
      <label
        v-for="role in adultRoles"
        :key="role.id"
        class="adult-toggle-row adult-role-row"
      >
        <span>
          <strong>{{ role.name }}</strong>
          <small>{{ role.id }}</small>
        </span>
        <input
          type="checkbox"
          :checked="adultStore.roleIsEnabled(role.id)"
          @change="onRoleToggle(role.id, $event)"
        >
      </label>
    </section>

    <section class="adult-card">
      <h4>{{ t("settings.adult.pacingTitle") }}</h4>
      <label class="adult-toggle-row">
        <span>{{ t("settings.adult.pacingOverride") }}</span>
        <input v-model="adultStore.pacingOverrideEnabled" type="checkbox">
      </label>
      <label class="adult-field">
        <span>
          {{ t("settings.adult.intervalLabel") }}
          <span class="adult-help" :title="String(t('settings.adult.intervalHelp'))">?</span>
        </span>
        <input v-model.number="intervalDraft" type="number" min="1" step="100">
      </label>
      <button type="button" class="adult-save" @click="savePacing">
        {{ t("keybindings.save") }}
      </button>
    </section>

    <section class="adult-card">
      <h4>{{ t("settings.adult.queueTitle") }}</h4>
      <label class="adult-toggle-row">
        <span>
          <strong>{{ t("settings.adult.queueEnable") }}</strong>
          <small>{{ t("settings.adult.queueHelp") }}</small>
        </span>
        <input
          type="checkbox"
          :checked="adultStore.backgroundQueueEnabled"
          @change="onQueueToggle"
        >
      </label>
      <label class="adult-field">
        <span>
          {{ t("settings.adult.queueCapLabel") }}
          <span class="adult-help" :title="String(t('settings.adult.queueCapHelp'))">?</span>
        </span>
        <input v-model.number="queueCapDraft" type="number" min="1" step="1">
      </label>
      <small class="adult-muted">{{ t("settings.adult.queueRecommendation") }}</small>
      <button type="button" class="adult-save" @click="saveQueueSettings">
        {{ t("keybindings.save") }}
      </button>
    </section>

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
          <button type="button" class="adult-save" @click="confirmAdult">
            {{ t("settings.adult.confirm") }}
          </button>
          <button type="button" class="adult-secondary" @click="cancelAdult">
            {{ t("settings.adult.cancel") }}
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.adult-settings {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.adult-settings h3,
.adult-settings h4 {
  margin: 0;
}
.adult-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-app);
  background: var(--bg-elevated);
}
.adult-toggle-row,
.adult-field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.adult-toggle-row > span {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.adult-toggle-row small,
.adult-muted {
  color: var(--text-secondary);
  line-height: 1.5;
}
.adult-role-row + .adult-role-row {
  padding-top: 10px;
  border-top: 1px solid var(--border-light);
}
.adult-field input[type="number"] {
  width: 132px;
  padding: 7px 9px;
  border: 1px solid var(--border-light);
  border-radius: 6px;
  color: var(--text-primary);
  background: var(--bg-primary);
}
.adult-help {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 17px;
  height: 17px;
  margin-left: 4px;
  border: 1px solid currentColor;
  border-radius: 50%;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: help;
}
.adult-save,
.adult-secondary {
  width: fit-content;
  padding: 8px 14px;
  border: 1px solid var(--border-light);
  border-radius: 7px;
  cursor: pointer;
}
.adult-save {
  color: var(--text-accent);
  background: var(--accent);
}
.adult-secondary {
  color: var(--text-primary);
  background: var(--bg-elevated);
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
  border: 1px solid var(--border-light);
  border-radius: var(--radius-app);
  color: var(--text-primary);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.adult-confirm p {
  line-height: 1.7;
}
.adult-confirm-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
</style>
