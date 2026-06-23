<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost'
    size?: 'sm' | 'md'
    disabled?: boolean
    type?: 'button' | 'submit' | 'reset'
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    disabled: false,
    type: 'button',
  },
)
</script>

<template>
  <button
    :type="type"
    class="ui-btn"
    :class="[`ui-btn--${variant}`, `ui-btn--${size}`]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.ui-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--tool-space-2, 8px);
  min-height: var(--tool-row-h, 32px);
  padding: 0 var(--tool-space-3, 12px);
  border-radius: var(--tool-radius, 4px);
  border: 1px solid transparent;
  font-size: var(--tool-fs-md, 13px);
  font-family: var(--font-ui);
  line-height: var(--tool-line, 1.5);
  cursor: pointer;
  transition: var(--tool-transition, var(--control-transition));
  white-space: nowrap;
}

.ui-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ui-btn--primary {
  background: var(--tool-accent, var(--accent));
  border-color: var(--tool-accent, var(--accent));
  color: #fff;
}

.ui-btn--primary:hover:not(:disabled) {
  filter: brightness(1.06);
}

.ui-btn--sm {
  min-height: var(--tool-control-h-sm, 24px);
  padding: 0 var(--tool-space-2, 8px);
  font-size: var(--tool-fs-sm, 12px);
}

.ui-btn--secondary {
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-elevated)));
  border-color: var(--tool-border, var(--border-light));
  color: var(--tool-text, var(--text-primary));
  box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--tool-text) 4%, transparent);
}

.ui-btn--secondary:hover:not(:disabled) {
  background: color-mix(in srgb, var(--tool-bg, var(--bg-secondary)) 55%, var(--tool-elevated, var(--bg-elevated)) 45%);
  border-color: color-mix(in srgb, var(--tool-text) 18%, var(--tool-border, var(--border-light)));
}

.ui-btn--ghost {
  background: transparent;
  color: var(--tool-text-muted, var(--text-secondary));
}

.ui-btn--ghost:hover:not(:disabled) {
  background: color-mix(in srgb, var(--tool-border, var(--border-light)) 35%, transparent);
  color: var(--tool-text, var(--text-primary));
}

.ui-btn:focus {
  outline: none;
}

.ui-btn:focus-visible {
  box-shadow: 0 0 0 1px var(--tool-accent, var(--accent));
}
</style>
