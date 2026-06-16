<script setup lang="ts">
import type { TheaterCastConfig } from '../../composables/theater/theaterCastConfig'
import Toast from '../../components/Toast.vue'
import { useAppToast } from '../../composables/useAppToast'
import { useTheaterShell } from '../../composables/useTheaterShell'
import PokeDock from './PokeDock.vue'
import TheaterFooter from './TheaterFooter.vue'
import TheaterHeader from './TheaterHeader.vue'
import TheaterSettingsSheet from './TheaterSettingsSheet.vue'
import TheaterStage from './TheaterStage.vue'
import TheaterThinkChain from './TheaterThinkChain.vue'

const {
  castLabel,
  castTier,
  sceneLabelKey,
  visibleLines,
  stageState,
  loadError,
  dockDisabled,
  onPoke,
  onCustomTweak,
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
  castAdaptSceneId,
  castSkeletonReady,
  castAdaptLastIssue,
  restartScene,
  settingsOpen,
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
  showToast,
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
        @open-settings="openSettings()"
        @restart="restartScene()"
      />

      <TheaterStage
        :lines="visibleLines"
        :state="stageState"
        :load-error="loadError"
        :cast="castInfo"
      />

      <TheaterThinkChain
        :visible="thinkingActive"
        :title="thinkingTitle"
        :steps="thinkingSteps"
        :waiting-phase="waitingPhase"
        :waiting-seconds="waitingSeconds"
      />

      <section class="theater-dock">
        <PokeDock
          :disabled="dockDisabled"
          @poke="onPoke"
          @custom="onCustomTweak"
        />
      </section>

      <TheaterFooter
        :source="footerSource"
        :beat="visibleCount"
        :total="displayLines.length"
      />
    </div>

    <TheaterSettingsSheet
      :visible="settingsOpen"
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
      :cast-adapt-scene-id="castAdaptSceneId"
      :cast-skeleton-ready="castSkeletonReady"
      :cast-adapt-last-issue="castAdaptLastIssue"
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
</style>
