<script setup lang="ts">
import AsyncPluginVue from "./AsyncPluginVue.vue";
import PluginErrorPlaceholder from "./PluginErrorPlaceholder.vue";
import { useDirectoryPluginSlotEmbed } from "../composables/useDirectoryPluginSlotEmbed";

const props = withDefaults(
  defineProps<{
    /** 官方语义插槽名，如 `settings.plugins` */
    slotName: string;
    /** 变更后重新拉取 bootstrap */
    bootstrapEpoch?: number;
    /** 无障碍名称 */
    ariaLabel?: string;
  }>(),
  { bootstrapEpoch: 0, ariaLabel: "插件嵌入区" },
);

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
});
</script>

<template>
  <div v-if="pluginError" class="pse pse--error" role="status">
    {{ pluginError }}
  </div>
  <div v-else-if="slots.length > 0" class="pse" :aria-label="ariaLabel">
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
