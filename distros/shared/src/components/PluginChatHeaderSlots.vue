<script setup lang="ts">
import { useDirectoryPluginSlotEmbed } from '@oclive/shared/composables/useDirectoryPluginSlotEmbed'
import { SLOT_CHAT_HEADER } from '@oclive/shared/stores/pluginStore'
import { useI18n } from 'vue-i18n'
import AsyncPluginVue from './AsyncPluginVue.vue'
import PluginErrorPlaceholder from './PluginErrorPlaceholder.vue'

const props = withDefaults(
  defineProps<{
    /** Bump to refetch bootstrap after changes (synced with plugin manager save) */
    bootstrapEpoch?: number
  }>(),
  { bootstrapEpoch: 0 },
)

const { t } = useI18n()

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
  slot: SLOT_CHAT_HEADER,
  bootstrapEpoch: () => props.bootstrapEpoch,
})
</script>

<template>
  <div v-if="pluginError" class="pch-msg pch-msg--err" role="status">
    {{ pluginError }}
  </div>
  <div
    v-else-if="slots.length > 0"
    class="pch-strip"
    :aria-label="t('pluginManager.slotsAria.chatHeaderStrip')"
  >
    <div
      v-for="s in slots"
      :key="`${s.pluginId}:${s.appearanceId ?? ''}`"
      class="pch-slot"
    >
      <AsyncPluginVue
        v-if="showVue(s)"
        class="pch-vue"
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
        class="pch-frame"
        :src="s.url"
        :title="`plugin chat.header ${s.pluginId}`"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onFrameLoad(s.pluginId)"
        @error="onFrameError(s.pluginId)"
      />
      <PluginErrorPlaceholder
        v-if="frameErrors[s.pluginId]"
        class="pch-fail"
        :message="frameErrors[s.pluginId]!"
        :detail="frameErrorDetails[s.pluginId] || undefined"
        :show-fallback="false"
        @retry="retrySlot(s)"
      />
    </div>
  </div>
</template>

<style scoped>
.pch-strip {
  display: flex;
  flex-wrap: wrap;
  align-items: stretch;
  gap: 8px;
  width: 100%;
  min-height: 0;
  padding: 6px 18px 4px;
  background: var(--bg-primary);
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 80%, transparent);
}
.pch-frame {
  flex: 1 1 auto;
  min-width: 120px;
  min-height: 44px;
  height: 56px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.pch-vue {
  flex: 1 1 auto;
  min-width: 120px;
  min-height: 44px;
}
.pch-msg {
  font-size: 12px;
}
.pch-msg--err {
  color: var(--text-danger, #c33);
}
.pch-msg--muted {
  color: var(--text-secondary);
}
.pch-slot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}
.pch-fail {
  max-width: min(420px, 100%);
}
</style>
