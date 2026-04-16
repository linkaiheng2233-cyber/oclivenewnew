<script setup lang="ts">
import { ref, watch } from "vue";
import AsyncPluginVue from "./AsyncPluginVue.vue";
import PluginErrorPlaceholder from "./PluginErrorPlaceholder.vue";
import { useDirectoryPluginSlotEmbed } from "../composables/useDirectoryPluginSlotEmbed";
import { SLOT_SETTINGS_PANEL } from "../stores/pluginStore";

const props = withDefaults(
  defineProps<{
    /** 变更后重新拉取 bootstrap（与插件管理面板保存联动） */
    bootstrapEpoch?: number;
  }>(),
  { bootstrapEpoch: 0 },
);

const {
  pluginError,
  slots: panelSlots,
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
  slot: SLOT_SETTINGS_PANEL,
  bootstrapEpoch: () => props.bootstrapEpoch,
});

const activeTab = ref(0);

watch(panelSlots, (list) => {
  if (activeTab.value >= list.length) {
    activeTab.value = 0;
  }
});
</script>

<template>
  <div class="psp-root">
    <div v-if="pluginError" class="psp-msg psp-msg--err" role="status">
      {{ pluginError }}
    </div>
    <template v-else-if="panelSlots.length > 0">
      <div class="psp-tabs" role="tablist" aria-label="插件设置页">
        <button
          v-for="(s, i) in panelSlots"
          :key="s.pluginId"
          type="button"
          role="tab"
          class="psp-tab"
          :aria-selected="activeTab === i"
          @click="activeTab = i"
        >
          {{ s.pluginId }}
        </button>
      </div>
      <div
        v-for="(s, i) in panelSlots"
        v-show="activeTab === i"
        :key="`frame-${s.pluginId}`"
        class="psp-frame-wrap"
        role="tabpanel"
      >
        <AsyncPluginVue
          v-if="showVue(s)"
          class="psp-vue"
          :plugin-id="s.pluginId"
          :vue-component="s.vueComponent!"
          :bridge-asset-rel="s.entry"
          :reload-nonce="reloadNonceFor(s.pluginId)"
          skeleton-variant="block"
          @failed="onVueFailed(s.pluginId)"
          @compile-error="onVueCompileError(s.pluginId, $event)"
        />
        <iframe
          v-if="showIframe(s)"
          :key="`if-${s.pluginId}-${reloadNonceFor(s.pluginId)}`"
          class="psp-frame"
          :src="s.url"
          :title="`plugin settings ${s.pluginId}`"
          loading="lazy"
          referrerpolicy="no-referrer"
          @load="onFrameLoad(s.pluginId)"
          @error="onFrameError(s.pluginId)"
        />
        <PluginErrorPlaceholder
          v-if="frameErrors[s.pluginId]"
          class="psp-fail"
          :message="frameErrors[s.pluginId]!"
          :detail="frameErrorDetails[s.pluginId] || undefined"
          :show-fallback="false"
          @retry="retrySlot(s)"
        />
      </div>
    </template>
    <p v-else class="psp-msg psp-msg--muted">暂无声明 <code>settings.panel</code> 插槽的插件。</p>
  </div>
</template>

<style scoped>
.psp-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 200px;
}
.psp-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.psp-tab {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
  cursor: pointer;
  color: var(--text-primary);
}
.psp-tab[aria-selected="true"] {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 12%, var(--bg-elevated));
}
.psp-frame-wrap {
  flex: 1;
  min-height: 280px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.psp-frame {
  width: 100%;
  min-height: 260px;
  flex: 1;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.psp-vue {
  width: 100%;
  min-height: 200px;
  flex: 1;
}
.psp-msg {
  margin: 0;
  font-size: 13px;
}
.psp-msg--err {
  color: var(--text-danger, #c33);
}
.psp-msg--warn {
  font-size: 12px;
  color: var(--text-secondary);
}
.psp-msg--muted {
  color: var(--text-secondary);
  font-size: 13px;
}
code {
  font-size: 0.9em;
}
</style>
