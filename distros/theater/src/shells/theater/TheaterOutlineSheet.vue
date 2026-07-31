<script setup lang="ts">
import UiButton from '@oclive/shared/components/ui/UiButton.vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { getStoredOutline, setStoredOutline } from '../../composables/theater/useTheaterOutlineMode'

const props = defineProps<{
  open: boolean
  loading?: boolean
  castLabel?: string
}>()

const emit = defineEmits<{
  close: []
  submit: [outline: string]
}>()

const { t } = useI18n()
const draft = ref(getStoredOutline())

watch(
  () => props.open,
  (v) => {
    if (v)
      draft.value = getStoredOutline()
  },
)

const canSubmit = computed(() => draft.value.trim().length >= 8 && !props.loading)

function onSubmit() {
  const text = draft.value.trim()
  if (!text)
    return
  setStoredOutline(text)
  emit('submit', text)
}
</script>

<template>
  <div v-if="open" class="outline-sheet" role="dialog" aria-modal="true" :aria-label="t('theater.outline.title')">
    <div class="outline-sheet__backdrop" @click="emit('close')" />
    <div class="outline-sheet__panel">
      <header class="outline-sheet__head">
        <h2>{{ t('theater.outline.title') }}</h2>
        <p v-if="castLabel" class="outline-sheet__cast">
          {{ castLabel }}
        </p>
        <p class="outline-sheet__lead">
          {{ t('theater.outline.lead') }}
        </p>
      </header>
      <textarea
        v-model="draft"
        class="outline-sheet__input"
        rows="8"
        :placeholder="t('theater.outline.placeholder')"
        :disabled="loading"
      />
      <footer class="outline-sheet__actions">
        <UiButton variant="ghost" :disabled="loading" @click="emit('close')">
          {{ t('theater.outline.cancel') }}
        </UiButton>
        <UiButton variant="primary" :disabled="!canSubmit" :loading="loading" @click="onSubmit">
          {{ t('theater.outline.submit') }}
        </UiButton>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.outline-sheet {
  position: fixed;
  inset: 0;
  z-index: var(--tool-z-overlay, 40);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.outline-sheet__backdrop {
  position: absolute;
  inset: 0;
  background: color-mix(in srgb, var(--bg-primary) 35%, transparent);
}

.outline-sheet__panel {
  position: relative;
  width: min(560px, 100%);
  max-height: 85vh;
  margin: var(--tool-space-4, 16px);
  padding: var(--tool-space-4, 16px);
  border-radius: var(--tool-radius-lg, 12px);
  border: 1px solid var(--tool-divider);
  background: var(--tool-chrome-editor, var(--bg-primary));
  box-shadow: var(--tool-shadow-lg, 0 8px 32px rgba(0, 0, 0, 0.12));
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-3, 12px);
}

.outline-sheet__head h2 {
  margin: 0;
  font-size: var(--tool-text-lg, 1.125rem);
}

.outline-sheet__cast,
.outline-sheet__lead {
  margin: 0.25rem 0 0;
  font-size: var(--tool-text-sm, 0.875rem);
  color: var(--tool-text-muted, var(--text-secondary));
}

.outline-sheet__input {
  width: 100%;
  min-height: 10rem;
  resize: vertical;
  padding: var(--tool-space-3, 12px);
  border-radius: var(--tool-radius-md, 8px);
  border: 1px solid var(--tool-divider);
  font: inherit;
  line-height: 1.5;
}

.outline-sheet__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--tool-space-2, 8px);
}
</style>
