<script setup lang="ts">
import { useDirectoryPluginSlotEmbed } from '@oclive/shared/composables/useDirectoryPluginSlotEmbed'
import { SLOT_ROLE_DETAIL } from '@oclive/shared/stores/pluginStore'
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
  slot: SLOT_ROLE_DETAIL,
  bootstrapEpoch: () => props.bootstrapEpoch,
})
</script>

<template>
  <div v-if="pluginError" class="prd-msg prd-msg--err" role="status">
    {{ pluginError }}
  </div>
  <div v-else-if="slots.length > 0" class="prd-list" :aria-label="t('pluginManager.slotsAria.roleDetailList')">
    <div v-for="s in slots" :key="`${s.pluginId}:${s.appearanceId ?? ''}`" class="prd-item">
      <AsyncPluginVue
        v-if="showVue(s)"
        class="prd-vue"
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
        class="prd-frame"
        :src="s.url"
        :title="`plugin role.detail ${s.pluginId}`"
        :allow="framePermissions(s)"
        sandbox="allow-scripts"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onPluginFrameLoad(s, $event)"
        @error="onFrameError(s.pluginId)"
      />
      <PluginErrorPlaceholder
        v-if="frameErrors[s.pluginId]"
        class="prd-fail"
        :message="frameErrors[s.pluginId]!"
        :detail="frameErrorDetails[s.pluginId] || undefined"
        :show-fallback="false"
        @retry="retrySlot(s)"
      />
    </div>
  </div>
</template>

<style scoped>
.prd-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.prd-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
}
.prd-frame {
  width: 100%;
  min-height: 120px;
  height: 180px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.prd-vue {
  width: 100%;
  min-height: 100px;
}
.prd-msg {
  margin: 0;
  font-size: 12px;
}
.prd-msg--err {
  color: var(--text-danger, #c33);
}
.prd-msg--muted {
  color: var(--text-secondary);
}
</style>
