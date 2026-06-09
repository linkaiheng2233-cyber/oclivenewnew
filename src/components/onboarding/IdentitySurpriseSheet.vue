<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import UiButton from '../ui/UiButton.vue'

defineProps<{
  visible: boolean
  options: Array<{ id: string, name: string }>
}>()

const emit = defineEmits<{
  pick: [id: string]
  keep: []
}>()

const { t } = useI18n()
</script>

<template>
  <div v-if="visible" class="identity-surprise-sheet" role="dialog" aria-live="polite">
    <p class="identity-surprise-sheet__title">
      {{ t('onboarding.identitySurprise.title') }}
    </p>
    <div class="identity-surprise-sheet__actions">
      <UiButton
        v-for="opt in options"
        :key="opt.id"
        size="sm"
        variant="secondary"
        @click="emit('pick', opt.id)"
      >
        {{ opt.name }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="emit('keep')">
        {{ t('onboarding.identitySurprise.keep') }}
      </UiButton>
    </div>
  </div>
</template>

<style scoped>
.identity-surprise-sheet {
  margin: 0.5rem 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 0.5rem;
  background: var(--oc-accent-soft, rgba(80, 120, 200, 0.12));
  border: 1px solid var(--oc-accent-border, rgba(80, 120, 200, 0.35));
}
.identity-surprise-sheet__title {
  margin: 0 0 0.5rem;
  font-weight: 600;
}
.identity-surprise-sheet__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}
</style>
