<script setup lang="ts">
import type { TheaterSourceKind } from '../../composables/theater/theaterLogic'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  source: TheaterSourceKind
  beat?: number
  total?: number
}>()

const { t } = useI18n()

const sourceLabel = computed(() => t(`theater.footer.source.${props.source}`))
</script>

<template>
  <footer class="theater-footer" role="status">
    <span class="theater-footer__source">{{ sourceLabel }}</span>
    <span v-if="beat != null && total != null && total > 0" class="theater-footer__beat">
      · {{ t('theater.footer.beat', { beat, total }) }}
    </span>
  </footer>
</template>

<style scoped>
.theater-footer {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  min-height: var(--tool-statusbar-h, 24px);
  padding: 0 var(--tool-space-4, 16px);
  border-top: 1px solid var(--tool-divider, var(--border-light));
  background: var(--tool-chrome-status, var(--bg-status));
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
}

.theater-footer__beat {
  margin-left: var(--tool-space-1, 4px);
}
</style>
