<script setup lang="ts">
import type { CastAdaptIssue } from '../../composables/theater/theaterCastAdapt'
import type { TheaterCastConfig } from '../../composables/theater/theaterCastConfig'
import type { TheaterPairRelationId } from '../../composables/theater/theaterPairRelation'
import ImportProgressModal from '@oclive/shared/components/ImportProgressModal.vue'
import UiButton from '@oclive/shared/components/ui/UiButton.vue'
import UiFieldRow from '@oclive/shared/components/ui/UiFieldRow.vue'
import UiSection from '@oclive/shared/components/ui/UiSection.vue'
import UiSelect from '@oclive/shared/components/ui/UiSelect.vue'
import { useRolePackImport } from '@oclive/shared/composables/useRolePackImport'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { countAdaptedCacheEntries, resolveCastAdaptStatus } from '../../composables/theater/theaterCastAdapt'
import {
  DEFAULT_THEATER_CAST_CONFIG,
  enrichCastConfigFromRoles,
  getTheaterCastConfig,
  isHybridCast,
  resolveCastTier,
} from '../../composables/theater/theaterCastConfig'
import { THEATER_PAIR_RELATION_IDS } from '../../composables/theater/theaterPairRelation'
import TheaterCastAdaptProgress from './TheaterCastAdaptProgress.vue'

const props = defineProps<{
  active?: boolean
  applyCast?: (config: TheaterCastConfig) => Promise<void>
  applyDefaultCast?: () => Promise<void>
  clearCastAdaptCache?: () => number
  reAdaptCurrentCast?: () => Promise<void>
  castAdaptActive?: boolean
  castAdaptSteps?: string[]
  castAdaptProgressCurrent?: number
  castAdaptProgressTotal?: number
  castAdaptProgressLabel?: string
  castAdaptWaitingPhase?: 'thinking' | 'model'
  castAdaptWaitingSeconds?: number
  castAdaptSkeletonHash?: string
  castAdaptPresetId?: string
  castSkeletonReady?: boolean
  castAdaptLastIssue?: CastAdaptIssue | null
}>()

const emit = defineEmits<{
  apply: [config: TheaterCastConfig]
  notify: [payload: { type: 'success' | 'error' | 'info' | 'warning', message: string }]
}>()

type CastSlotKey = 'castA' | 'castB'

const { t, te } = useI18n()
const roleStore = useRoleStore()

const draft = ref<TheaterCastConfig>(getTheaterCastConfig())
const savedConfig = ref<TheaterCastConfig>(getTheaterCastConfig())
const applying = ref(false)
const restoring = ref(false)
const clearingCache = ref(false)
const reAdapting = ref(false)
const cacheEntryCount = ref(0)
const importTarget = ref<CastSlotKey | null>(null)

function refreshCacheCount() {
  cacheEntryCount.value = countAdaptedCacheEntries()
}

function refreshSavedConfig() {
  savedConfig.value = getTheaterCastConfig()
}

const activeCastConfig = computed(() =>
  enrichCastConfigFromRoles(savedConfig.value, roleStore.roles),
)

const castTierLabelKey = computed(() => {
  const cfg = activeCastConfig.value
  if (isHybridCast(cfg))
    return 'theater.cast.tierHybrid'
  return resolveCastTier(cfg) === 'default'
    ? 'theater.cast.tierDefault'
    : 'theater.cast.tierApplied'
})

const castAdaptStatus = computed(() =>
  resolveCastAdaptStatus(
    activeCastConfig.value,
    props.castAdaptPresetId ?? 'breakfast',
    props.castAdaptSkeletonHash ?? '',
  ),
)

const castAdaptStatusHintKey = computed(() => {
  switch (castAdaptStatus.value) {
    case 'default':
      return 'theater.cast.statusDefault'
    case 'cached':
      return 'theater.cast.statusCached'
    case 'renameOnly':
      return 'theater.cast.renameOnlyHint'
  }
  return 'theater.cast.statusDefault'
})

