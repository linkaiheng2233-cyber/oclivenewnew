<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { pluginBridgeInvoke, type PluginUiSlotInfo } from '@oclive/shared/api'
import { useDirectoryPluginSlotEmbed } from '@oclive/shared/composables/useDirectoryPluginSlotEmbed'
import { createPluginFrameBridge } from '@oclive/shared/utils/pluginFrameBridge'
import AsyncPluginVue from './AsyncPluginVue.vue'
import PluginErrorPlaceholder from './PluginErrorPlaceholder.vue'

const props = withDefaults(
  defineProps<{
    /** Official semantic slot name, e.g. `settings.plugins` */
    slotName: string
    /** Bump to refetch bootstrap after changes */
    bootstrapEpoch?: number
    /** Accessible name */
    ariaLabel?: string
  }>(),
  { bootstrapEpoch: 0, ariaLabel: '' },
)

const { t } = useI18n()

const resolvedAriaLabel = computed(
  () => props.ariaLabel?.trim() || t('pluginManager.slotEmbed.defaultAria'),
)

const {
  pluginError,
  slots,
  frameErrors,
  frameErrorDetails,
  reloadNonceFor,
  onFrameError,
  onFrameLoad,
  onVueFailed,
  onVueCompileError,
  retrySlot,
  showIframe,
  showVue,
} = useDirectoryPluginSlotEmbed({
  slot: () => props.slotName,
  bootstrapEpoch: () => props.bootstrapEpoch,
})

const frameBridge = createPluginFrameBridge(pluginBridgeInvoke)
const registeredFrames = new Map<
  string,
  { element: HTMLIFrameElement, unregister: () => void }
>()

function frameKey(slot: PluginUiSlotInfo): string {
  return `${slot.pluginId}:${slot.appearanceId ?? ''}`
}

function bindPluginFrame(slot: PluginUiSlotInfo, value: unknown): void {
  const key = frameKey(slot)
  const current = registeredFrames.get(key)
  if (current?.element === value)
    return
  current?.unregister()
  registeredFrames.delete(key)

  if (!(value instanceof HTMLIFrameElement) || !value.contentWindow)
    return
  registeredFrames.set(key, {
    element: value,
    unregister: frameBridge.register(value.contentWindow, {
      pluginId: slot.pluginId,
      assetRel: slot.entry,
    }),
  })
}

onMounted(() => window.addEventListener('message', frameBridge.handleMessage))
onBeforeUnmount(() => {
  window.removeEventListener('message', frameBridge.handleMessage)
  for (const frame of registeredFrames.values())
    frame.unregister()
  registeredFrames.clear()
  frameBridge.dispose()
})
</script>

<template>
  <div v-if="pluginError" class="pse pse--error" role="status">
    {{ pluginError }}
  </div>
  <div v-else-if="slots.length > 0" class="pse" :aria-label="resolvedAriaLabel">
    <div
      v-for="s in slots"
      :key="`${s.pluginId}:${s.appearanceId ?? ''}`"
      class="pse-slot"
    >
      <AsyncPluginVue
        v-if="showVue(s)"
        class="pse-vue"
        :plugin-id="s.pluginId"
        :vue-component="s.vueComponent!"
        :bridge-asset-rel="s.entry"
        :reload-nonce="reloadNonceFor(s.pluginId)"
        @failed="onVueFailed(s.pluginId)"
        @compile-error="onVueCompileError(s.pluginId, $event)"
      />
      <iframe
        v-if="showIframe(s)"
        :key="`if-${s.pluginId}-${s.appearanceId ?? ''}-${reloadNonceFor(s.pluginId)}`"
        class="pse-frame"
        :src="s.url"
        :title="`plugin ${s.pluginId}`"
        :ref="el => bindPluginFrame(s, el)"
        sandbox="allow-scripts"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onFrameLoad(s.pluginId)"
        @error="onFrameError(s.pluginId)"
      />
      <PluginErrorPlaceholder
        v-if="frameErrors[s.pluginId]"
        class="pse-fail"
        :message="frameErrors[s.pluginId]!"
        :detail="frameErrorDetails[s.pluginId] || undefined"
        :show-fallback="false"
        @retry="retrySlot(s)"
      />
    </div>
  </div>
</template>

<style scoped>
.pse {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}
.pse--error {
  font-size: 12px;
  color: var(--text-secondary);
}
.pse-slot {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}
.pse-frame {
  width: 100%;
  min-height: 120px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.pse-vue {
  width: 100%;
  min-height: 48px;
  display: flex;
  align-items: center;
}
.pse-fail {
  max-width: 100%;
}
</style>
