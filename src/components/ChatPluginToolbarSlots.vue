<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted, ref, watch } from "vue";
import AsyncPluginVue from "./AsyncPluginVue.vue";
import { setHostEventSubscribedEvents } from "../lib/hostEventBus";
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

const slots = ref<PluginUiSlotInfo[]>([]);
const loadError = ref<string | null>(null);
/** pluginId -> 加载失败 */
const frameErrors = ref<Record<string, string>>({});
/** Vue 编译/运行失败时回退 iframe */
const vueFallback = ref<Record<string, boolean>>({});

const CHAT_TOOLBAR = "chat_toolbar";

async function loadSlots() {
  loadError.value = null;
  try {
    const boot = await getDirectoryPluginBootstrap(currentRoleId.value);
    setHostEventSubscribedEvents(boot.subscribedHostEvents ?? []);
    pluginStore.developerMode = boot.developerMode ?? false;
    slots.value = (boot.uiSlots ?? []).filter((s) => s.slot === CHAT_TOOLBAR);
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
  <div v-if="loadError" class="plugin-toolbar plugin-toolbar--error" role="status">
    {{ loadError }}
  </div>
  <div
    v-else-if="slots.length > 0"
    class="plugin-toolbar"
    aria-label="插件工具栏"
  >
    <template v-for="s in slots" :key="s.pluginId">
      <AsyncPluginVue
        v-if="showVue(s)"
        class="plugin-toolbar-vue"
        :plugin-id="s.pluginId"
        :vue-component="s.vueComponent!"
        :bridge-asset-rel="s.entry"
        @failed="onVueFailed(s.pluginId)"
      />
      <iframe
        v-if="showIframe(s)"
        class="plugin-toolbar-frame"
        :src="s.url"
        :title="`plugin ${s.pluginId}`"
        loading="lazy"
        referrerpolicy="no-referrer"
        @error="onFrameError(s.pluginId)"
      />
    </template>
    <span
      v-for="s in slots"
      v-show="frameErrors[s.pluginId]"
      :key="`err-${s.pluginId}`"
      class="plugin-toolbar-fail"
    >
      {{ frameErrors[s.pluginId] }}
    </span>
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
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
