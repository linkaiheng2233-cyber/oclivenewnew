<script setup lang="ts">
defineProps<{
  modelValue?: string
  disabled?: boolean
  id?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': [event: Event]
}>()

function onChange(e: Event) {
  const value = (e.target as HTMLSelectElement).value
  emit('update:modelValue', value)
  emit('change', e)
}
</script>

<template>
  <select
    :id="id"
    class="ui-select"
    :value="modelValue"
    :disabled="disabled"
    @change="onChange"
  >
    <slot />
  </select>
</template>
