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

const SETTINGS_PANEL = "settings.panel";

const panelSlots = ref<PluginUiSlotInfo[]>([]);
const loadError = ref<string | null>(null);
const frameErrors = ref<Record<string, string>>({});
const vueFallback = ref<Record<string, boolean>>({});
const activeTab = ref(0);

async function loadSlots() {
  loadError.value = null;
  try {
    const boot = await getDirectoryPluginBootstrap(currentRoleId.value);
    setHostEventSubscribedEvents(boot.subscribedHostEvents ?? []);
    pluginStore.developerMode = boot.developerMode ?? false;
    panelSlots.value = (boot.uiSlots ?? []).filter((s) => s.slot === SETTINGS_PANEL);
    if (activeTab.value >= panelSlots.value.length) {
      activeTab.value = 0;
    }
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

watch(panelSlots, (list) => {
  if (activeTab.value >= list.length) {
    activeTab.value = 0;
  }
});

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
  <div class="psp-root">
    <div v-if="loadError" class="psp-msg psp-msg--err" role="status">
      {{ loadError }}
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
          @failed="onVueFailed(s.pluginId)"
        />
        <iframe
          v-if="showIframe(s)"
          class="psp-frame"
          :src="s.url"
          :title="`plugin settings ${s.pluginId}`"
          loading="lazy"
          referrerpolicy="no-referrer"
          @error="onFrameError(s.pluginId)"
        />
        <p v-if="frameErrors[s.pluginId]" class="psp-msg psp-msg--warn">
          {{ frameErrors[s.pluginId] }}
        </p>
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
