<script setup lang="ts">
import { ref, toRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '../../composables/useModalFocusRestore'

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
const cancelBtnRef = ref<HTMLButtonElement | null>(null)
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'open'), dialogRef, { primary: cancelBtnRef })

watch(
  () => props.open,
  (v) => {
    if (!v)
      return
  },
)

function onBackdrop(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('arsd-backdrop')) {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="open"
    class="arsd-backdrop"
    role="dialog"
    aria-modal="true"
    :aria-label="t('pluginWorkbench.graph.removeSlotDialogTitle')"
    @click="onBackdrop"
  >
    <div ref="dialogRef" class="arsd-panel" tabindex="-1" @click.stop>
      <h3 class="arsd-title">
        {{ t("pluginWorkbench.graph.removeSlotDialogTitle") }}
      </h3>
      <p class="arsd-body">
        {{ t("pluginWorkbench.graph.removeSlotConfirm", { key: slotKey }) }}
      </p>
      <div class="arsd-actions">
        <button
          ref="cancelBtnRef"
          type="button"
          class="arsd-btn"
          :disabled="busy"
          @click="emit('close')"
        >
          {{ t("pluginWorkbench.graph.removeSlotCancel") }}
        </button>
        <button
          type="button"
          class="arsd-btn arsd-btn--danger"
          :disabled="busy"
          @click="emit('confirm')"
        >
          {{ t("pluginWorkbench.graph.removeSlotConfirmBtn") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.arsd-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 40%, transparent);
}
.arsd-panel {
  width: min(420px, 100%);
  padding: 16px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.arsd-title {
  margin: 0 0 10px;
  font-size: 16px;
}
.arsd-body {
  margin: 0 0 14px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.arsd-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.arsd-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.arsd-btn--danger {
  border-color: color-mix(in srgb, #dc2626 40%, var(--border-light));
  color: #b91c1c;
}
</style>
