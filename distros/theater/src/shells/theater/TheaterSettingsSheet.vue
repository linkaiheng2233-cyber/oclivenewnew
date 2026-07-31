<script setup lang="ts">
import type { LocalePreference } from '@oclive/shared/i18n'

import type { TheaterCastConfig } from '../../composables/theater/theaterCastConfig'

import type { TheaterPokeMode } from '../../composables/useTheaterPokeSettings'

import UiButton from '@oclive/shared/components/ui/UiButton.vue'

import UiFieldRow from '@oclive/shared/components/ui/UiFieldRow.vue'

import UiSection from '@oclive/shared/components/ui/UiSection.vue'

import UiSelect from '@oclive/shared/components/ui/UiSelect.vue'

import { useAppToast } from '@oclive/shared/composables/useAppToast'

import { useOcliveAppearance } from '@oclive/shared/composables/useOcliveAppearance'

import { getLocalePreference, setLocalePreference } from '@oclive/shared/i18n'

import { defineAsyncComponent, onMounted, onUnmounted, ref, watch } from 'vue'

import { useI18n } from 'vue-i18n'
import {
  getTheaterCustomLeadCast,
  getTheaterPokeMode,
  getTheaterVariantSwipeEnabled,
  setTheaterCustomLeadCast,
  setTheaterPokeMode,
  setTheaterVariantSwipeEnabled,

} from '../../composables/useTheaterPokeSettings'

import {

  getTheaterPortraitLayout,

  resetTheaterPortraitLayout,

  setTheaterPortraitMaxHeight,

  setTheaterPortraitWidth,

  THEATER_PORTRAIT_LIMITS,

} from '../../composables/useTheaterPortraitLayout'
import TheaterCastPanel from './TheaterCastPanel.vue'

const props = defineProps<{
  visible: boolean
  settingsTab?: SettingsTab
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
  castAdaptLastIssue?: import('../../composables/theater/theaterCastAdapt').CastAdaptIssue | null
}>()

const emit = defineEmits<{
  'close': []
  'update:settingsTab': [tab: SettingsTab]
  'applyCast': [config: import('../../composables/theater/theaterCastConfig').TheaterCastConfig]
  'notify': [payload: { type: 'success' | 'error' | 'info' | 'warning', message: string }]
}>()

const KernelConnectionSettingsPanel = defineAsyncComponent(
  () => import('@oclive/shared/components/settings/KernelConnectionSettingsPanel.vue'),
)

const ModelManagerBody = defineAsyncComponent(
  () => import('@oclive/shared/components/model/ModelManagerBody.vue'),
)

type SettingsTab = 'general' | 'stage' | 'cast' | 'model'

const { t } = useI18n()

const { showToast } = useAppToast()

const { themeCycleLabel, cycleTheme, bumpScale, scaleLabel } = useOcliveAppearance()

const localePreference = ref<LocalePreference>(getLocalePreference())

const tab = ref<SettingsTab>('general')

function setTab(next: SettingsTab) {
  tab.value = next
  emit('update:settingsTab', next)
}

watch(
  () => props.settingsTab,
  (next) => {
    if (next && next !== tab.value)
      tab.value = next
  },
)

watch(
  () => props.visible,
  (open) => {
    if (open && props.settingsTab)
      tab.value = props.settingsTab
  },
)

const portraitLayout = ref(getTheaterPortraitLayout())

const pokeMode = ref<TheaterPokeMode>(getTheaterPokeMode())
const variantSwipe = ref(getTheaterVariantSwipeEnabled())
const customLeadCast = ref<'a' | 'b'>(getTheaterCustomLeadCast())

function onPokeModeChange(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value as TheaterPokeMode
  setTheaterPokeMode(v)
  pokeMode.value = v
}

function onVariantSwipeChange(ev: Event) {
  const enabled = (ev.target as HTMLInputElement).checked
  setTheaterVariantSwipeEnabled(enabled)
  variantSwipe.value = enabled
}

function onCustomLeadChange(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value as 'a' | 'b'
  setTheaterCustomLeadCast(v)
  customLeadCast.value = v
}

function onLocaleChange(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value as LocalePreference

  setLocalePreference(v)

  localePreference.value = v
}

function onPortraitWidthInput(ev: Event) {
  const v = Number((ev.target as HTMLInputElement).value)

  portraitLayout.value.width = setTheaterPortraitWidth(v)
}

function onPortraitHeightInput(ev: Event) {
  const v = Number((ev.target as HTMLInputElement).value)

  portraitLayout.value.maxHeight = setTheaterPortraitMaxHeight(v)
}

function onResetPortraitLayout() {
  portraitLayout.value = resetTheaterPortraitLayout()

  showToast('success', t('theater.settings.portraitResetDone'))
}

function onBackdrop() {
  emit('close')
}

function onBack() {
  emit('close')
}

