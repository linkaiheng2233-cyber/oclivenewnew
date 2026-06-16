<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  open: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  submit: [text: string]
  close: []
}>()

const { t } = useI18n()
const inputRef = ref<HTMLInputElement | null>(null)
const draft = ref('')

watch(() => props.open, async (open) => {
  if (open) {
    draft.value = ''
    await nextTick()
    inputRef.value?.focus()
  }
})

function onSubmit() {
  const text = draft.value.trim()
  if (!text || props.disabled)
    return
  emit('submit', text)
  draft.value = ''
  emit('close')
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    onSubmit()
  }
  if (e.key === 'Escape')
    emit('close')
}
</script>

<template>
  <Transition name="custom-fade">
    <div
      v-if="open"
      class="poke-custom"
      role="dialog"
      :aria-label="t('theater.poke.customTitle')"
    >
      <p class="poke-custom__lead">
        {{ t('theater.poke.customLead') }}
      </p>
      <div class="poke-custom__row">
        <input
          ref="inputRef"
          v-model="draft"
          type="text"
          class="poke-custom__input"
          :placeholder="t('theater.poke.customPlaceholder')"
          :disabled="disabled"
          @keydown="onKeydown"
        >
        <button
          type="button"
          class="poke-custom__submit"
          :disabled="disabled || !draft.trim()"
          @click="onSubmit"
        >
          {{ t('theater.poke.customSubmit') }}
        </button>
      </div>
      <button
        type="button"
        class="poke-custom__cancel"
        @click="emit('close')"
      >
        {{ t('theater.poke.customCancel') }}
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.poke-custom {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-2, 8px);
  padding: var(--tool-space-3, 12px) var(--tool-space-4, 16px);
  border-top: 1px solid var(--tool-divider, var(--border-light));
  background: var(--tool-elevated, var(--bg-elevated));
}

.poke-custom__lead {
  margin: 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
}

.poke-custom__row {
  display: flex;
  gap: var(--tool-space-2, 8px);
}

.poke-custom__input {
  flex: 1;
  min-width: 0;
  min-height: var(--tool-row-h, 32px);
  padding: 0 var(--tool-space-3, 12px);
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--radius-btn, 8px);
  background: var(--tool-chrome-editor, var(--bg-primary));
  color: var(--text-primary);
  font-size: var(--tool-fs-md, 13px);
}

.poke-custom__input:focus-visible {
  outline: 2px solid var(--focus-ring-color, var(--tool-accent));
  outline-offset: 2px;
}

.poke-custom__submit {
  min-height: var(--tool-row-h, 32px);
  padding: 0 var(--tool-space-3, 12px);
  border: 1px solid var(--tool-accent);
  border-radius: var(--radius-btn, 8px);
  background: color-mix(in srgb, var(--tool-accent) 12%, var(--tool-elevated));
  color: var(--text-primary);
  font-size: var(--tool-fs-md, 13px);
  cursor: pointer;
  white-space: nowrap;
}

.poke-custom__submit:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.poke-custom__cancel {
  align-self: flex-start;
  padding: 0;
  border: none;
  background: none;
  color: var(--text-secondary);
  font-size: var(--tool-fs-sm, 12px);
  cursor: pointer;
}

.custom-fade-enter-active,
.custom-fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.custom-fade-enter-from,
.custom-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
