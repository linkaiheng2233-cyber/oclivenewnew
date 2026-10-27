<script setup lang="ts">
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{ error?: unknown, summary?: string, details?: string, dismissible?: boolean }>()
defineEmits<{ dismiss: [] }>()
const { t } = useI18n()
const failure = computed(() => toErrorPresentation(props.error, props.summary))
</script>

<template>
  <section class="error-notice" role="alert">
    <div class="error-notice-heading">
      <p>{{ failure.summary }}</p>
      <button v-if="dismissible" type="button" :aria-label="t('errorUi.dismiss')" @click="$emit('dismiss')">
        ×
      </button>
    </div>
    <slot />
    <details v-if="details || failure.details" class="error-notice-details">
      <summary>{{ t('errorUi.details') }}</summary>
      <pre tabindex="0">{{ details || failure.details }}</pre>
    </details>
  </section>
</template>

<style scoped>
.error-notice { padding: 12px 14px; border: 1px solid var(--error, #b35146); border-radius: 10px; background: var(--bg-elevated, #fffaf5); color: var(--text-primary, #273a39); text-align: start; min-width: 0; }
.error-notice-heading { display: flex; align-items: start; gap: 12px; }
.error-notice-heading p { margin: 0; flex: 1; font-size: 14px; line-height: 1.6; font-weight: 600; overflow-wrap: anywhere; }
.error-notice-heading button { border: 0; background: transparent; color: inherit; font-size: 20px; cursor: pointer; padding: 0 4px; }
.error-notice-details { margin-top: 10px; font-size: 12px; }
.error-notice-details summary { cursor: pointer; }
.error-notice-details pre { margin: 8px 0 0; padding: 10px; border-radius: 6px; background: var(--bg-secondary, #eeeae1); white-space: pre-wrap; overflow-wrap: anywhere; max-height: 240px; overflow: auto; font: 12px/1.6 Consolas, monospace; user-select: text; }
</style>
