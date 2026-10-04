<script setup lang="ts">
import type { MinimalRoleLocalSource } from '@oclive/shared/api/chat'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  source: Readonly<MinimalRoleLocalSource> | null
  disabled?: boolean
  loading?: boolean
}>()
const emit = defineEmits<{
  bind: [source: MinimalRoleLocalSource]
  return: []
  cancel: []
}>()
const { t } = useI18n()
const assetRoot = ref('')
const definitionReference = ref('')
watch(() => props.source, (source) => {
  if (source) {
    assetRoot.value = source.asset_root
    definitionReference.value = source.definition_reference
  }
}, { immediate: true })
function bind() {
  if (props.disabled || !assetRoot.value.trim() || !definitionReference.value.trim())
    return
  emit('bind', {
    role_id: `minimal-${crypto.randomUUID()}`,
    asset_root: assetRoot.value,
    definition_reference: definitionReference.value,
  })
}
</script>

<template>
  <details class="minimal-source">
    <summary>{{ t(source ? 'app.minimalRole.active' : 'app.minimalRole.select') }}</summary>
    <div class="minimal-source__panel">
      <p>{{ t('app.minimalRole.scope') }}</p>
      <form @submit.prevent="bind">
        <label>
          {{ t('app.minimalRole.assetRoot') }}
          <input v-model="assetRoot" name="asset-root" :disabled="disabled" required autocomplete="off">
        </label>
        <label>
          {{ t('app.minimalRole.definitionReference') }}
          <input v-model="definitionReference" name="definition-reference" :disabled="disabled" required autocomplete="off">
        </label>
        <p>{{ t('app.minimalRole.validationHint') }}</p>
        <button type="submit" :disabled="disabled || !assetRoot.trim() || !definitionReference.trim()">
          {{ t('app.minimalRole.bind') }}
        </button>
      </form>
      <button v-if="loading" type="button" data-action="cancel-minimal" :disabled="disabled" @click="emit('cancel')">
        {{ t('app.minimalRole.stopWaiting') }}
      </button>
      <button v-if="source" type="button" data-action="return-rich" :disabled="disabled" @click="emit('return')">
        {{ t('app.minimalRole.return') }}
      </button>
    </div>
  </details>
</template>

<style scoped>
.minimal-source { position: relative; max-width: 100%; font-size: 0.85rem; }
summary { cursor: pointer; padding: 6px; }
.minimal-source__panel { position: absolute; top: 100%; right: 0; width: min(420px, 85vw); padding: 12px; border: 1px solid var(--border-light); background: var(--bg-primary); color: var(--text-primary); z-index: 100; box-shadow: 0 4px 16px #0002; }
label { display: block; margin: 8px 0; }
input { display: block; width: 100%; box-sizing: border-box; }
button { margin: 4px; }
p { margin: 6px 0; }
</style>
