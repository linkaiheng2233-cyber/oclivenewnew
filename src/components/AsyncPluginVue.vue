<script setup lang="ts">
import type { Component } from "vue";
import { defineComponent, h, provide, shallowRef, watch } from "vue";
import { confirm } from "@tauri-apps/api/dialog";
import { storeToRefs } from "pinia";
import { loadPluginVueComponent } from "../utils/compilePluginVueSfc";
import { createOcliveApi, type OcliveApi } from "../composables/useOclive";
import { usePluginStore } from "../stores/pluginStore";
import { readPluginAssetText } from "../utils/tauri-api";
import { scanVueComponentSource } from "../utils/vueComponentSecurity";

const props = defineProps<{
  pluginId: string;
  vueComponent: string;
  bridgeAssetRel: string;
}>();

const emit = defineEmits<{
  failed: [];
}>();

const pluginStore = usePluginStore();
const { developerMode } = storeToRefs(pluginStore);

const loaded = shallowRef<Component | null>(null);

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
  () => [props.pluginId, props.vueComponent] as const,
  async () => {
    loaded.value = null;
    if (developerMode.value) {
      try {
        const src = await readPluginAssetText(props.pluginId, props.vueComponent);
        const { warnings } = scanVueComponentSource(src);
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
      }
    }
    const c = await loadPluginVueComponent(
      props.pluginId,
      props.vueComponent,
    );
    if (!c) {
      emit("failed");
      return;
    }
    loaded.value = c;
  },
  { immediate: true },
);
</script>

<template>
  <VueSlotInner
    v-if="loaded"
    :key="`${pluginId}-${bridgeAssetRel}`"
    :comp="loaded"
    :plugin-id="pluginId"
    :bridge-asset-rel="bridgeAssetRel"
  />
</template>
