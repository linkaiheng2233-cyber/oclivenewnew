<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import PluginSettingsPanelSlots from '../PluginSettingsPanelSlots.vue'
import HelpHint from '../shared/HelpHint.vue'
import UiButton from '../ui/UiButton.vue'
import UiSection from '../ui/UiSection.vue'

defineProps<{
  bootstrapEpoch: number
  /** In pure_chat, only embed platform side-channel settings (e.g. voice.asr). */
  platformOnly?: boolean
}>()

defineEmits<{
  'request-general-advanced': []
}>()

const { t } = useI18n()
</script>

<template>
  <form class="sv-body" @submit.prevent>
    <UiSection
      :title="t('settings.pluginsPanelTitle')"
      :description="t('settings.pluginsPanelHint1')"
    >
      <template #extra>
        <HelpHint :paragraphs="[t('settings.pluginsPanelHint1'), t('settings.pluginsPanelHint2')]" />
      </template>
      <PluginSettingsPanelSlots
        :bootstrap-epoch="bootstrapEpoch"
        :platform-only="platformOnly"
      />
    </UiSection>

    <UiSection :title="t('keybindings.title')">
      <p class="sv-muted">
        {{ t("keybindings.movedNotice") }}
      </p>
      <UiButton size="sm" variant="secondary" type="button" @click="$emit('request-general-advanced')">
        {{ t("keybindings.goToGeneralAdvanced") }}
      </UiButton>
    </UiSection>
  </form>
</template>