const castAdaptLastIssueLabel = computed(() => {
  const issue = props.castAdaptLastIssue
  if (!issue)
    return ''
  const key = `theater.cast.issue.${issue.code}`
  if (te(key))
    return t(key)
  return issue.kind === 'failure'
    ? t('theater.cast.issue.unknown')
    : t('theater.cast.issue.degradedUnknown')
})

const showAiRewriteProbBadge = computed(() => {
  const issue = props.castAdaptLastIssue
  if (issue?.kind === 'failure')
    return true
  if (resolveCastTier(activeCastConfig.value) === 'default')
    return false
  return castAdaptStatus.value !== 'cached'
})

const draftNeedsAiRewrite = computed(() =>
  resolveCastTier(enrichCastConfigFromRoles(draft.value, roleStore.roles)) !== 'default',
)

const showHybridHint = computed(() => isHybridCast(activeCastConfig.value))

const castAdaptBusy = computed(() =>
  Boolean(props.castAdaptActive)
  || applying.value
  || restoring.value
  || clearingCache.value
  || reAdapting.value,
)

const {
  conflictOpen,
  pendingPeek,
  importProgressOpen,
  importPercent,
  importMessage,
  importFileIndex,
  importFileTotal,
  importCurrentFile,
  closeConflict,
  confirmOverwrite,
  runImportWithPicker,
} = useRolePackImport({
  onImported: async (roleId) => {
    const slot = importTarget.value
    if (!slot)
      return
    const role = roleStore.roles.find(r => r.id === roleId)
    draft.value = {
      ...draft.value,
      [slot]: {
        roleId,
        displayName: role?.name ?? roleId,
      },
    }
    importTarget.value = null
  },
  onNotify: payload => emit('notify', payload),
})

function roleOptions(excludeRoleId: string) {
  return roleStore.roles.filter(r => r.id !== excludeRoleId)
}

function roleSummary(roleId: string) {
  return roleStore.roles.find(r => r.id === roleId) ?? null
}

const sameRoleSelected = computed(() =>
  draft.value.castA.roleId !== ''
  && draft.value.castA.roleId === draft.value.castB.roleId,
)

const canApply = computed(() =>
  props.castSkeletonReady !== false
  && !sameRoleSelected.value
  && roleStore.roles.some(r => r.id === draft.value.castA.roleId)
  && roleStore.roles.some(r => r.id === draft.value.castB.roleId)
  && !castAdaptBusy.value,
)

const canRestoreDefault = computed(() =>
  resolveCastTier(activeCastConfig.value) !== 'default'
  && !castAdaptBusy.value,
)

const canReAdapt = computed(() =>
  resolveCastTier(activeCastConfig.value) !== 'default'
  && !castAdaptBusy.value,
)

const canClearCache = computed(() =>
  cacheEntryCount.value > 0
  && !castAdaptBusy.value,
)

function onSlotChange(slot: CastSlotKey, roleId: string) {
  const role = roleStore.roles.find(r => r.id === roleId)
  draft.value = {
    ...draft.value,
    [slot]: {
      roleId,
      displayName: role?.name ?? roleId,
    },
  }
}

async function onImport(slot: CastSlotKey, mode: 'archive' | 'folder') {
  importTarget.value = slot
  await runImportWithPicker(mode)
}

async function onRestoreDefault() {
  if (!canRestoreDefault.value)
    return
  restoring.value = true
  try {
    if (props.applyDefaultCast)
      await props.applyDefaultCast()
    draft.value = enrichCastConfigFromRoles(
      { ...DEFAULT_THEATER_CAST_CONFIG },
      roleStore.roles,
    )
    refreshSavedConfig()
  }
  finally {
    restoring.value = false
  }
}

async function onClearCache() {
  if (!canClearCache.value)
    return
  clearingCache.value = true
  try {
    const cleared = props.clearCastAdaptCache?.() ?? 0
    refreshCacheCount()
    emit('notify', {
      type: cleared > 0 ? 'success' : 'info',
      message: cleared > 0
        ? t('theater.cast.clearCacheDone', { count: cleared })
        : t('theater.cast.clearCacheEmpty'),
    })
  }
  finally {
    clearingCache.value = false
  }
}

