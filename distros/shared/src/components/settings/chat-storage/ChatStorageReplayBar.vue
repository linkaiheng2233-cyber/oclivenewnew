<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  active: boolean
  percent: number
}>()

const { t } = useI18n()
</script>

<template>
  <div v-if="active" class="css-replay-bar">
    <div class="css-replay-track">
      <div class="css-replay-fill" :style="{ width: `${percent}%` }" />
    </div>
    <span class="css-muted">{{ t('chatStorage.replayRunning', { percent }) }}</span>
  </div>
</template>

<style scoped>
.css-replay-bar {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.css-replay-track {
  height: 6px;
  border-radius: 3px;
  background: var(--oc-border, #333);
  overflow: hidden;
}
.css-replay-fill {
  height: 100%;
  background: var(--oc-accent, #6cf);
  transition: width 0.3s ease;
}
.css-muted {
  opacity: 0.7;
  font-size: 0.875rem;
}
</style>
