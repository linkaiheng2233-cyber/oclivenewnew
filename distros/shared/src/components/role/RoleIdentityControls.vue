<script setup lang="ts">
import { OCLIVE_DEFAULT_IDENTITY_SENTINEL } from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useUserIdentityState } from '@oclive/shared/composables/useUserIdentityState'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import UiFieldRow from '../ui/UiFieldRow.vue'
import UiSelect from '../ui/UiSelect.vue'

withDefaults(
  defineProps<{
    variant?: 'full' | 'compact'
    /** Settings → General: field row layout without duplicate section chrome */
    settingsLayout?: boolean
  }>(),
  {
    variant: 'full',
    settingsLayout: false,
  },
)

const { t } = useI18n()
const { showToast } = useAppToast()
const {
  loading,
  hasCatalog,
  identitySelectValue,
  identityState,
  setIdentity,
} = useUserIdentityState()

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

async function onIdentityChange(ev: Event) {
  const next = (ev.target as HTMLSelectElement).value
  try {
    await setIdentity(next)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
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
      <p v-if="!hasCatalog" class="ric-hint">
        {{ t('settings.noIdentityCatalogHint') }}
      </p>
      <UiFieldRow v-else :label="t('roleRuntime.userIdentity')">
        <UiSelect
          :model-value="identitySelectValue"
          :disabled="loading"
          @change="onIdentityChange"
        >
          <option v-for="r in identityRows" :key="r.id" :value="r.id">
            {{ r.name || r.id }}
          </option>
        </UiSelect>
      </UiFieldRow>
    </template>

    <template v-else-if="variant === 'compact'">
      <select
        v-if="hasCatalog"
        class="ric-compact-select"
        :disabled="loading"
        :value="identitySelectValue"
        :aria-label="t('roleRuntime.userIdentity')"
        @change="onIdentityChange"
      >
        <option v-for="r in identityRows" :key="r.id" :value="r.id">
          {{ r.name || r.id }}
        </option>
      </select>
    </template>

    <template v-else>
      <div v-if="hasCatalog" class="row">
        <label for="identity-select-full">{{ t('roleRuntime.userIdentity') }}</label>
        <select
          id="identity-select-full"
          class="select"
          :disabled="loading"
          :value="identitySelectValue"
          @change="onIdentityChange"
        >
          <option v-for="r in identityRows" :key="r.id" :value="r.id">
            {{ r.name || r.id }}
          </option>
        </select>
      </div>
    </template>
  </div>
</template>

<style scoped>
.role-identity-controls--compact {
  flex-shrink: 0;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px) var(--tool-space-3, 12px);
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  text-align: center;
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}
.ric-compact-select {
  width: 100%;
  margin-bottom: 0;
  padding: var(--tool-space-1, 4px) var(--tool-space-2, 8px);
  border-radius: var(--tool-radius, 4px);
  border: 1px solid var(--tool-border, var(--border-light));
  background: var(--tool-elevated, var(--bg-elevated));
  font: inherit;
  color: var(--tool-text, inherit);
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
.ric-hint {
  margin: 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  line-height: 1.45;
}
</style>
