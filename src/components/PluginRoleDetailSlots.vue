<script setup lang="ts">
import AsyncPluginVue from "./AsyncPluginVue.vue";
import { useDirectoryPluginSlotEmbed } from "../composables/useDirectoryPluginSlotEmbed";
import { SLOT_ROLE_DETAIL } from "../stores/pluginStore";

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
  onFrameError,
  onVueFailed,
  showIframe,
  showVue,
} = useDirectoryPluginSlotEmbed({
  slot: SLOT_ROLE_DETAIL,
  bootstrapEpoch: () => props.bootstrapEpoch,
});
</script>

<template>
  <div v-if="pluginError" class="prd-msg prd-msg--err" role="status">{{ pluginError }}</div>
  <div v-else-if="slots.length > 0" class="prd-list" aria-label="角色详情插件插槽">
    <div v-for="s in slots" :key="s.pluginId" class="prd-item">
      <AsyncPluginVue
        v-if="showVue(s)"
        class="prd-vue"
        :plugin-id="s.pluginId"
        :vue-component="s.vueComponent!"
        :bridge-asset-rel="s.entry"
        @failed="onVueFailed(s.pluginId)"
      />
      <iframe
        v-if="showIframe(s)"
        class="prd-frame"
        :src="s.url"
        :title="`plugin role.detail ${s.pluginId}`"
        loading="lazy"
        referrerpolicy="no-referrer"
        @error="onFrameError(s.pluginId)"
      />
      <p v-if="frameErrors[s.pluginId]" class="prd-msg prd-msg--muted">{{ frameErrors[s.pluginId] }}</p>
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
