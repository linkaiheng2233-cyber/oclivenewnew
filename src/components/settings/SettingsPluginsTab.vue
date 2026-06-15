<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import HotkeySettingsSection from '../hotkey/HotkeySettingsSection.vue'
import PluginSettingsPanelSlots from '../PluginSettingsPanelSlots.vue'
import HelpHint from '../shared/HelpHint.vue'
import UiSection from '../ui/UiSection.vue'

defineProps<{
  bootstrapEpoch: number
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
      <PluginSettingsPanelSlots :bootstrap-epoch="bootstrapEpoch" />
    </UiSection>

    <HotkeySettingsSection />
  </form>
</template>
