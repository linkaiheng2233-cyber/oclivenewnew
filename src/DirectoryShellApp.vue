<script setup lang="ts">
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import AsyncPluginVue from "./components/AsyncPluginVue.vue";
import PluginErrorPlaceholder from "./components/PluginErrorPlaceholder.vue";
import { useSinglePluginError } from "./composables/usePluginError";
import { usePluginStore } from "./stores/pluginStore";
import type { PluginVueCompileError } from "./utils/compilePluginVueSfc";

const { t } = useI18n();

const props = defineProps<{
  pluginId: string;
  vueEntry: string;
  /** 传给 `plugin_bridge_invoke` 的 `assetRel`，与 `shell.vueEntry` 路径一致 */
  bridgeAssetRel: string;
  htmlFallbackUrl: string;
  developerMode: boolean;
}>();

const pluginStore = usePluginStore();
const { bootstrapEpoch } = storeToRefs(pluginStore);

const reloadNonce = ref(0);
const {
  message: loadError,
  detail: errorDetail,
  clearError,
  setError,
} = useSinglePluginError();

function onFailed() {
  setError(t("devTools.directoryShell.loadFail"), null);
}

function onCompileError(err: PluginVueCompileError) {
  setError(err.friendlyMessage, err.rawMessage);
}

function retry() {
  clearError();
  reloadNonce.value += 1;
}

function useHtmlFallback() {
  window.location.replace(props.htmlFallbackUrl);
}

watch(bootstrapEpoch, () => {
  clearError();
  reloadNonce.value += 1;
});
</script>

<template>
  <div class="oclive-directory-shell-vue">
    <PluginErrorPlaceholder
      v-if="loadError"
      :title="t('devTools.directoryShell.title')"
      :message="loadError"
      :detail="errorDetail ?? undefined"
      :show-retry="true"
      :show-fallback="true"
      :retry-label="t('devTools.directoryShell.retry')"
      :fallback-label="t('devTools.directoryShell.useHtml')"
      @retry="retry"
      @fallback="useHtmlFallback"
    />
    <AsyncPluginVue
      v-else
      :key="`${bootstrapEpoch}-${reloadNonce}`"
      :plugin-id="pluginId"
      :vue-component="vueEntry"
      :bridge-asset-rel="bridgeAssetRel"
      :developer-mode="developerMode"
      :reload-nonce="reloadNonce"
      skeleton-variant="block"
      @failed="onFailed"
      @compile-error="onCompileError($event)"
    />
  </div>
</template>

<style scoped>
.oclive-directory-shell-vue {
  width: 100vw;
  min-height: 100vh;
  box-sizing: border-box;
  padding: 16px;
}
</style>