async function onClearAndReAdapt() {
  if (!canReAdapt.value)
    return
  reAdapting.value = true
  try {
    if (props.reAdaptCurrentCast)
      await props.reAdaptCurrentCast()
    refreshCacheCount()
  }
  finally {
    reAdapting.value = false
    refreshCacheCount()
  }
}

function onPairRelationChange(raw: string) {
  draft.value = {
    ...draft.value,
    pairRelationId: raw as TheaterPairRelationId,
  }
}

async function onApply() {
  if (!canApply.value)
    return
  if (sameRoleSelected.value) {
    emit('notify', { type: 'error', message: t('theater.cast.sameRoleError') })
    return
  }
  const missing = [draft.value.castA.roleId, draft.value.castB.roleId]
    .filter(id => !roleStore.roles.some(r => r.id === id))
  if (missing.length > 0) {
    emit('notify', { type: 'error', message: t('theater.cast.roleNotFound') })
    return
  }
  applying.value = true
  try {
    const config = enrichCastConfigFromRoles(draft.value, roleStore.roles)
    if (props.applyCast)
      await props.applyCast(config)
    else
      emit('apply', config)
    refreshSavedConfig()
  }
  finally {
    applying.value = false
  }
}

function resetDraft() {
  draft.value = enrichCastConfigFromRoles(savedConfig.value, roleStore.roles)
}

