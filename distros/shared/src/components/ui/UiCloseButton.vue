<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{ label?: string, disabled?: boolean }>()
const { t } = useI18n()
</script>

<template>
  <button type="button" class="ui-close-button" :disabled="disabled" :aria-label="label ?? t('common.close')" :title="label ?? t('common.close')">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true" focusable="false">
      <path d="m6 6 12 12M18 6 6 18" />
    </svg>
  </button>
</template>

<style scoped>
.ui-close-button { display:inline-flex; align-items:center; justify-content:center; flex-shrink:0; width:34px; height:34px; padding:0; border:1px solid var(--border-light); border-radius:8px; color:var(--text-secondary); background:transparent; cursor:pointer; transition:background var(--ui-motion-fast,120ms) ease-out,border-color var(--ui-motion-fast,120ms) ease-out,transform var(--ui-motion-fast,120ms) ease-out; }
.ui-close-button svg { width:18px; height:18px; pointer-events:none; }
.ui-close-button:hover:not(:disabled) { color:var(--text-primary); background:var(--bg-secondary); border-color:var(--border-focus,var(--accent)); }
.ui-close-button:active:not(:disabled) { transform:translateY(1px); }
.ui-close-button:focus-visible { outline:2px solid var(--accent); outline-offset:2px; }
.ui-close-button:disabled { opacity:.5; cursor:default; }
@media(prefers-reduced-motion:reduce) { .ui-close-button { transition:none; }.ui-close-button:active:not(:disabled) { transform:none; } }
:global(html[data-skin="win98"] .ui-close-button) { width:24px; height:24px; border:2px outset #dfdfdf; border-radius:0; color:#000000; background:#c0c0c0; transition:none; }
:global(html[data-skin="win98"] .ui-close-button svg) { width:12px; height:12px; }
:global(html[data-skin="win98"] .ui-close-button:hover:not(:disabled)) { background:#c0c0c0; border-color:#dfdfdf; }
:global(html[data-skin="win98"] .ui-close-button:active:not(:disabled)) { transform:none; border-style:inset; }
:global(html[data-skin="win98"] .ui-close-button:focus-visible) { outline:1px dotted #000000; outline-offset:-4px; }
</style>