function onKeydown(e: KeyboardEvent) {
  if (!props.visible)

    return

  if (e.key === 'Escape') {
    e.preventDefault()

    e.stopPropagation()

    onBack()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)

  portraitLayout.value = getTheaterPortraitLayout()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <Teleport to="body">
    <div

      v-if="visible"

      class="sv-backdrop"

      role="dialog"

      aria-modal="true"

      :aria-label="t('theater.settings.title')"

      @click.self="onBackdrop"
    >
      <div class="sv-dialog theater-settings-dialog" @click.stop>
        <header class="sv-head">
          <h2 class="sv-title">
            {{ t('theater.settings.title') }}
          </h2>

          <button

            type="button"

            class="sv-close"

            :aria-label="t('settings.closeAria')"

            @click="onBack"
          >
            ×
          </button>
        </header>

        <nav class="sv-nav" :aria-label="t('settings.ariaNav')">
          <button

            type="button"

            class="sv-nav-btn"

            :aria-current="tab === 'general' ? 'page' : undefined"

            @click="setTab('general')"
          >
            {{ t('settings.tabGeneral') }}
          </button>

          <button

            type="button"

            class="sv-nav-btn"

            :aria-current="tab === 'stage' ? 'page' : undefined"

            @click="setTab('stage')"
          >
            {{ t('theater.settings.tabStage') }}
          </button>

          <button

            type="button"

            class="sv-nav-btn"

            :aria-current="tab === 'cast' ? 'page' : undefined"

            @click="setTab('cast')"
          >
            {{ t('theater.settings.tabCast') }}
          </button>

          <button

            type="button"

            class="sv-nav-btn"

            :aria-current="tab === 'model' ? 'page' : undefined"

            @click="setTab('model')"
          >
            {{ t('theater.settings.tabModel') }}
          </button>
        </nav>

        <div class="sv-body">
          <template v-if="tab === 'general'">
            <p class="sv-lead">
              {{ t('theater.settings.lead') }}
            </p>

            <UiSection

              :title="t('settings.appearanceSectionTitle')"

              :description="t('settings.appearanceSectionHelp')"
            >
              <UiFieldRow :label="t('app.locale.label')">
                <UiSelect

                  :model-value="localePreference"

                  @change="onLocaleChange"
                >
                  <option value="system">
                    {{ t('app.locale.system') }}
                  </option>

                  <option value="zh-CN">
                    {{ t('app.locale.zhCN') }}
                  </option>

                  <option value="en-US">
                    {{ t('app.locale.enUS') }}
                  </option>
                </UiSelect>
              </UiFieldRow>

              <UiFieldRow :label="t('app.more.ui')" class="sv-appearance-row">
                <div class="sv-appearance-controls">
                  <UiButton size="sm" variant="secondary" @click="bumpScale(-1)">
                    A−
                  </UiButton>

                  <span class="sv-appearance-scale">{{ scaleLabel }}</span>

                  <UiButton size="sm" variant="secondary" @click="bumpScale(1)">
                    A+
                  </UiButton>

                  <UiButton size="sm" variant="secondary" @click="cycleTheme">
                    {{ themeCycleLabel }}
                  </UiButton>
                </div>
              </UiFieldRow>
            </UiSection>
          </template>

          <template v-else-if="tab === 'stage'">
            <p class="sv-lead">
              {{ t('theater.settings.stageLead') }}
            </p>

            <UiSection

              :title="t('theater.settings.portraitSectionTitle')"

              :description="t('theater.settings.portraitSectionHelp')"
            >
              <UiFieldRow :label="t('theater.settings.portraitWidth')">
                <div class="theater-range-row">
                  <input

                    type="range"

                    class="theater-range"

                    :min="THEATER_PORTRAIT_LIMITS.width.min"

                    :max="THEATER_PORTRAIT_LIMITS.width.max"

                    step="4"

                    :value="portraitLayout.width"

                    @input="onPortraitWidthInput"
                  >

                  <span class="sv-muted theater-range-val">{{ portraitLayout.width }}px</span>
                </div>
              </UiFieldRow>

              <UiFieldRow :label="t('theater.settings.portraitMaxHeight')">
                <div class="theater-range-row">
                  <input

                    type="range"

                    class="theater-range"

                    :min="THEATER_PORTRAIT_LIMITS.maxHeight.min"

                    :max="THEATER_PORTRAIT_LIMITS.maxHeight.max"

                    step="8"

                    :value="portraitLayout.maxHeight"

                    @input="onPortraitHeightInput"
                  >

                  <span class="sv-muted theater-range-val">{{ portraitLayout.maxHeight }}px</span>
                </div>
              </UiFieldRow>

              <UiFieldRow :label="t('theater.settings.portraitReset')">
                <UiButton size="sm" variant="secondary" @click="onResetPortraitLayout">
                  {{ t('theater.settings.portraitReset') }}
                </UiButton>
              </UiFieldRow>
            </UiSection>

            <UiSection
              :title="t('theater.settings.pokeSectionTitle')"
              :description="t('theater.settings.pokeSectionHelp')"
            >
              <UiFieldRow :label="t('theater.settings.pokeMode')">
                <UiSelect
                  :model-value="pokeMode"
                  @change="onPokeModeChange"
                >
                  <option value="patch">
                    {{ t('theater.settings.pokeModePatch') }}
                  </option>
                  <option value="ripple">
                    {{ t('theater.settings.pokeModeRipple') }}
                  </option>
                </UiSelect>
              </UiFieldRow>

              <UiFieldRow :label="t('theater.settings.variantSwipe')">
                <label class="theater-checkbox-row">
                  <input
                    type="checkbox"
                    :checked="variantSwipe"
                    @change="onVariantSwipeChange"
                  >
                  <span>{{ t('theater.settings.variantSwipeHint') }}</span>
                </label>
              </UiFieldRow>

              <UiFieldRow :label="t('theater.settings.customLeadCast')">
                <UiSelect
                  :model-value="customLeadCast"
                  @change="onCustomLeadChange"
                >
                  <option value="a">
                    {{ t('theater.settings.customLeadA') }}
                  </option>
                  <option value="b">
                    {{ t('theater.settings.customLeadB') }}
                  </option>
                </UiSelect>
              </UiFieldRow>
            </UiSection>
          </template>

          <template v-else-if="tab === 'cast'">
            <TheaterCastPanel
              :active="visible && tab === 'cast'"
              :apply-cast="props.applyCast"
              :apply-default-cast="props.applyDefaultCast"
              :clear-cast-adapt-cache="props.clearCastAdaptCache"
              :re-adapt-current-cast="props.reAdaptCurrentCast"
              :cast-adapt-active="props.castAdaptActive ?? false"
              :cast-adapt-steps="props.castAdaptSteps ?? []"
              :cast-adapt-progress-current="props.castAdaptProgressCurrent ?? 0"
              :cast-adapt-progress-total="props.castAdaptProgressTotal ?? 0"
              :cast-adapt-progress-label="props.castAdaptProgressLabel ?? ''"
              :cast-adapt-waiting-phase="props.castAdaptWaitingPhase ?? 'thinking'"
              :cast-adapt-waiting-seconds="props.castAdaptWaitingSeconds ?? 0"
              :cast-adapt-skeleton-hash="props.castAdaptSkeletonHash ?? ''"
              :cast-adapt-preset-id="props.castAdaptPresetId ?? 'breakfast'"
              :cast-skeleton-ready="props.castSkeletonReady ?? false"
              :cast-adapt-last-issue="props.castAdaptLastIssue ?? null"
              @apply="emit('applyCast', $event)"
              @notify="emit('notify', $event)"
            />
          </template>

          <template v-else-if="tab === 'model'">
            <p class="sv-lead">
              {{ t('theater.settings.modelLead') }}
            </p>

            <UiSection :title="t('modelManager.title')">
              <ModelManagerBody />
            </UiSection>

            <UiSection :title="t('kernel.diagnostics.title')">
              <KernelConnectionSettingsPanel :active="visible && tab === 'model'" />
            </UiSection>
          </template>
        </div>

        <footer class="theater-settings-foot">
          <UiButton size="sm" variant="primary" @click="onBack">
            {{ t('theater.settings.back') }}
          </UiButton>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sv-backdrop {

  position: fixed;

  inset: 0;

  z-index: 10050;

  display: flex;

  align-items: center;

  justify-content: center;

  padding: 16px;

  background: color-mix(in srgb, #000 45%, transparent);

}

.theater-settings-dialog {

  width: min(640px, 100%);

  max-height: min(90vh, 800px);

  overflow: hidden;

  padding: 16px 18px 0;

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

.sv-nav-btn[aria-current='page'] {

  border-color: var(--border-light);

  background: var(--bg-elevated);

  color: var(--text-primary);

}

.sv-body {

  flex: 1;

  min-height: 0;

  overflow: auto;

  padding-bottom: 4px;

}

.sv-lead {

  margin: 0 0 12px;

  font-size: 13px;

  line-height: 1.45;

  color: var(--text-secondary);

}

.sv-muted {

  margin: 0;

  font-size: 12px;

  color: var(--text-secondary);

  line-height: 1.4;

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

.theater-range-row {

  display: flex;

  align-items: center;

  gap: 12px;

  width: 100%;

}

.theater-range {

  flex: 1;

  min-width: 0;

  accent-color: var(--tool-accent, var(--accent));

}

.theater-range-val {

  flex-shrink: 0;

  min-width: 3.5rem;

  text-align: right;

  font-variant-numeric: tabular-nums;

}

.theater-checkbox-row {

  display: flex;

  align-items: flex-start;

  gap: 8px;

  font-size: var(--tool-fs-sm, 12px);

  color: var(--text-secondary);

  cursor: pointer;

}

.theater-checkbox-row input {

  margin-top: 2px;

  accent-color: var(--tool-accent, var(--accent));

}

.theater-settings-foot {

  display: flex;

  justify-content: flex-end;

  margin: 0 -18px;

  padding: 12px 18px 16px;

  border-top: 1px solid var(--border-light);

  background: var(--bg-secondary, var(--tool-chrome-status));

}
</style>
