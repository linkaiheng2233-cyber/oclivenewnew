<script setup lang="ts">
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { redactDiagnosticText } from '@oclive/shared/utils/diagnosticText'
import { useI18n } from 'vue-i18n'
import ErrorNotice from './ErrorNotice.vue'
import UiButton from './ui/UiButton.vue'
import UiCloseButton from './ui/UiCloseButton.vue'

defineProps<{
  show: boolean
  type: 'success' | 'error' | 'warning' | 'info'
  message: string
  details?: string
  recoveryAction?: boolean
}>()
const emit = defineEmits<{ openRecovery: [] }>()
const { dismissToast } = useAppToast()
const { t } = useI18n()

function openRecovery(): void {
  dismissToast()
  emit('openRecovery')
}
</script>

<template>
  <transition name="toast">
    <div v-if="show" class="toast" :class="[type, { 'toast--notice': type === 'error' || details }]">
      <ErrorNotice v-if="type === 'error' || details" :error="type === 'error' ? message : undefined" :summary="details || type !== 'error' ? message : undefined" :details="details" :tone="type" dismissible @dismiss="dismissToast">
        <template v-if="type === 'error' && recoveryAction" #default>
          <UiButton size="sm" @click="openRecovery">
            {{ t('errorUi.openRecovery') }}
          </UiButton>
        </template>
      </ErrorNotice>
      <template v-else>
        <span class="toast-brief-icon" aria-hidden="true">{{ type === 'success' ? '✓' : type === 'warning' ? '!' : 'i' }}</span>
        <span class="toast-message" role="status">{{ redactDiagnosticText(message) }}</span>
        <UiCloseButton :label="t('errorUi.dismiss')" @click="dismissToast" />
      </template>
    </div>
  </transition>
</template>

<style scoped>
.toast {
  position: fixed;
  right: 12px;
  bottom: 12px;
  display:flex;
  align-items:center;
  gap:10px;
  width:max-content;
  max-width:min(440px,calc(100vw - 24px));
  max-height:calc(100dvh - 24px);
  overflow:auto;
  overscroll-behavior:contain;
  color: var(--text-primary);
  padding: 10px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
  backdrop-filter: blur(10px) saturate(108%);
  -webkit-backdrop-filter: blur(10px) saturate(108%);
  z-index: 12000;
  font-size: 13px;
}
.success {
  background: color-mix(in srgb, var(--success) 16%, var(--bg-elevated));
  border-color: color-mix(in srgb, var(--success) 55%, var(--border-light));
  color: var(--text-primary);
}
.error {
  background: color-mix(in srgb, var(--error) 18%, var(--bg-elevated));
  border-color: var(--error);
  color: var(--text-primary);
  font-weight: 500;
  line-height: 1.45;
  box-shadow: inset 3px 0 0 0 var(--error), var(--shadow-md),
    var(--frame-inset-highlight);
}
.warning {
  background: color-mix(in srgb, var(--warning) 14%, var(--bg-elevated));
  border-color: var(--warning);
  color: var(--text-primary);
}
.info {
  background: color-mix(in srgb, var(--info) 14%, var(--bg-elevated));
  border-color: color-mix(in srgb, var(--info) 45%, var(--border-light));
  color: var(--text-primary);
}
.toast--notice { display:block;padding:0;border:0;background:transparent;backdrop-filter:none;-webkit-backdrop-filter:none;box-shadow:var(--shadow-md); }
.toast-message { flex:1;min-width:0;line-height:1.7;overflow-wrap:anywhere; }
.toast-brief-icon { display:grid;place-items:center;flex:0 0 24px;width:24px;height:24px;border:1px solid currentColor;border-radius:50%;font-weight:700; }
.toast-enter-active,
.toast-leave-active {
  transition: opacity var(--ui-motion-panel,160ms) ease-out, transform var(--ui-motion-panel,160ms) ease-out;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(12px);
}
@media(prefers-reduced-motion:reduce) { .toast-enter-active,.toast-leave-active { transition:none; } }
:global(html[data-skin="win98"] .toast-enter-active),
:global(html[data-skin="win98"] .toast-leave-active) { transition:none; }
</style>
