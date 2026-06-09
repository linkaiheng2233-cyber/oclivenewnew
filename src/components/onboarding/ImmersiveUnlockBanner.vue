<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import UiButton from '../ui/UiButton.vue'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  tryStory: []
  dismiss: []
}>()

const { t } = useI18n()
</script>

<template>
  <div v-if="visible" class="immersive-unlock-banner" role="status">
    <p class="immersive-unlock-banner__title">
      {{ t('onboarding.immersiveUnlock.title') }}
    </p>
    <p class="immersive-unlock-banner__body">
      {{ t('onboarding.immersiveUnlock.body') }}
    </p>
    <div class="immersive-unlock-banner__actions">
      <UiButton size="sm" variant="primary" @click="emit('tryStory')">
        {{ t('onboarding.immersiveUnlock.try') }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="emit('dismiss')">
        {{ t('onboarding.immersiveUnlock.later') }}
      </UiButton>
    </div>
  </div>
</template>

<style scoped>
.immersive-unlock-banner {
  margin: 0.5rem 0.75rem 0;
  padding: 0.75rem 1rem;
  border-radius: 0.5rem;
  background: var(--oc-surface-elevated, rgba(127, 127, 127, 0.12));
  border: 1px solid var(--oc-border-subtle, rgba(127, 127, 127, 0.2));
}
.immersive-unlock-banner__title {
  margin: 0 0 0.25rem;
  font-weight: 600;
}
.immersive-unlock-banner__body {
  margin: 0 0 0.75rem;
  font-size: 0.9rem;
  opacity: 0.9;
}
.immersive-unlock-banner__actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}
</style>
