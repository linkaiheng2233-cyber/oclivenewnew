<script setup lang="ts">
import type { Component } from "vue";
import {
  computed,
  defineComponent,
  h,
  provide,
  ref,
  shallowRef,
  watch,
  withDefaults,
} from "vue";
import { confirm } from "@tauri-apps/api/dialog";
import { storeToRefs } from "pinia";
import {
  loadPluginVueComponent,
  PluginVueCompileError,
} from "../utils/compilePluginVueSfc";
import { createOcliveApi, type OcliveApi } from "../composables/useOclive";
import { usePluginStore } from "../stores/pluginStore";
import { readPluginAssetText } from "../utils/tauri-api";
import { scanVueComponentSource } from "../utils/vueComponentSecurity";
import PluginSkeleton from "./PluginSkeleton.vue";

const props = withDefaults(
  defineProps<{
    pluginId: string;
    vueComponent: string;
    bridgeAssetRel: string;
    /**
     * 传入布尔值时固定使用该设置（整壳 Vue 入口无 `pluginStore` 同步）；
     * 省略时从 `pluginStore.developerMode` 读取（嵌入主应用插槽）。
     */
    developerMode?: boolean;
    /** 父级递增以强制重新加载（重试）。 */
    reloadNonce?: number;
    /** 加载占位骨架形态 */
    skeletonVariant?: "toolbar" | "block";
  }>(),
  { skeletonVariant: "toolbar" },
);

const emit = defineEmits<{
  failed: [];
  compileError: [error: PluginVueCompileError];
  loading: [value: boolean];
}>();

const pluginStore = usePluginStore();
const { developerMode: storeDeveloperMode } = storeToRefs(pluginStore);
const effectiveDeveloperMode = computed(() =>
  typeof props.developerMode === "boolean"
    ? props.developerMode
    : storeDeveloperMode.value,
);

const loaded = shallowRef<Component | null>(null);
const loading = ref(false);

/** 在子组件 setup 内调用 createOcliveApi，保证 `on` 的卸载钩子绑定到正确实例 */
const VueSlotInner = defineComponent({
  name: "OcliveVueSlotInner",
  props: {
    comp: { type: Object, required: true },
    pluginId: { type: String, required: true },
    bridgeAssetRel: { type: String, required: true },
  },
  setup(p) {
    const api: OcliveApi = createOcliveApi(p.pluginId, p.bridgeAssetRel);
    provide("oclive", api);
    return () => h(p.comp as Component);
  },
});

watch(
  () =>
    [
      props.pluginId,
      props.vueComponent,
      effectiveDeveloperMode.value,
      props.reloadNonce ?? 0,
    ] as const,
  async () => {
    loaded.value = null;
    loading.value = true;
    emit("loading", true);
    let preloadedEntrySource: string | undefined;
    if (effectiveDeveloperMode.value) {
      try {
        preloadedEntrySource = await readPluginAssetText(
          props.pluginId,
          props.vueComponent,
        );
        const { warnings } = scanVueComponentSource(preloadedEntrySource);
        if (warnings.length > 0) {
          const ok = await confirm(
            `此插件包含潜在危险代码：\n${warnings.map((w) => `- ${w}`).join("\n")}\n\n是否继续加载？`,
            { title: "插件安全警告", type: "warning" },
          );
          if (!ok) {
            emit("failed");
            return;
          }
        }
      } catch (e) {
        console.warn("[AsyncPluginVue] security scan skipped", e);
        preloadedEntrySource = undefined;
      }
    }
    try {
      const c = await loadPluginVueComponent(
        props.pluginId,
        props.vueComponent,
        preloadedEntrySource
          ? { preloadedEntrySource }
          : undefined,
      );
      if (!c) {
        emit("failed");
        return;
      }
      loaded.value = c;
    } catch (e) {
      if (e instanceof PluginVueCompileError) {
        emit("compileError", e);
        return;
      }
      throw e;
    } finally {
      loading.value = false;
      emit("loading", false);
    }
  },
  { immediate: true },
);
</script>

<template>
  <PluginSkeleton
    v-if="loading && !loaded"
    class="apv-skel"
    :variant="props.skeletonVariant"
  />
  <VueSlotInner
    v-else-if="loaded"
    :key="`${pluginId}-${bridgeAssetRel}-${reloadNonce ?? 0}`"
    :comp="loaded"
    :plugin-id="pluginId"
    :bridge-asset-rel="bridgeAssetRel"
  />
</template>

<style scoped>
.apv-skel {
  min-width: 120px;
}
</style>
