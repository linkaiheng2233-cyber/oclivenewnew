<script setup lang="ts">
defineProps<{
  title?: string
  description?: string
}>()
</script>

<template>
  <section class="ui-section">
    <header v-if="title || description || $slots.title || $slots.extra" class="ui-section__head">
      <div class="ui-section__title-row">
        <h3 v-if="title || $slots.title" class="ui-section__title">
          <slot name="title">
            {{ title }}
          </slot>
        </h3>
        <slot name="extra" />
      </div>
      <p v-if="description" class="ui-section__desc">
        {{ description }}
      </p>
    </header>
    <div class="ui-section__body">
      <slot />
    </div>
  </section>
</template>
