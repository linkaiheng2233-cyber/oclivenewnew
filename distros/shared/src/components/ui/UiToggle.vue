<script setup lang="ts">
import { nextTick, useId } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  modelValue: boolean
  label: string
  description?: string
  disabled?: boolean
  busy?: boolean
  compact?: boolean
}>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'change': [event: Event]
}>()
const { t } = useI18n()
const descriptionId = useId()
function onChange(event: Event) {
  const input = event.target as HTMLInputElement
  emit('update:modelValue', input.checked)
  emit('change', event)
  // A synchronous save can fail before Vue renders the optimistic value.
  // Always reconcile the native checkbox with the owner's accepted state.
  void nextTick(() => {
    input.checked = props.modelValue
  })
}
</script>

<template>
  <label class="ui-toggle" :class="{ 'ui-toggle--disabled': disabled || busy, 'ui-toggle--compact': compact }">
    <span v-if="!compact" class="ui-toggle__copy">
      <strong>{{ label }}</strong>
      <span v-if="description" :id="descriptionId">{{ description }}</span>
    </span>
    <span class="ui-toggle__control">
      <span class="ui-toggle__state" aria-hidden="true">{{ busy ? t('settings.switchSaving') : modelValue ? t('settings.switchOn') : t('settings.switchOff') }}</span>
      <span class="ui-toggle__switch">
        <input type="checkbox" role="switch" :checked="modelValue" :disabled="disabled || busy" :aria-label="label" :aria-describedby="description ? descriptionId : undefined" :aria-busy="busy || undefined" @change="onChange">
        <span class="ui-toggle__track" aria-hidden="true"><span /></span>
      </span>
    </span>
  </label>
</template>

<style scoped>
.ui-toggle { display:flex; justify-content:space-between; align-items:center; gap:24px; min-height:56px; cursor:pointer; color:var(--text-primary); }
.ui-toggle__copy { display:flex; flex:1; flex-direction:column; gap:6px; min-width:0; }
.ui-toggle__copy strong { font-size:14px; font-weight:600; line-height:1.6; }
.ui-toggle__copy>span { color:var(--text-secondary); font-size:12px; line-height:1.7; overflow-wrap:anywhere; }
.ui-toggle__control { flex-shrink:0; display:flex; align-items:center; gap:10px; }
.ui-toggle__state { font-size:12px; color:var(--text-secondary); white-space:nowrap; }
.ui-toggle__switch { width:42px; height:26px; position:relative; display:block; }
.ui-toggle__switch input { position:absolute; inset:0; width:100%; height:100%; opacity:0; margin:0; z-index:1; cursor:inherit; }
.ui-toggle__track { position:absolute; inset:0; border-radius:20px; border:1px solid var(--text-muted,#8c9793); background:color-mix(in srgb,var(--text-secondary) 20%,var(--bg-secondary)); transition:background .16s,border-color .16s; }
.ui-toggle__track>span { position:absolute; top:3px; left:3px; width:18px; height:18px; border-radius:50%; background:var(--text-secondary); transition:transform .16s,background .16s; }
input:checked+.ui-toggle__track { background:var(--accent,#486b62); border-color:var(--accent,#486b62); }
input:checked+.ui-toggle__track>span { transform:translateX(16px); background:var(--accent-text,#fff); }
input:focus-visible+.ui-toggle__track { outline:2px solid var(--accent); outline-offset:4px; }
.ui-toggle--disabled { cursor:not-allowed; }
.ui-toggle--compact { min-height:30px; }
.ui-toggle--disabled .ui-toggle__switch { opacity:.55; }
@media(max-width:600px) { .ui-toggle{gap:14px}.ui-toggle__control{gap:6px}.ui-toggle__state{font-size:11px} }
@media(prefers-reduced-motion:reduce) { .ui-toggle__track,.ui-toggle__track>span{transition:none} }
:global(html[data-skin="win98"]) .ui-toggle__track { border-radius:0; border-style:inset; }
:global(html[data-skin="win98"]) .ui-toggle__track>span { border-radius:0; }
</style>
