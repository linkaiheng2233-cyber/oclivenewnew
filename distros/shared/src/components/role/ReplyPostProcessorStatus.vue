<script setup lang="ts">
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

withDefaults(
  defineProps<{
    showTitle?: boolean
  }>(),
  { showTitle: true },
)

const { t, te } = useI18n()
const roleStore = useRoleStore()

const postProcessorStatusText = computed(() => {
  const info = roleStore.roleInfo
  if (!info.replyPostProcessorEnabled) {
    return t('roleRuntime.postProcessorOff')
  }
  const backendKey = `roleRuntime.backend${info.replyPostProcessorBackend.charAt(0).toUpperCase()}${info.replyPostProcessorBackend.slice(1)}`
  const backendLabel = te(backendKey) ? t(backendKey) : info.replyPostProcessorBackend
  const profile = info.replyPostProcessorProfile ?? '—'
  return t('roleRuntime.postProcessorOn', { backend: backendLabel, profile })
})
</script>

<template>
  <div class="reply-post-processor-status">
    <div v-if="showTitle" class="rpp-head">
      <span class="rpp-label">{{ t('settings.postProcessorSectionTitle') }}</span>
    </div>
    <p class="rpp-text">
      {{ postProcessorStatusText }}
    </p>
  </div>
</template>

<style scoped>
.rpp-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}
.rpp-label {
  font-weight: 600;
  font-size: 14px;
}
.rpp-text {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
