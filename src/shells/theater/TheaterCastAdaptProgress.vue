<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  active: boolean
  steps: string[]
  progressCurrent?: number
  progressTotal?: number
  progressLabel?: string
  waitingPhase: 'thinking' | 'model'
  waitingSeconds: number
}>()

const { t } = useI18n()

const showProgress = computed(() =>
  (props.progressTotal ?? 0) > 0 && (props.progressCurrent ?? 0) > 0,
)

const progressPercent = computed(() => {
  if (!showProgress.value || !props.progressTotal)
    return 0
  return Math.round(((props.progressCurrent ?? 0) / props.progressTotal) * 100)
})

</script>

<template>
  <Transition name="think-fade">
    <div
      v-if="active"
      class="think-chain cast-adapt-progress"
      role="status"
      aria-live="polite"
    >
      <div class="think-chain__head">
        <span class="think-chain__dot" aria-hidden="true" />
        <span class="think-chain__dot" aria-hidden="true" />
        <span class="think-chain__dot" aria-hidden="true" />
        <span class="think-chain__title">{{ t('theater.cast.adaptProgressTitle') }}</span>
      </div>
      <div v-if="showProgress" class="think-chain__progress">
        <div class="think-chain__progress-track" aria-hidden="true">
          <div
            class="think-chain__progress-fill"
            :style="{ width: `${progressPercent}%` }"
          />
        </div>
        <p class="think-chain__progress-label">
          {{ progressLabel || t('theater.think.castPass.progressFallback', { current: progressCurrent, total: progressTotal }) }}
        </p>
      </div>
      <ul class="think-chain__steps">
        <li
          v-for="(step, i) in steps"
          :key="i"
          class="think-chain__step"
        >
          {{ step }}
        </li>
      </ul>
      <p
        v-if="waitingPhase === 'model'"
        class="think-chain__model-wait"
      >
        {{ t('theater.think.waitingModel', { seconds: waitingSeconds }) }}
      </p>
      <p
        v-if="waitingPhase !== 'model'"
        class="think-chain__hint"
      >
        {{ t('theater.cast.adaptInSettingsHint') }}
      </p>
    </div>
  </Transition>
</template>

<style scoped>
.cast-adapt-progress {
  margin: 0 0 12px;
  width: 100%;
  border: 1px solid var(--border-light);
  border-radius: var(--tool-radius-lg, 12px);
  background: color-mix(in srgb, var(--tool-accent) 8%, var(--tool-elevated, var(--bg-elevated)));
}

.think-chain {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-2, 8px);
  padding: var(--tool-space-3, 12px) var(--tool-space-4, 16px);
}

.think-chain__head {
  display: flex;
  align-items: center;
  gap: 6px;
}

.think-chain__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--tool-accent);
  animation: thinkPulse 1s ease-in-out infinite;
}

.think-chain__dot:nth-child(2) {
  animation-delay: 0.15s;
}

.think-chain__dot:nth-child(3) {
  animation-delay: 0.3s;
}

.think-chain__title {
  margin-left: 4px;
  font-size: var(--tool-fs-md, 13px);
  font-weight: 600;
  color: var(--text-primary);
}

.think-chain__progress {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.think-chain__progress-track {
  height: 4px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--tool-accent) 18%, transparent);
  overflow: hidden;
}

.think-chain__progress-fill {
  height: 100%;
  border-radius: inherit;
  background: var(--tool-accent);
  transition: width 0.35s ease;
}

.think-chain__progress-label {
  margin: 0;
  font-size: 11px;
  color: var(--text-secondary);
}

.think-chain__steps {
  list-style: none;
  margin: 0;
  padding: 0 0 0 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.think-chain__step {
  position: relative;
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.5;
  color: var(--text-secondary);
  animation: thinkStepIn 0.28s ease both;
}

.think-chain__step::before {
  content: '·';
  position: absolute;
  left: -12px;
  color: var(--tool-accent);
}

.think-chain__model-wait {
  margin: 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
}

.think-chain__hint {
  margin: 0;
  font-size: 11px;
  color: var(--text-tertiary, var(--text-secondary));
  opacity: 0.85;
}

.think-fade-enter-active,
.think-fade-leave-active {
  transition: opacity 0.24s ease, transform 0.24s ease;
}

.think-fade-enter-from,
.think-fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

@keyframes thinkPulse {
  0%,
  80%,
  100% {
    opacity: 0.35;
    transform: scale(0.85);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes thinkStepIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .think-chain__dot,
  .think-chain__step,
  .think-fade-enter-active,
  .think-fade-leave-active {
    animation: none;
    transition: none;
  }
}
</style>
