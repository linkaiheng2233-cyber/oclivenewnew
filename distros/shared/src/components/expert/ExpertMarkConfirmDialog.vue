<script setup lang="ts">
import { ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'

const props = defineProps<{
  open: boolean
  slotKey: string
  busy?: boolean
}>()

const emit = defineEmits<{
  close: []
  confirm: []
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'open'), dialogRef)

function onBackdrop(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('emcd-backdrop')) {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="open"
    class="emcd-backdrop"
    role="dialog"
    aria-modal="true"
    :aria-label="t('expertConfig.nodeMark.title')"
    @click="onBackdrop"
  >
    <div ref="dialogRef" class="emcd-panel" tabindex="-1">
      <h3 class="emcd-title">
        {{ t("expertConfig.nodeMark.title") }}
      </h3>
      <p class="emcd-body">
        {{ t("expertConfig.nodeMark.body", { slot: slotKey }) }}
      </p>
      <div class="emcd-actions">
        <button type="button" class="emcd-btn" :disabled="busy" @click="emit('close')">
          {{ t("expertConfig.cancel") }}
        </button>
        <button
          type="button"
          class="emcd-btn emcd-btn--primary"
          :disabled="busy"
          @click="emit('confirm')"
        >
          {{ t("expertConfig.nodeMark.confirm") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.emcd-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
}
.emcd-panel {
  width: min(420px, 92vw);
  padding: 16px 18px;
  border-radius: 10px;
  background: var(--bg-elevated, #1e1e24);
  border: 1px solid var(--border-light, #444);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
}
.emcd-title {
  margin: 0 0 8px;
  font-size: 15px;
}
.emcd-body {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.emcd-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.emcd-btn {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light, #444);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
}
.emcd-btn--primary {
  background: var(--accent, #6b9bd1);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
}
.emcd-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
