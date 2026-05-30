<script setup lang="ts">
import type { Component } from 'vue'
import type { OcliveApi } from '../composables/useOclive'
import { confirm } from '@tauri-apps/api/dialog'
import { storeToRefs } from 'pinia'
import {
  computed,
  defineComponent,
  h,
  provide,
  ref,
  shallowRef,
  watch,
} from 'vue'
import { useI18n } from 'vue-i18n'
import { createOcliveApi } from '../composables/useOclive'
import { usePluginStore } from '../stores/pluginStore'
import {
  loadPluginVueComponent,
  PluginVueCompileError,
} from '../utils/compilePluginVueSfc'
import { readPluginAssetText } from '../api'
import { scanVueComponentSource } from '../utils/vueComponentSecurity'
import PluginSkeleton from './PluginSkeleton.vue'

const props = withDefaults(
  defineProps<{
    pluginId: string
    vueComponent: string
    bridgeAssetRel: string
    /**
     * When a boolean is passed, pin that setting (full-shell Vue entry has no pluginStore sync);
     * when omitted, read pluginStore.developerMode (embedded in host app slots).
     */
    developerMode?: boolean
    /** Parent increments to force reload (retry). */
    reloadNonce?: number
    /** Loading skeleton shape */
    skeletonVariant?: 'toolbar' | 'block'
  }>(),
  { skeletonVariant: 'toolbar' },
)

const emit = defineEmits<{
  failed: []
  compileError: [error: PluginVueCompileError]
  loading: [value: boolean]
}>()

const { t } = useI18n()

const pluginStore = usePluginStore()
const { developerMode: storeDeveloperMode } = storeToRefs(pluginStore)
const effectiveDeveloperMode = computed(() =>
  typeof props.developerMode === 'boolean'
    ? props.developerMode
    : storeDeveloperMode.value,
)

const loaded = shallowRef<Component | null>(null)
const loading = ref(false)

/** Call createOcliveApi inside child setup so `on` teardown hooks bind to the correct instance */
const VueSlotInner = defineComponent({
  name: 'OcliveVueSlotInner',
  props: {
    comp: { type: Object, required: true },
    pluginId: { type: String, required: true },
    bridgeAssetRel: { type: String, required: true },
  },
  setup(p) {
    const api: OcliveApi = createOcliveApi(p.pluginId, p.bridgeAssetRel)
    provide('oclive', api)
    return () => h(p.comp as Component)
  },
})

watch(
  () =>
    [
      props.pluginId,
      props.vueComponent,
      effectiveDeveloperMode.value,
      props.reloadNonce ?? 0,
    ] as const,
  async () => {
    loaded.value = null
    loading.value = true
    emit('loading', true)
    let preloadedEntrySource: string | undefined
    if (effectiveDeveloperMode.value) {
      try {
        preloadedEntrySource = await readPluginAssetText(
          props.pluginId,
          props.vueComponent,
        )
        const { warnings } = await scanVueComponentSource(preloadedEntrySource)
        if (warnings.length > 0) {
          const list = warnings.map(w => `- ${w}`).join('\n')
          const ok = await confirm(t('devTools.pluginVueSecurity.confirmBody', { list }), {
            title: t('devTools.pluginVueSecurity.confirmTitle'),
            type: 'warning',
          })
          if (!ok) {
            emit('failed')
            return
          }
        }
      }
      catch (e) {
        console.warn('[AsyncPluginVue] security scan skipped', e)
        preloadedEntrySource = undefined
      }
    }
    try {
      const c = await loadPluginVueComponent(
        props.pluginId,
        props.vueComponent,
        preloadedEntrySource
          ? { preloadedEntrySource }
          : undefined,
      )
      if (!c) {
        emit('failed')
        return
      }
      loaded.value = c
    }
    catch (e) {
      if (e instanceof PluginVueCompileError) {
        emit('compileError', e)
        return
      }
      throw e
    }
    finally {
      loading.value = false
      emit('loading', false)
    }
  },
  { immediate: true },
)
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
