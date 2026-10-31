<script setup lang="ts">
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { redactDiagnosticText } from '@oclive/shared/utils/diagnosticText'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import UiButton from './ui/UiButton.vue'
import UiCloseButton from './ui/UiCloseButton.vue'

const props = withDefaults(defineProps<{
  error?: unknown
  summary?: string
  details?: string
  dismissible?: boolean
  tone?: 'error' | 'warning' | 'info' | 'success'
}>(), { tone: 'error' })
defineEmits<{ dismiss: [] }>()
const { t } = useI18n()
const failure = computed(() => toErrorPresentation(props.error, props.summary))
const diagnosticText = computed(() => redactDiagnosticText(props.details || failure.value.details))
const copyText = computed(() => [failure.value.summary, diagnosticText.value].filter(Boolean).join('\n\n'))
const copyState = ref<'idle' | 'copying' | 'copied' | 'failed'>('idle')
let copyGeneration = 0
watch(copyText, () => {
  copyGeneration += 1
  copyState.value = 'idle'
})
onBeforeUnmount(() => {
  copyGeneration += 1
})

async function copyDetails(): Promise<void> {
  if (copyState.value === 'copying')
    return
  const attempt = ++copyGeneration
  copyState.value = 'copying'
  try {
    await navigator.clipboard.writeText(copyText.value)
    if (attempt === copyGeneration)
      copyState.value = 'copied'
  }
  catch {
    if (attempt === copyGeneration)
      copyState.value = 'failed'
  }
}
</script>

<template>
  <section class="error-notice" :class="`error-notice--${tone}`" :role="tone === 'error' ? 'alert' : 'status'">
    <div class="error-notice-heading">
      <span class="error-notice-symbol" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <template v-if="tone === 'success'"><circle cx="12" cy="12" r="9" /><path d="m8 12 3 3 5-6" /></template>
          <template v-else-if="tone === 'warning'"><path d="m12 3 10 18H2L12 3Z M12 9v5M12 17h.01" /></template>
          <template v-else><circle cx="12" cy="12" r="9" /><path d="M12 7v6M12 17h.01" /></template>
        </svg>
      </span>
      <div class="error-notice-copy">
        <span class="error-notice-label">{{ t(`errorUi.tone.${tone}`) }}</span>
        <p>{{ failure.summary }}</p>
      </div>
      <UiCloseButton v-if="dismissible" :label="t('errorUi.dismiss')" @click="$emit('dismiss')" />
    </div>
    <div v-if="$slots.default" class="error-notice-actions">
      <slot />
    </div>
    <details v-if="diagnosticText" :key="copyText" class="error-notice-details">
      <summary>{{ t('errorUi.details') }}</summary>
      <div class="error-notice-detail-tools">
        <span>{{ t('errorUi.detailsHint') }}</span>
        <UiButton size="sm" :disabled="copyState === 'copying'" @click="copyDetails">
          {{ t(copyState === 'copying' ? 'errorUi.copying' : 'errorUi.copy') }}
        </UiButton>
      </div>
      <p v-if="copyState === 'copied' || copyState === 'failed'" class="error-notice-copy-status" role="status">
        {{ t(copyState === 'copied' ? 'errorUi.copied' : 'errorUi.copyFailed') }}
      </p>
      <pre tabindex="0" :aria-label="t('errorUi.details')">{{ diagnosticText }}</pre>
    </details>
  </section>
</template>

<style scoped>
.error-notice { --notice-color:var(--error,#b35146); padding:16px; border:1px solid color-mix(in srgb,var(--notice-color) 45%,var(--border-light)); border-inline-start:3px solid var(--notice-color); border-radius:10px; background:var(--bg-elevated,#fffaf5); color:var(--text-primary,#273a39); text-align:start; min-width:0; }
.error-notice--warning { --notice-color:var(--fluent-warning-text,#66552e); }
.error-notice--info { --notice-color:var(--accent,#4a6b62); }
.error-notice--success { --notice-color:var(--success,#3f6b4d); }
.error-notice-heading { display:flex; align-items:flex-start; gap:11px; }
.error-notice-symbol { display:grid;place-items:center;flex:0 0 30px;width:30px;height:30px;border-radius:9px;color:var(--notice-color);background:color-mix(in srgb,var(--notice-color) 10%,transparent); }
.error-notice-symbol svg { width:20px;height:20px; }
.error-notice-copy { flex:1;min-width:0; }
.error-notice-label { display:block;margin-bottom:5px;color:var(--notice-color);font-size:11px;font-weight:650;line-height:1.4; }
.error-notice-heading p { margin:0;font-size:14px;line-height:1.75;font-weight:550;overflow-wrap:anywhere; }
.error-notice-actions { display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin-top:12px; }
.error-notice-details { margin-top:14px;padding-top:10px;border-top:1px solid var(--border-light);font-size:12px; }
.error-notice-details summary { cursor:pointer;color:var(--text-secondary);line-height:1.6;padding:4px 0; }
.error-notice-detail-tools { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px;margin-top:10px;color:var(--text-secondary);font-size:11px;line-height:1.6; }
.error-notice-copy-status { margin:8px 0;color:var(--text-secondary);font-size:12px;line-height:1.6; }
.error-notice-details pre { margin:10px 0 0;padding:12px;border:1px solid var(--border-light);border-radius:6px;background:var(--bg-secondary,#eeeae1);white-space:pre-wrap;overflow-wrap:anywhere;max-height:220px;overflow:auto;font:12px/1.7 Consolas,monospace;user-select:text; }
.error-notice :is(summary,pre):focus-visible { outline:2px solid var(--accent);outline-offset:3px; }
@media(max-width:480px) { .error-notice { padding:13px; }.error-notice-heading { gap:8px; }.error-notice-symbol { flex-basis:26px;width:26px;height:26px; } }
</style>

<style>
@import '@oclive/shared/styles/win98/component-recovery.css';
</style>
