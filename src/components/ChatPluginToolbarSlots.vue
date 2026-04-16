<script setup lang="ts">
import AsyncPluginVue from "./AsyncPluginVue.vue";
import PluginErrorPlaceholder from "./PluginErrorPlaceholder.vue";
import { useDirectoryPluginSlotEmbed } from "../composables/useDirectoryPluginSlotEmbed";
import { SLOT_CHAT_TOOLBAR } from "../stores/pluginStore";

const props = withDefaults(
  defineProps<{
    /** 变更后重新拉取 bootstrap（与插件管理面板保存联动） */
    bootstrapEpoch?: number;
  }>(),
  { bootstrapEpoch: 0 },
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
  slot: SLOT_CHAT_TOOLBAR,
  bootstrapEpoch: () => props.bootstrapEpoch,
});
</script>

<template>
  <div v-if="pluginError" class="plugin-toolbar plugin-toolbar--error" role="status">
    {{ pluginError }}
  </div>
  <div
    v-else-if="slots.length > 0"
    class="plugin-toolbar"
    aria-label="插件工具栏"
  >
    <div
      v-for="s in slots"
      :key="s.pluginId"
      class="plugin-toolbar-slot"
    >
      <AsyncPluginVue
        v-if="showVue(s)"
        class="plugin-toolbar-vue"
        :plugin-id="s.pluginId"
        :vue-component="s.vueComponent!"
        :bridge-asset-rel="s.entry"
        :reload-nonce="reloadNonceFor(s.pluginId)"
        @failed="onVueFailed(s.pluginId)"
        @compile-error="onVueCompileError(s.pluginId, $event)"
      />
      <iframe
        v-if="showIframe(s)"
        :key="`if-${s.pluginId}-${reloadNonceFor(s.pluginId)}`"
        class="plugin-toolbar-frame"
        :src="s.url"
        :title="`plugin ${s.pluginId}`"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onFrameLoad(s.pluginId)"
        @error="onFrameError(s.pluginId)"
      />
      <PluginErrorPlaceholder
        v-if="frameErrors[s.pluginId]"
        class="plugin-toolbar-fail"
        :message="frameErrors[s.pluginId]!"
        :detail="frameErrorDetails[s.pluginId] || undefined"
        :show-fallback="false"
        @retry="retrySlot(s)"
      />
    </div>
  </div>
</template>

<style scoped>
.plugin-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  min-height: 0;
  padding: 6px 18px 0;
  background: var(--bg-primary);
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 80%, transparent);
}
.plugin-toolbar-slot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.plugin-toolbar--error {
  font-size: 12px;
  color: var(--text-secondary);
  padding: 4px 18px;
}
.plugin-toolbar-frame {
  flex: 0 1 auto;
  min-width: 120px;
  max-width: min(420px, 100%);
  height: 36px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
  vertical-align: middle;
}
.plugin-toolbar-vue {
  flex: 0 1 auto;
  min-width: 80px;
  max-width: min(420px, 100%);
  min-height: 32px;
  display: flex;
  align-items: center;
}
.plugin-toolbar-fail {
  max-width: min(420px, 100%);
}
</style>
