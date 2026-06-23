<script setup lang="ts">
import type { PluginVueCompileError } from '@oclive/shared/utils/compilePluginVueSfc'
import { storeToRefs } from 'pinia'
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AsyncPluginVue from '@oclive/shared/components/AsyncPluginVue.vue'
import PluginErrorPlaceholder from '@oclive/shared/components/PluginErrorPlaceholder.vue'
import { useSinglePluginError } from '@oclive/shared/composables/usePluginError'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'

const props = defineProps<{
  pluginId: string
  vueEntry: string
  /** Passed to `plugin_bridge_invoke` as `assetRel`; same path as `shell.vueEntry` */
  bridgeAssetRel: string
  htmlFallbackUrl: string
  developerMode: boolean
}>()

const { t, locale } = useI18n()

function syncBrowserChromeFromLocale(): void {
  document.title = t('app.documentTitle')
  document.documentElement.setAttribute('lang', locale.value === 'en-US' ? 'en' : 'zh-CN')
}

onMounted(() => {
  syncBrowserChromeFromLocale()
})

watch(locale, () => {
  syncBrowserChromeFromLocale()
})

const pluginStore = usePluginStore()
const { bootstrapEpoch } = storeToRefs(pluginStore)

const reloadNonce = ref(0)
const {
  message: loadError,
  detail: errorDetail,
  clearError,
  setError,
} = useSinglePluginError()

function onFailed() {
  setError(t('devTools.directoryShell.loadFail'), null)
}

function onCompileError(err: PluginVueCompileError) {
  setError(err.friendlyMessage, err.rawMessage)
}

function retry() {
  clearError()
  reloadNonce.value += 1
}

function useHtmlFallback() {
  window.location.replace(props.htmlFallbackUrl)
}

watch(bootstrapEpoch, () => {
  clearError()
  reloadNonce.value += 1
})
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
