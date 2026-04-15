<script setup lang="ts">
import AsyncPluginVue from "./components/AsyncPluginVue.vue";

const props = defineProps<{
  pluginId: string;
  vueEntry: string;
  /** 传给 `plugin_bridge_invoke` 的 `assetRel`，与 `shell.vueEntry` 路径一致 */
  bridgeAssetRel: string;
  htmlFallbackUrl: string;
  developerMode: boolean;
}>();

function onFailed() {
  window.location.replace(props.htmlFallbackUrl);
}
</script>

<template>
  <div class="oclive-directory-shell-vue">
    <AsyncPluginVue
      :plugin-id="pluginId"
      :vue-component="vueEntry"
      :bridge-asset-rel="bridgeAssetRel"
      :developer-mode="developerMode"
      @failed="onFailed"
    />
  </div>
</template>

<style scoped>
.oclive-directory-shell-vue {
  width: 100vw;
  min-height: 100vh;
  box-sizing: border-box;
}
</style>