watch(
  () => props.active,
  (open) => {
    if (open) {
      void roleStore.loadRoles()
      refreshSavedConfig()
      resetDraft()
      refreshCacheCount()
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="theater-cast-panel">
    <div class="theater-cast-status">
      <p class="theater-cast-status__title">
        {{ t(castTierLabelKey) }}
      </p>
      <p class="theater-cast-status__hint">
        {{ t(castAdaptStatusHintKey) }}
      </p>
      <div
        v-if="castAdaptLastIssueLabel"
        class="theater-cast-status__issue-row"
      >
        <p
          class="theater-cast-status__issue"
          :class="castAdaptLastIssue?.kind === 'failure' ? 'theater-cast-status__issue--failure' : 'theater-cast-status__issue--degraded'"
        >
          {{ castAdaptLastIssueLabel }}
        </p>
        <span
          v-if="showAiRewriteProbBadge"
          class="theater-cast-status__prob-badge"
          :title="t('theater.cast.aiRewriteProbHint')"
        >
          {{ t('theater.cast.aiRewriteProbBadge') }}
        </span>
      </div>
      <p class="theater-cast-status__hint theater-cast-status__hint--design">
        {{ t('theater.cast.modeDesignHint') }}
      </p>
      <p v-if="showHybridHint" class="theater-cast-status__hint">
        {{ t('theater.cast.tierHybridHint') }}
      </p>
    </div>

    <TheaterCastAdaptProgress
      v-if="castAdaptActive"
      :active="castAdaptActive"
      :steps="castAdaptSteps ?? []"
      :progress-current="castAdaptProgressCurrent ?? 0"
      :progress-total="castAdaptProgressTotal ?? 0"
      :progress-label="castAdaptProgressLabel ?? ''"
      :waiting-phase="castAdaptWaitingPhase ?? 'thinking'"
      :waiting-seconds="castAdaptWaitingSeconds ?? 0"
    />

    <p class="theater-cast-lead">
      {{ t('theater.cast.lead') }}
    </p>

    <UiSection
      v-for="slot in (['castA', 'castB'] as const)"
      :key="slot"
      :title="t(slot === 'castA' ? 'theater.cast.slotA' : 'theater.cast.slotB')"
    >
      <UiFieldRow :label="t('theater.cast.roleSelect')">
        <UiSelect
          :model-value="draft[slot].roleId"
          @update:model-value="onSlotChange(slot, $event)"
        >
          <option
            v-for="role in roleOptions(slot === 'castA' ? draft.castB.roleId : draft.castA.roleId)"
            :key="role.id"
            :value="role.id"
          >
            {{ role.name }} ({{ role.id }})
          </option>
        </UiSelect>
      </UiFieldRow>

      <UiFieldRow :label="t('theater.cast.packInfo')">
        <p v-if="roleSummary(draft[slot].roleId)" class="theater-cast-meta">
          {{ roleSummary(draft[slot].roleId)?.name }}
          · v{{ roleSummary(draft[slot].roleId)?.version }}
          · {{ roleSummary(draft[slot].roleId)?.author }}
        </p>
        <p v-else class="theater-cast-meta theater-cast-meta--muted">
          {{ t('theater.cast.roleNotFound') }}
        </p>
      </UiFieldRow>

      <UiFieldRow :label="t('theater.cast.import')">
        <div class="theater-cast-import-row">
          <UiButton
            size="sm"
            variant="secondary"
            :disabled="importProgressOpen"
            @click="onImport(slot, 'archive')"
          >
            {{ t('common.rolePack.importArchive') }}
          </UiButton>
          <UiButton
            size="sm"
            variant="secondary"
            :disabled="importProgressOpen"
            @click="onImport(slot, 'folder')"
          >
            {{ t('common.rolePack.importFolder') }}
          </UiButton>
        </div>
      </UiFieldRow>
    </UiSection>

    <p v-if="sameRoleSelected" class="theater-cast-error">
      {{ t('theater.cast.sameRoleError') }}
    </p>

    <UiSection :title="t('theater.cast.cacheSection')">
      <p class="theater-cast-meta">
        {{ t('theater.cast.cacheStatus', { count: cacheEntryCount }) }}
      </p>
      <p class="theater-cast-meta theater-cast-meta--muted">
        {{ t('theater.cast.cacheHint') }}
      </p>
      <div class="theater-cast-import-row theater-cast-cache-actions">
        <UiButton
          size="sm"
          variant="secondary"
          :disabled="!canClearCache"
          @click="onClearCache"
        >
          {{ t('theater.cast.clearCache') }}
        </UiButton>
        <UiButton
          size="sm"
          variant="secondary"
          :disabled="!canReAdapt"
          @click="onClearAndReAdapt"
        >
          {{ t('theater.cast.clearCacheAndReAdapt') }}
          <span
            class="theater-cast-apply-badge theater-cast-apply-badge--secondary"
            :title="t('theater.cast.aiRewriteProbHint')"
          >
            {{ t('theater.cast.aiRewriteProbBadge') }}
          </span>
        </UiButton>
      </div>
    </UiSection>

    <div class="theater-cast-actions">
      <UiFieldRow :label="t('theater.cast.pairRelationField')" class="theater-cast-relation-field">
        <UiSelect
          :model-value="draft.pairRelationId"
          @change="onPairRelationChange(($event.target as HTMLSelectElement).value)"
        >
          <option
            v-for="relId in THEATER_PAIR_RELATION_IDS"
            :key="relId"
            :value="relId"
          >
            {{ t(`theater.cast.pairRelationOpts.${relId}`) }}
          </option>
        </UiSelect>
      </UiFieldRow>
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="!canRestoreDefault"
        @click="onRestoreDefault"
      >
        {{ t('theater.cast.restoreDefault') }}
      </UiButton>
      <UiButton size="sm" variant="secondary" :disabled="castAdaptBusy" @click="resetDraft">
        {{ t('theater.cast.reset') }}
      </UiButton>
      <UiButton
        size="sm"
        variant="primary"
        :disabled="!canApply"
        @click="onApply"
      >
        {{ t('theater.cast.apply') }}
        <span
          v-if="draftNeedsAiRewrite"
          class="theater-cast-apply-badge"
          :title="t('theater.cast.aiRewriteProbHint')"
        >
          {{ t('theater.cast.aiRewriteProbBadge') }}
        </span>
      </UiButton>
    </div>

    <p class="theater-cast-hint">
      {{ t('theater.cast.pairRelationHint') }}
    </p>
    <p class="theater-cast-hint">
      {{ t('theater.cast.applyHint') }}
    </p>

    <ImportProgressModal
      :open="importProgressOpen"
      :percent="importPercent"
      :message="importMessage"
      :file-index="importFileIndex"
      :file-total="importFileTotal"
      :current-file="importCurrentFile"
    />

    <Teleport to="body">
      <div
        v-if="conflictOpen && pendingPeek"
        class="theater-cast-conflict-backdrop"
        role="dialog"
        aria-modal="true"
      >
        <div class="theater-cast-conflict-card" @click.stop>
          <h3 class="theater-cast-conflict-title">
            {{ t('common.rolePack.conflictTitle') }}
          </h3>
          <p class="theater-cast-conflict-body">
            {{
              t('common.rolePack.conflictBody', {
                id: pendingPeek.id,
                name: pendingPeek.name,
                version: pendingPeek.version,
              })
            }}
          </p>
          <div class="theater-cast-conflict-actions">
            <UiButton size="sm" variant="secondary" @click="closeConflict">
              {{ t('common.cancel') }}
            </UiButton>
            <UiButton
              size="sm"
              variant="primary"
              :disabled="importProgressOpen"
              @click="confirmOverwrite"
            >
              {{ t('common.rolePack.overwrite') }}
            </UiButton>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.theater-cast-status {
  margin: 0 0 12px;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--tool-accent, #6b8cff) 8%, transparent);
  border: 1px solid var(--border-light);
}

.theater-cast-status__title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.theater-cast-status__hint {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
}

.theater-cast-status__hint--design {
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-tertiary, var(--text-secondary));
}

.theater-cast-status__issue-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-top: 8px;
}

.theater-cast-status__issue {
  margin: 0;
  flex: 1;
  min-width: 0;
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.45;
}

.theater-cast-status__prob-badge {
  flex-shrink: 0;
  margin-top: 8px;
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
  line-height: 1.3;
  color: #9a4a4a;
  background: color-mix(in srgb, #c45c5c 12%, transparent);
  border: 1px solid color-mix(in srgb, #c45c5c 28%, var(--border-light));
  cursor: help;
}

.theater-cast-status__issue--failure {
  color: #9a4a4a;
  background: color-mix(in srgb, #c45c5c 10%, transparent);
  border: 1px solid color-mix(in srgb, #c45c5c 25%, var(--border-light));
}

.theater-cast-status__issue--degraded {
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--tool-accent, #6b8cff) 6%, transparent);
  border: 1px solid var(--border-light);
}

.theater-cast-lead {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}

.theater-cast-meta {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.theater-cast-meta--muted {
  color: var(--text-tertiary, var(--text-secondary));
}

.theater-cast-import-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.theater-cast-cache-actions {
  margin-top: 8px;
}

.theater-cast-error {
  margin: 0 0 8px;
  font-size: 12px;
  color: #c45c5c;
}

.theater-cast-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}

.theater-cast-apply-badge {
  margin-left: 6px;
  display: inline-block;
  padding: 0 5px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
  vertical-align: middle;
  color: rgba(255, 255, 255, 0.92);
  background: color-mix(in srgb, #fff 18%, transparent);
  border: 1px solid color-mix(in srgb, #fff 28%, transparent);
  cursor: help;
}

.theater-cast-apply-badge--secondary {
  color: #9a4a4a;
  background: color-mix(in srgb, #c45c5c 10%, transparent);
  border: 1px solid color-mix(in srgb, #c45c5c 22%, var(--border-light));
}

.theater-cast-relation-field {
  flex: 1 1 160px;
  margin-right: auto;
  margin-bottom: 0;
}

.theater-cast-hint {
  margin: 10px 0 0;
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
}

.theater-cast-conflict-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.45));
}

.theater-cast-conflict-card {
  max-width: 400px;
  width: 100%;
  padding: 20px;
  border-radius: 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
}

.theater-cast-conflict-title {
  margin: 0 0 12px;
  font-size: 16px;
}

.theater-cast-conflict-body {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.theater-cast-conflict-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
