<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { scheduleVoiceExpansionWarm } from '@oclive/shared/composables/useVoiceExpansionWarm'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import PluginSettingsPanelSlots from '../PluginSettingsPanelSlots.vue'

defineProps<{
  bootstrapEpoch: number
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()

const voicePluginOnly = [VOICE_ASR_PLUGIN_ID] as const

onMounted(() => {
  void scheduleVoiceExpansionWarm(id => pluginStore.isPluginDisabled(id))
})
</script>

<template>
  <div class="sv-body sv-body--voice">
    <p class="sv-lead sv-muted sv-body--voice__lead">
      {{ t('settings.voicePanelLead') }}
    </p>
    <PluginSettingsPanelSlots
      class="sv-voice-slots"
      :bootstrap-epoch="bootstrapEpoch"
      :plugin-id-allowlist="voicePluginOnly"
      fill-height
      hide-tabs
    />
  </div>
</template>

<style scoped>
.sv-body--voice {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

.sv-body--voice__lead {
  flex-shrink: 0;
  margin: 0;
}

.sv-voice-slots {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sv-voice-slots :deep(.psp-root) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.sv-voice-slots :deep(.psp-frame-wrap) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sv-voice-slots :deep(.psp-vue) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sv-voice-slots :deep(.voice-asr-settings) {
  flex: 1;
  min-height: 0;
  max-height: none;
  height: auto;
  overflow: hidden;
}

.sv-voice-slots :deep(.voice-asr-settings .panel-body) {
  padding-right: 0.375rem;
}
</style>
