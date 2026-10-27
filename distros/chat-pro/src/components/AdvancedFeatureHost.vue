<script setup lang="ts">
import type { useMainShell } from '../composables/useMainShell'
import HotkeyHost from '@oclive/shared/components/hotkey/HotkeyHost.vue'
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { usePluginEvents } from '@oclive/shared/composables/usePluginEvents'
import { useVoiceAutoTts } from '@oclive/shared/composables/useVoiceAutoTts'
import { resetVoiceExpansionWarmSchedule, startVoiceExpansionWarmOnStartup } from '@oclive/shared/composables/useVoiceExpansionWarm'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { inject, onBeforeUnmount, onMounted } from 'vue'

// The shell mounts this optional host only after the user enables advanced tools.
const shell = inject(MAIN_SHELL_KEY) as ReturnType<typeof useMainShell> | undefined
if (!shell)
  throw new Error('AdvancedFeatureHost requires MAIN_SHELL_KEY provider')

usePluginEvents({
  showToast: shell.showToast,
  onQuickActionTravel: shell.onPluginQuickActionTravel,
  onPureChatMode: shell.resetPureChatSceneUi,
  onVoiceAsrSubmit: ({ text, mode }) => {
    if (!shell.uiStore.advancedFeaturesEnabled || shell.roleStore.minimalRoleActive || !text?.trim())
      return
    if (mode === 'fill') {
      hostEventBus.emit('chat:set_input_draft', { text: text.trim() })
      return
    }
    if (!shell.chatListLoading.value)
      void shell.onSend({ content: text.trim() })
  },
})
useVoiceAutoTts({ showToast: shell.showToast })
onMounted(() => {
  startVoiceExpansionWarmOnStartup(id => shell.pluginStore.isPluginDisabled(id))
  // API errors already use the shared reporter; keep them out of unhandled promises.
  void shell.debugStore.loadDebugData().catch(() => {})
})
onBeforeUnmount(resetVoiceExpansionWarmSchedule)
</script>

<template>
  <HotkeyHost />
</template>
