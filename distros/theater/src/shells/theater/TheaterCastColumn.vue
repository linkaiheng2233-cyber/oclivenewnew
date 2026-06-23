<script setup lang="ts">
import type { CastColumnEntry } from '../../composables/theater/theaterPortrait'
import TheaterCastPortrait from './TheaterCastPortrait.vue'

defineProps<{
  side: 'left' | 'right'
  casts: CastColumnEntry[]
}>()
</script>

<template>
  <div class="theater-cast-column" :data-side="side">
    <TheaterCastPortrait
      v-for="entry in casts"
      :key="entry.castId"
      :side="side"
      :role-id="entry.roleId"
      :name="entry.name"
      :emotion="entry.emotion"
      :active="entry.active"
      :event-affected="entry.eventAffected"
    />
  </div>
</template>

<style scoped>
.theater-cast-column {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  gap: var(--tool-space-2, 8px);
  flex-shrink: 0;
}
</style>
