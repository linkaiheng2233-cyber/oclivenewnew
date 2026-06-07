<script setup lang="ts">
defineProps<{
  label?: string
}>()
</script>

<template>
  <div class="ui-field-row">
    <span class="ui-field-row__label">
      <slot name="label">
        {{ label }}
      </slot>
    </span>
    <div class="ui-field-row__control">
      <slot />
    </div>
  </div>
</template>
