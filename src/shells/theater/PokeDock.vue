<script setup lang="ts">
import type { PokeChipId } from '../../composables/theater/theaterLogic'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { THEATER_POKE_CHIPS } from '../../composables/theater/theaterLogic'
import PokeChip from './PokeChip.vue'
import PokeCustomInput from './PokeCustomInput.vue'

defineProps<{
  disabled: boolean
}>()

const emit = defineEmits<{
  poke: [chipId: PokeChipId]
  custom: [text: string]
}>()

const { t } = useI18n()
const customOpen = ref(false)
</script>

<template>
  <div class="poke-dock" role="toolbar" :aria-label="t('theater.poke.dock')">
    <div class="poke-dock__chips">
      <PokeChip
        v-for="chip in THEATER_POKE_CHIPS"
        :key="chip.id"
        :emoji="chip.emoji"
        :label="t(chip.labelKey)"
        :disabled="disabled"
        @click="emit('poke', chip.id)"
      />
      <PokeChip
        variant="custom"
        emoji="➕"
        :label="t('theater.poke.custom')"
        :disabled="disabled"
        @click="customOpen = !customOpen"
      />
    </div>
    <PokeCustomInput
      :open="customOpen"
      :disabled="disabled"
      @submit="emit('custom', $event)"
      @close="customOpen = false"
    />
  </div>
</template>

<style scoped>
.poke-dock {
  display: flex;
  flex-direction: column;
  background: var(--tool-chrome-editor, var(--bg-primary));
}

.poke-dock__chips {
  display: flex;
  flex-wrap: nowrap;
  gap: var(--tool-space-2, 8px);
  overflow-x: auto;
  padding: var(--tool-space-4, 16px);
}
</style>
