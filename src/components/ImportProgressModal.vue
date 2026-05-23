<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '../composables/useModalFocusRestore'

const props = defineProps<{
  open: boolean
  percent: number
  message: string
  fileIndex?: number | null
  fileTotal?: number | null
  currentFile?: string | null
}>()

const { t } = useI18n()

const cardRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'open'), cardRef)

const fileLine = computed(() => {
  const cur = props.fileIndex
  const tot = props.fileTotal
  if (cur == null || tot == null || tot <= 0)
    return ''
  return t('common.importPackFileProgress', { current: cur, total: tot })
})

const currentFileLine = computed(() => {
  const name = props.currentFile?.trim()
  if (!name)
    return ''
  return t('common.importPackCurrentFile', { name })
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-busy="true"
    >
      <div ref="cardRef" class="modal-card" tabindex="-1" @click.stop>
        <h2 class="title">
          {{ t("common.importPackTitle") }}
        </h2>
        <div class="bar-track" aria-hidden="true">
          <div
            class="bar-fill"
            :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }"
          />
        </div>
        <p v-if="fileLine" class="msg file-line">
          {{ fileLine }}
        </p>
        <p v-if="currentFileLine" class="msg file-name">
          {{ currentFileLine }}
        </p>
        <p class="msg">
          {{ message }}
        </p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.5));
}
.modal-card {
  width: 100%;
  max-width: 400px;
  padding: 20px;
  border-radius: 12px;
  background: var(--bg-panel, #1a1a22);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.title {
  margin: 0 0 14px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}
.bar-track {
  height: 10px;
  border-radius: 999px;
  background: var(--border-light, #333);
  overflow: hidden;
}
.bar-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #5a8fd4, #7ec8e3);
  transition: width 0.12s ease-out;
}
.msg {
  margin: 12px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.file-line {
  margin-top: 10px;
  font-weight: 500;
  color: var(--text-primary);
}
</style>
