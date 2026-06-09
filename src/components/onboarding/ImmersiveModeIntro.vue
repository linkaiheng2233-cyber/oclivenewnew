<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '../../composables/useModalFocusRestore'
import UiButton from '../ui/UiButton.vue'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  dismiss: []
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
const locallyDismissed = ref(false)

const open = computed(() => props.visible && !locallyDismissed.value)

watch(
  () => props.visible,
  (next) => {
    if (next)
      locallyDismissed.value = false
  },
)

useModalFocusRestore(open, dialogRef)

function dismiss(): void {
  locallyDismissed.value = true
  emit('dismiss')
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="immersive-intro-overlay"
      role="dialog"
      aria-modal="true"
      :aria-label="t('onboarding.immersiveIntro.title')"
      @click.self="dismiss"
      @keydown.escape.stop="dismiss"
    >
      <div
        ref="dialogRef"
        class="immersive-intro-card"
        tabindex="-1"
        @click.stop
      >
        <h3 class="immersive-intro-card__title">
          {{ t('onboarding.immersiveIntro.title') }}
        </h3>
        <p class="immersive-intro-card__body">
          {{ t('onboarding.immersiveIntro.body') }}
        </p>
        <UiButton size="sm" variant="primary" type="button" @click="dismiss">
          {{ t('onboarding.immersiveIntro.ok') }}
        </UiButton>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.immersive-intro-overlay {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(0, 0, 0, 0.35);
  pointer-events: auto;
}
.immersive-intro-card {
  position: relative;
  z-index: 1;
  max-width: 22rem;
  padding: 1.25rem 1.5rem;
  border-radius: 0.75rem;
  background: var(--oc-surface, var(--bg-primary, #fff));
  color: var(--oc-text, var(--text-primary, #111));
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  pointer-events: auto;
}
.immersive-intro-card__title {
  margin: 0 0 0.5rem;
  font-size: 1.05rem;
}
.immersive-intro-card__body {
  margin: 0 0 1rem;
  font-size: 0.9rem;
  line-height: 1.5;
}
</style>
