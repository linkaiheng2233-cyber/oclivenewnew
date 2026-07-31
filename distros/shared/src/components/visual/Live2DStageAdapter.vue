<script setup lang="ts">
/**
 * Theater Live2D stage adapter (Phase 5).
 * Falls back to PNG hero when Cubism runtime is unavailable.
 */
import { computed, ref, watch } from 'vue'
import CharacterInfo from '../role/CharacterInfo.vue'

const props = defineProps<{
  roleId: string
  name: string
  emotion: string
  visualStateId?: string | null
  performanceDirective?: {
    kind?: string
    path?: string | null
    fallback_image?: string | null
    live2d_model?: string | null
    motion?: string | null
  } | null
}>()

const live2dReady = ref(false)
const loadError = ref<string | null>(null)

const isLive2d = computed(
  () => props.performanceDirective?.kind === 'live2d',
)

const portraitAssetRelPath = computed(() => {
  const d = props.performanceDirective
  return d?.path ?? d?.fallback_image ?? null
})

watch(
  () => props.performanceDirective?.live2d_model,
  (model) => {
    live2dReady.value = false
    loadError.value = null
    if (!model || !isLive2d.value)
      return
    // Cubism hook placeholder: mount model in future Theater build.
    loadError.value = 'Live2D runtime not bundled in this build; using fallback image.'
  },
  { immediate: true },
)
</script>

<template>
  <div class="live2d-stage">
    <div v-if="isLive2d && live2dReady" class="live2d-canvas" />
    <CharacterInfo
      v-else
      layout="stack"
      :role-id="roleId"
      :name="name"
      :emotion="emotion"
      :portrait-asset-rel-path="portraitAssetRelPath"
    />
    <p v-if="loadError" class="live2d-hint">
      {{ loadError }}
    </p>
  </div>
</template>

<style scoped>
.live2d-stage {
  width: 100%;
}
.live2d-hint {
  margin: 0.35rem 0 0;
  font-size: 0.75rem;
  color: var(--fluent-text-secondary, #666);
}
</style>
