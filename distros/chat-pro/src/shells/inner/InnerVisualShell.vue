<script setup lang="ts">
import { resolveVisualAdapter } from '@oclive/shared/adapters/visual'
import CharacterInfo from '@oclive/shared/components/role/CharacterInfo.vue'
/**
 * Sprint E placeholder: inner narrative visual shell (catalog `context: inner`).
 * Consumes the same `performance_directive` as social hero; no second LLM.
 */
import { computed } from 'vue'

const props = defineProps<{
  roleId: string
  name: string
  emotion: string
  portraitAssetRelPath?: string | null
  directiveKind?: string | null
}>()

const adapterKind = computed(() => props.directiveKind ?? 'image')
const adapter = computed(() =>
  resolveVisualAdapter(adapterKind.value, { mode: 'inner' }),
)
</script>

<template>
  <div class="inner-visual" data-context="inner">
    <CharacterInfo
      layout="stack"
      :role-id="roleId"
      :name="name"
      :emotion="emotion"
      :portrait-asset-rel-path="portraitAssetRelPath"
    />
    <p v-if="adapter.kind !== 'image'" class="inner-hint">
      {{ adapter.kind }} adapter (inner) — Phase 6 stub
    </p>
  </div>
</template>

<style scoped>
.inner-visual {
  padding: 0.75rem;
  border-radius: 8px;
  background: color-mix(in srgb, var(--fluent-bg-subtle, #f5f5f5) 80%, transparent);
}
.inner-hint {
  margin: 0.35rem 0 0;
  font-size: 0.72rem;
  opacity: 0.75;
}
</style>
