<script setup lang="ts">
import type { ScriptLine } from '../../composables/theater/theaterLogic'

defineProps<{
  line: ScriptLine
}>()
</script>

<template>
  <article
    class="script-line"
    :class="line.cast === 'a' ? 'script-line--a' : 'script-line--b'"
  >
    <div class="script-line__bar" aria-hidden="true" />
    <div class="script-line__body">
      <p class="script-line__name">
        {{ line.name }}
      </p>
      <p class="script-line__text">
        {{ line.text }}
      </p>
      <p v-if="line.stageHint" class="script-line__hint">
        ({{ line.stageHint }})
      </p>
    </div>
  </article>
</template>

<style scoped>
.script-line {
  display: flex;
  gap: var(--tool-space-3, 12px);
  margin-bottom: var(--tool-space-4, 16px);
  animation: scriptLineIn 0.42s ease both;
}

.script-line--a .script-line__bar {
  background: var(--theater-cast-a);
}

.script-line--a .script-line__name {
  color: var(--theater-cast-a);
}

.script-line--b .script-line__bar {
  background: var(--theater-cast-b);
}

.script-line--b .script-line__name {
  color: var(--theater-cast-b);
}

.script-line__bar {
  width: 3px;
  flex-shrink: 0;
  border-radius: 2px;
}

.script-line__body {
  min-width: 0;
}

.script-line__name {
  margin: 0 0 var(--tool-space-1, 4px);
  font-size: var(--tool-fs-sm, 12px);
  font-weight: 700;
}

.script-line__text {
  margin: 0;
  font-size: var(--tool-fs-lg, 15px);
  line-height: var(--tool-line, 1.5);
}

.script-line__hint {
  margin: var(--tool-space-1, 4px) 0 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
  font-style: italic;
}

@keyframes scriptLineIn {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .script-line {
    animation: none;
  }
}
</style>
