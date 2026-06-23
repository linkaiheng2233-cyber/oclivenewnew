<script setup lang="ts">
defineProps<{
  emoji: string
  label: string
  disabled?: boolean
  variant?: 'default' | 'custom'
}>()

const emit = defineEmits<{
  click: []
  preview: []
  previewEnd: []
}>()
</script>

<template>
  <button
    type="button"
    class="poke-chip"
    :class="{ 'poke-chip--custom': variant === 'custom' }"
    :disabled="disabled"
    :aria-label="label"
    @click="emit('click')"
    @mouseenter="emit('preview')"
    @mouseleave="emit('previewEnd')"
    @focus="emit('preview')"
    @blur="emit('previewEnd')"
  >
    <span class="poke-chip__emoji" aria-hidden="true">{{ emoji }}</span>
    <span class="poke-chip__label">{{ label }}</span>
  </button>
</template>

<style scoped>
.poke-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--tool-space-2, 8px);
  min-height: var(--tool-row-h, 32px);
  padding: 0 var(--tool-space-3, 12px);
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--radius-btn, 8px);
  background: var(--tool-elevated, var(--bg-elevated));
  color: var(--text-primary);
  font-size: var(--tool-fs-md, 13px);
  white-space: nowrap;
  cursor: pointer;
  transition: var(--tool-transition, var(--control-transition));
}

.poke-chip:hover:not(:disabled) {
  border-color: var(--tool-accent);
  background: color-mix(in srgb, var(--tool-accent) 8%, var(--tool-elevated));
}

.poke-chip:focus-visible {
  outline: 2px solid var(--focus-ring-color, var(--tool-accent));
  outline-offset: 2px;
}

.poke-chip:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.poke-chip--custom {
  border-style: dashed;
}

.poke-chip__emoji {
  font-size: 1.1em;
  line-height: 1;
}
</style>
