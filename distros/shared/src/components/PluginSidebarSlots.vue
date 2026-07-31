<script setup lang="ts">
import { useDirectoryPluginSlotEmbed } from '@oclive/shared/composables/useDirectoryPluginSlotEmbed'
import { SLOT_SIDEBAR } from '@oclive/shared/stores/pluginStore'
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
  bindPluginFrame,
  framePermissions,
  reloadNonceFor,
  onFrameError,
  onPluginFrameLoad,
  onVueFailed,
  onVueCompileError,
  retrySlot,
  showIframe,
  showVue,
} = useDirectoryPluginSlotEmbed({
  slot: SLOT_SIDEBAR,
  bootstrapEpoch: () => props.bootstrapEpoch,
})
</script>

<template>
  <div v-if="pluginError" class="psb-msg psb-msg--err" role="status">
    {{ pluginError }}
  </div>
  <div v-else-if="slots.length > 0" class="psb-list" :aria-label="t('pluginManager.slotsAria.sidebarList')">
    <div v-for="s in slots" :key="`${s.pluginId}:${s.appearanceId ?? ''}`" class="psb-item">
      <AsyncPluginVue
        v-if="showVue(s)"
        class="psb-vue"
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
        :ref="el => bindPluginFrame(s, el)"
        class="psb-frame"
        :src="s.url"
        :title="`plugin sidebar ${s.pluginId}`"
        :allow="framePermissions(s)"
        sandbox="allow-scripts"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onPluginFrameLoad(s, $event)"
        @error="onFrameError(s.pluginId)"
      />
      <PluginErrorPlaceholder
        v-if="frameErrors[s.pluginId]"
        class="psb-fail"
        :message="frameErrors[s.pluginId]!"
        :detail="frameErrorDetails[s.pluginId] || undefined"
        :show-fallback="false"
        @retry="retrySlot(s)"
      />
    </div>
  </div>
</template>

<style scoped>
.psb-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.psb-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
}
.psb-frame {
  width: 100%;
  min-height: 96px;
  height: 140px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.psb-vue {
  width: 100%;
  min-height: 80px;
}
.psb-msg {
  margin: 0;
  font-size: 12px;
}
.psb-msg--err {
  color: var(--text-danger, #c33);
}
.psb-msg--muted {
  color: var(--text-secondary);
}
</style>
