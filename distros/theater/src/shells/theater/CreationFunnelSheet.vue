<script setup lang="ts">
import UiButton from '@oclive/shared/components/ui/UiButton.vue'

import { onMounted, onUnmounted } from 'vue'

import { useI18n } from 'vue-i18n'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{

  create: []

  dismiss: []

}>()

const { t } = useI18n()

function onBackdrop() {
  emit('dismiss')
}

function onKeydown(e: KeyboardEvent) {
  if (!props.visible)
    return
  if (e.key === 'Escape')
    emit('dismiss')
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <Teleport to="body">
    <div

      v-if="visible"

      class="creation-funnel-backdrop"

      role="presentation"

      @click.self="onBackdrop"
    >
      <div

        class="creation-funnel-sheet"

        role="dialog"

        aria-live="polite"

        :aria-label="t('theater.funnel.title')"

        @click.stop
      >
        <p class="creation-funnel-sheet__title">
          {{ t('theater.funnel.title') }}
        </p>

        <div class="creation-funnel-sheet__actions">
          <UiButton size="sm" variant="secondary" @click="emit('create')">
            {{ t('theater.funnel.create') }}
          </UiButton>

          <UiButton size="sm" variant="ghost" @click="emit('dismiss')">
            {{ t('theater.funnel.keep') }}
          </UiButton>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.creation-funnel-backdrop {

  position: fixed;

  inset: 0;

  z-index: 10020;

  display: flex;

  align-items: flex-end;

  justify-content: center;

  padding: 0 0 calc(var(--tool-statusbar-h, 24px) + 16px);

  background: color-mix(in srgb, #000 28%, transparent);

}

.creation-funnel-sheet {

  width: min(92vw, 420px);

  margin: 0;

  padding: var(--tool-space-4, 16px);

  border-radius: var(--tool-radius-lg, 12px);

  background: var(--dialog-panel-bg, var(--tool-elevated));

  border: 1px solid color-mix(in srgb, var(--tool-accent) 35%, var(--tool-border));

  box-shadow: 0 8px 32px color-mix(in srgb, #000 18%, transparent);

}

.creation-funnel-sheet__title {

  margin: 0 0 var(--tool-space-3, 12px);

  font-weight: 600;

  font-size: var(--tool-fs-md, 13px);

  line-height: var(--tool-line, 1.5);

}

.creation-funnel-sheet__actions {

  display: flex;

  flex-wrap: wrap;

  gap: var(--tool-space-2, 8px);

}
</style>
