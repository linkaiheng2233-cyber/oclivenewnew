<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted, ref, watch } from "vue";
import AsyncPluginVue from "./AsyncPluginVue.vue";
import {
  getDirectoryPluginBootstrap,
  type PluginUiSlotInfo,
} from "../utils/tauri-api";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";

const props = withDefaults(
  defineProps<{
    /** 变更后重新拉取 bootstrap（与插件管理面板保存联动） */
    bootstrapEpoch?: number;
  }>(),
  { bootstrapEpoch: 0 },
);

const roleStore = useRoleStore();
const { currentRoleId } = storeToRefs(roleStore);
const pluginStore = usePluginStore();

const ROLE_DETAIL = "role.detail";

const slots = ref<PluginUiSlotInfo[]>([]);
const loadError = ref<string | null>(null);
const frameErrors = ref<Record<string, string>>({});
const vueFallback = ref<Record<string, boolean>>({});

async function loadSlots() {
  loadError.value = null;
  try {
    const boot = await getDirectoryPluginBootstrap(currentRoleId.value);
    pluginStore.applyDirectoryBootstrap(boot);
    slots.value = (boot.uiSlots ?? []).filter((s) => s.slot === ROLE_DETAIL);
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(loadSlots);
watch(
  () => [props.bootstrapEpoch, currentRoleId.value] as const,
  () => {
    vueFallback.value = {};
    void loadSlots();
  },
);

function onFrameError(pluginId: string) {
  frameErrors.value = {
    ...frameErrors.value,
    [pluginId]: "页面加载失败",
  };
}

function onVueFailed(pluginId: string) {
  vueFallback.value = { ...vueFallback.value, [pluginId]: true };
}

function showIframe(s: PluginUiSlotInfo): boolean {
  if (pluginStore.pluginState.force_iframe_mode) return true;
  const vc = s.vueComponent?.trim();
  if (!vc) return true;
  return vueFallback.value[s.pluginId] === true;
}

function showVue(s: PluginUiSlotInfo): boolean {
  if (pluginStore.pluginState.force_iframe_mode) return false;
  const vc = s.vueComponent?.trim();
  if (!vc) return false;
  return vueFallback.value[s.pluginId] !== true;
}
</script>

<template>
  <div v-if="loadError" class="prd-msg prd-msg--err" role="status">{{ loadError }}</div>
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
