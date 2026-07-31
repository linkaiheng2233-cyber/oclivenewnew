<script setup lang="ts">
import type { TheaterCastConfig } from '../../composables/theater/theaterCastConfig'
import Toast from '@oclive/shared/components/Toast.vue'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useTheaterShell } from '../../composables/useTheaterShell'
import PokeDock from './PokeDock.vue'
import TheaterFooter from './TheaterFooter.vue'
import TheaterHeader from './TheaterHeader.vue'
import TheaterOutlineSheet from './TheaterOutlineSheet.vue'
import TheaterSettingsSheet from './TheaterSettingsSheet.vue'
import TheaterStage from './TheaterStage.vue'
import TheaterThinkChain from './TheaterThinkChain.vue'
import TheaterVariantBackdrop from './TheaterVariantBackdrop.vue'

const {
  castLabel,
  castTier,
  sceneLabelKey,
  activeScenePresetId,
  scenePresets,
  activePokeChips,
  pokeEnabled,
  visibleLines,
  stageState,
  loadError,
  dockDisabled,
  onPoke,
  onCustomTweak,
  setPreviewChip,
  eventHighlightCast,
  variantBackdropOpen,
  variantBReady,
  variantPatchA,
  variantPatchB,
  selectPokeVariant,
  dismissVariantBackdrop,
  thinkingActive,
  thinkingSteps,
  thinkingTitle,
  waitingSeconds,
  waitingPhase,
  castAdaptActive,
  castAdaptSteps,
  castAdaptPassProgress,
  castAdaptProgressLabel,
  castAdaptWaitingSeconds,
  castAdaptWaitingPhase,
  castAdaptSkeletonHash,
  castAdaptPresetId,
  castSkeletonReady,
  castAdaptLastIssue,
  restartScene,
  settingsOpen,
  settingsTab,
  openSettings,
  closeSettings,
  visibleCount,
  displayLines,
  footerSource,
  castInfo,
  applyCastConfig,
  applyDefaultCast,
  clearCastAdaptCache,
  reAdaptCurrentCast,
  switchScenePreset,
  showToast,
  outlineOpen,
  outlineLoading,
  openOutline,
  closeOutline,
  submitOutline,
} = useTheaterShell()

const { toast } = useAppToast()

async function onApplyCast(config: TheaterCastConfig) {
  await applyCastConfig(config)
  closeSettings()
}

async function onApplyDefaultCast() {
  await applyDefaultCast()
  closeSettings()
}
</script>

<template>
  <main class="theater-layout">
    <div v-show="!settingsOpen" class="theater-frame">
      <TheaterHeader
        :cast-label="castLabel"
        :cast-tier="castTier"
        :scene-label-key="sceneLabelKey"
        :active-scene-preset-id="activeScenePresetId"
        :scene-presets="scenePresets"
        @select-scene="switchScenePreset"
        @open-settings="openSettings()"
        @open-outline="openOutline()"
        @restart="restartScene()"
      />

      <TheaterStage
        :lines="visibleLines"
        :state="stageState"
        :load-error="loadError"
        :cast="castInfo"
        :event-highlight-cast="eventHighlightCast"
      >
        <template #variant-backdrop>
          <TheaterVariantBackdrop
            :visible="variantBackdropOpen && variantBReady"
            :patch-a="variantPatchA"
            :patch-b="variantPatchB"
            @select-b="selectPokeVariant(1)"
            @dismiss="dismissVariantBackdrop()"
          />
        </template>
      </TheaterStage>

      <TheaterThinkChain
        :visible="thinkingActive"
        :title="thinkingTitle"
        :steps="thinkingSteps"
        :waiting-phase="waitingPhase"
        :waiting-seconds="waitingSeconds"
      />

      <section class="theater-dock">
        <PokeDock
          v-if="pokeEnabled"
          :chips="activePokeChips"
          :disabled="dockDisabled"
          @poke="onPoke"
          @custom="onCustomTweak"
          @preview="setPreviewChip"
        />
      </section>

      <TheaterFooter
        :source="footerSource"
        :beat="visibleCount"
        :total="displayLines.length"
      />
    </div>

    <TheaterOutlineSheet
      :open="outlineOpen"
      :loading="outlineLoading"
      :cast-label="castLabel"
      @close="closeOutline()"
      @submit="submitOutline"
    />

    <TheaterSettingsSheet
      :visible="settingsOpen"
      :settings-tab="settingsTab"
      :apply-cast="onApplyCast"
      :apply-default-cast="onApplyDefaultCast"
      :clear-cast-adapt-cache="clearCastAdaptCache"
      :re-adapt-current-cast="reAdaptCurrentCast"
      :cast-adapt-active="castAdaptActive"
      :cast-adapt-steps="castAdaptSteps"
      :cast-adapt-progress-current="castAdaptPassProgress?.current ?? 0"
      :cast-adapt-progress-total="castAdaptPassProgress?.total ?? 0"
      :cast-adapt-progress-label="castAdaptProgressLabel"
      :cast-adapt-waiting-phase="castAdaptWaitingPhase"
      :cast-adapt-waiting-seconds="castAdaptWaitingSeconds"
      :cast-adapt-skeleton-hash="castAdaptSkeletonHash"
      :cast-adapt-preset-id="castAdaptPresetId"
      :cast-skeleton-ready="castSkeletonReady"
      :cast-adapt-last-issue="castAdaptLastIssue"
      @update:settings-tab="settingsTab = $event"
      @close="closeSettings()"
      @apply-cast="onApplyCast"
      @notify="(p) => showToast?.(p.type, p.message)"
    />

    <Toast :show="toast.show" :type="toast.type" :message="toast.message" />
  </main>
</template>

<style scoped>
.theater-layout {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  width: 100%;
  background: var(--shell-page-bg, var(--bg-page));
  color: var(--text-primary);
}

.theater-frame {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.theater-dock {
  flex-shrink: 0;
  border-top: 1px solid var(--tool-divider, var(--border-light));
}

.theater-dock__hint {
  margin: 0;
  padding: var(--tool-space-3, 12px) var(--tool-space-4, 16px);
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
  text-align: center;
}
</style>
