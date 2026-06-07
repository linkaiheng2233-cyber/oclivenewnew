<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getUserIdentityState,
  OCLIVE_DEFAULT_IDENTITY_SENTINEL,
  setSceneUserIdentity,
  setUserIdentity,
  type UserIdentityStateResponse,
} from '../../api'
import { useAppToast } from '../../composables/useAppToast'
import { useRoleStore } from '../../stores/roleStore'
import { useUiStore } from '../../stores/uiStore'
import HelpHint from '../shared/HelpHint.vue'

const props = withDefaults(
  defineProps<{
    variant?: 'full' | 'compact'
    /** Settings → General: show section titles and catalog-empty hint */
    settingsLayout?: boolean
  }>(),
  {
    variant: 'full',
    settingsLayout: false,
  },
)

const { t, te } = useI18n()
const { showToast } = useAppToast()
const roleStore = useRoleStore()
const uiStore = useUiStore()
const busy = ref(false)
const identityState = ref<UserIdentityStateResponse | null>(null)

const identityRows = computed(() => {
  const rows = identityState.value?.identities ?? []
  if (!rows.length)
    return []
  const defaultId = identityState.value?.default_identity_id ?? ''
  const defaultLabel = rows.find(r => r.id === defaultId)?.display_name ?? defaultId
  return [
    { id: OCLIVE_DEFAULT_IDENTITY_SENTINEL, name: t('roleRuntime.identityFollowDefault', { name: defaultLabel || '—' }) },
    ...rows.map(r => ({ id: r.id, name: r.display_name || r.id })),
  ]
})

const identitySelectValue = computed(() => {
  if (!identityState.value)
    return ''
  if (identityState.value.use_manifest_default)
    return OCLIVE_DEFAULT_IDENTITY_SENTINEL
  return identityState.value.current_identity_id
})

const hasIdentityCatalog = computed(() => identityRows.value.length > 0)

const postProcessorStatusText = computed(() => {
  const info = roleStore.roleInfo
  if (!info.replyPostProcessorEnabled) {
    return t('roleRuntime.postProcessorOff')
  }
  const backendKey = `roleRuntime.backend${info.replyPostProcessorBackend.charAt(0).toUpperCase()}${info.replyPostProcessorBackend.slice(1)}`
  const backendLabel = te(backendKey) ? t(backendKey) : info.replyPostProcessorBackend
  const profile = info.replyPostProcessorProfile ?? '—'
  return t('roleRuntime.postProcessorOn', { backend: backendLabel, profile })
})

async function refreshIdentityState(): Promise<void> {
  const roleId = roleStore.currentRoleId
  if (!roleId) {
    identityState.value = null
    return
  }
  try {
    identityState.value = await getUserIdentityState(
      roleId,
      roleStore.roleInfo.identityBinding === 'per_scene' ? uiStore.sceneId : null,
    )
  }
  catch {
    identityState.value = null
  }
}

onMounted(() => {
  void refreshIdentityState()
})

watch(
  () => [roleStore.currentRoleId, uiStore.sceneId, roleStore.roleInfo.identityBinding] as const,
  () => {
    void refreshIdentityState()
  },
)

async function onIdentityChange(ev: Event) {
  const next = (ev.target as HTMLSelectElement).value
  if (next === identitySelectValue.value)
    return
  busy.value = true
  try {
    const perScene = roleStore.roleInfo.identityBinding === 'per_scene'
    if (perScene && next !== OCLIVE_DEFAULT_IDENTITY_SENTINEL) {
      identityState.value = await setSceneUserIdentity(roleStore.currentRoleId, uiStore.sceneId, next)
    }
    else {
      identityState.value = await setUserIdentity(roleStore.currentRoleId, next)
    }
    await roleStore.refreshRoleInfo()
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <div
    class="role-identity-controls"
    :class="{
      'role-identity-controls--compact': variant === 'compact',
      'role-identity-controls--settings': settingsLayout,
    }"
  >
    <template v-if="settingsLayout">
      <div class="ric-settings-block">
        <div v-if="variant === 'full'" class="ric-settings-head">
          <span class="ric-label">{{ t('settings.userIdentitySectionTitle') }}</span>
          <HelpHint :text="t('settings.userIdentitySectionLead')" />
        </div>
        <p v-if="!hasIdentityCatalog" class="ric-hint">
          {{ t('settings.noIdentityCatalogHint') }}
        </p>
        <div v-if="hasIdentityCatalog" class="row">
          <label :for="`identity-select-${variant}`">{{ t('roleRuntime.userIdentity') }}</label>
          <select
            :id="`identity-select-${variant}`"
            class="select"
            :disabled="busy"
            :value="identitySelectValue"
            @change="onIdentityChange"
          >
            <option v-for="r in identityRows" :key="r.id" :value="r.id">
              {{ r.name || r.id }}
            </option>
          </select>
        </div>
      </div>
      <div class="ric-settings-block">
        <div v-if="variant === 'full'" class="ric-settings-head">
          <span class="ric-label">{{ t('settings.postProcessorSectionTitle') }}</span>
        </div>
        <p class="sub post-processor-status">
          {{ postProcessorStatusText }}
        </p>
      </div>
    </template>

    <template v-else-if="variant === 'compact'">
      <select
        v-if="hasIdentityCatalog"
        class="ric-compact-select"
        :disabled="busy"
        :value="identitySelectValue"
        :aria-label="t('roleRuntime.userIdentity')"
        @change="onIdentityChange"
      >
        <option v-for="r in identityRows" :key="r.id" :value="r.id">
          {{ r.name || r.id }}
        </option>
      </select>
      <p class="ric-compact-status">
        {{ postProcessorStatusText }}
      </p>
    </template>

    <template v-else>
      <div v-if="hasIdentityCatalog" class="row">
        <label for="identity-select-full">{{ t('roleRuntime.userIdentity') }}</label>
        <select
          id="identity-select-full"
          class="select"
          :disabled="busy"
          :value="identitySelectValue"
          @change="onIdentityChange"
        >
          <option v-for="r in identityRows" :key="r.id" :value="r.id">
            {{ r.name || r.id }}
          </option>
        </select>
      </div>
      <p class="sub post-processor-status">
        {{ postProcessorStatusText }}
      </p>
    </template>
  </div>
</template>

<style scoped>
.role-identity-controls--compact {
  flex-shrink: 0;
  padding: 6px 12px 10px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  border-top: 1px solid var(--border-light);
}
.ric-compact-select {
  width: 100%;
  margin-bottom: 4px;
  padding: 4px 6px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font: inherit;
  color: inherit;
}
.ric-compact-status {
  margin: 0;
  font-size: 11px;
  opacity: 0.9;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
label {
  min-width: 72px;
  color: var(--text-secondary);
}
.select {
  flex: 1;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.post-processor-status {
  margin-top: 0;
}
.ric-settings-block + .ric-settings-block {
  margin-top: 4px;
}
.ric-settings-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}
.ric-label {
  font-weight: 600;
  font-size: 14px;
}
.ric-hint {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
