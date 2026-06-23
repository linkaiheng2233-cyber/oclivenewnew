<script setup lang="ts">
import { inject, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import HelpHint from '../shared/HelpHint.vue'
import { useDistroUxProfile } from '@oclive/shared/composables/useDistroUxProfile'
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { useRoleStore } from '@oclive/shared/stores/roleStore'

const { t } = useI18n()
const roleStore = useRoleStore()
const shell = inject(MAIN_SHELL_KEY, null)
const { allowModeSwitch, ensureDistroUxProfileLoaded } = useDistroUxProfile()

onMounted(() => {
  void ensureDistroUxProfileLoaded()
})

async function onChange(ev: Event): Promise<void> {
  if (shell?.onInteractionModeChange)
    await shell.onInteractionModeChange(ev)
}
</script>

<template>
  <!-- Story mode only: daily chat discovers immersive via unlock banner; bar is the exit ramp. -->
  <div
    v-if="allowModeSwitch && roleStore.interactionImmersive"
    class="interaction-mode-bar"
    role="group"
    :aria-label="t('app.more.interactionMode')"
  >
    <label class="interaction-mode-bar__label" for="chat-interaction-mode">
      {{ t('app.more.interactionMode') }}
    </label>
    <select
      id="chat-interaction-mode"
      class="interaction-mode-bar__select"
      :value="roleStore.roleInfo.interactionMode"
      @change="onChange"
    >
      <option value="pure_chat">
        {{ t('app.more.interactionPureChat') }}
      </option>
      <option value="immersive">
        {{ t('app.more.interactionImmersive') }}
      </option>
    </select>
    <HelpHint
      :paragraphs="[
        t('app.more.interactionPureChatHint'),
        t('app.more.interactionImmersiveHint'),
      ]"
    />
  </div>
</template>

<style scoped>
.interaction-mode-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.75rem;
  margin: 0.5rem 0.75rem 0;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  background: var(--oc-surface-elevated, rgba(127, 127, 127, 0.08));
  border: 1px solid var(--oc-border-subtle, rgba(127, 127, 127, 0.18));
  font-size: 0.85rem;
}
.interaction-mode-bar__label {
  font-weight: 600;
  color: var(--text-primary);
}
.interaction-mode-bar__select {
  min-width: 7.5rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.85rem;
}
</style>
