<script setup lang="ts">
import AsyncPluginVue from "./AsyncPluginVue.vue";
import PluginErrorPlaceholder from "./PluginErrorPlaceholder.vue";
import { useDirectoryPluginSlotEmbed } from "../composables/useDirectoryPluginSlotEmbed";
import { SLOT_SIDEBAR } from "../stores/pluginStore";

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
  slot: SLOT_SIDEBAR,
  bootstrapEpoch: () => props.bootstrapEpoch,
});
</script>

<template>
  <div v-if="pluginError" class="psb-msg psb-msg--err" role="status">{{ pluginError }}</div>
  <div v-else-if="slots.length > 0" class="psb-list" aria-label="侧边栏插件插槽">
    <div v-for="s in slots" :key="s.pluginId" class="psb-item">
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
        :key="`if-${s.pluginId}-${reloadNonceFor(s.pluginId)}`"
        class="psb-frame"
        :src="s.url"
        :title="`plugin sidebar ${s.pluginId}`"
        loading="lazy"
        referrerpolicy="no-referrer"
        @load="onFrameLoad(s.pluginId)"
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
