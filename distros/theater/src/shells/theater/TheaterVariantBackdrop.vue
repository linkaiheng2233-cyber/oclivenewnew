<script setup lang="ts">
import type { ScriptLine } from '../../composables/theater/theaterLogic'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  visible: boolean
  patchA: ScriptLine[]
  patchB: ScriptLine[]
}>()

const emit = defineEmits<{
  selectB: []
  dismiss: []
}>()

const { t } = useI18n()
const dragPx = ref(0)
const dragging = ref(false)
const THRESHOLD_PX = 72

const revealRatio = computed(() => Math.min(1, Math.max(0, dragPx.value / 160)))

function formatLine(line: ScriptLine): string {
  const hint = line.stageHint ? ` (${line.stageHint})` : ''
  return `${line.name}：${line.text}${hint}`
}

function onPointerDown(ev: PointerEvent) {
  dragging.value = true
  ;(ev.target as HTMLElement).setPointerCapture(ev.pointerId)
}

function onPointerMove(ev: PointerEvent) {
  if (!dragging.value)
    return
  dragPx.value = Math.max(0, dragPx.value - ev.movementX)
}

function onPointerUp() {
  if (!dragging.value)
    return
  dragging.value = false
  if (dragPx.value >= THRESHOLD_PX)
    emit('selectB')
  dragPx.value = 0
}

function onDismiss() {
  emit('dismiss')
}
</script>

<template>
  <div
    v-if="visible && patchB.length > 0"
    class="theater-variant-backdrop"
    role="region"
    :aria-label="t('theater.variant.aria')"
  >
    <div class="theater-variant-backdrop__hint">
      {{ t('theater.variant.hint') }}
    </div>
    <div
      class="theater-variant-backdrop__panel"
      :style="{ transform: `translateX(${-revealRatio * 100}%)` }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <div class="theater-variant-backdrop__layer theater-variant-backdrop__layer--a">
        <p class="theater-variant-backdrop__label">
          {{ t('theater.variant.current') }}
        </p>
        <p
          v-for="line in patchA"
          :key="line.id"
          class="theater-variant-backdrop__line"
        >
          {{ formatLine(line) }}
        </p>
      </div>
      <div class="theater-variant-backdrop__layer theater-variant-backdrop__layer--b">
        <p class="theater-variant-backdrop__label">
          {{ t('theater.variant.alternate') }}
        </p>
        <p
          v-for="line in patchB"
          :key="line.id"
          class="theater-variant-backdrop__line theater-variant-backdrop__line--alt"
        >
          {{ formatLine(line) }}
        </p>
      </div>
      <div class="theater-variant-backdrop__handle">
        <span>{{ t('theater.variant.drag') }}</span>
      </div>
    </div>
    <button
      type="button"
      class="theater-variant-backdrop__close"
      @click="onDismiss"
    >
      {{ t('theater.variant.keepA') }}
    </button>
  </div>
</template>

<style scoped>
.theater-variant-backdrop {
  position: absolute;
  inset: auto 0 0;
  z-index: 4;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px) var(--tool-space-3, 12px);
  pointer-events: none;
}

.theater-variant-backdrop__hint {
  margin: 0 0 6px;
  font-size: var(--tool-fs-xs, 11px);
  color: var(--text-secondary);
  text-align: center;
}

.theater-variant-backdrop__panel {
  position: relative;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  overflow: hidden;
  border-radius: var(--tool-radius-lg, 12px);
  border: 1px solid var(--theater-variant-border, color-mix(in srgb, var(--tool-accent) 35%, var(--border-light)));
  background: var(--theater-variant-bg, color-mix(in srgb, var(--tool-elevated) 92%, var(--tool-accent) 8%));
  box-shadow: 0 -4px 24px color-mix(in srgb, var(--text-primary) 8%, transparent);
  cursor: grab;
  touch-action: none;
  pointer-events: auto;
}

.theater-variant-backdrop__panel:active {
  cursor: grabbing;
}

.theater-variant-backdrop__layer {
  padding: 10px 12px 36px;
  min-width: 0;
}

.theater-variant-backdrop__layer--b {
  background: var(--theater-variant-alt-bg, color-mix(in srgb, var(--theater-cast-a-soft) 40%, transparent));
}

.theater-variant-backdrop__label {
  margin: 0 0 6px;
  font-size: var(--tool-fs-xs, 11px);
  font-weight: 600;
  color: var(--text-secondary);
}

.theater-variant-backdrop__line {
  margin: 0 0 4px;
  font-size: var(--tool-fs-sm, 12px);
  line-height: 1.45;
  color: var(--text-primary);
}

.theater-variant-backdrop__line--alt {
  color: color-mix(in srgb, var(--theater-cast-a) 70%, var(--text-primary));
}

.theater-variant-backdrop__handle {
  position: absolute;
  left: 50%;
  bottom: 6px;
  transform: translateX(-50%);
  padding: 2px 10px;
  border-radius: 999px;
  font-size: var(--tool-fs-xs, 11px);
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--tool-elevated) 80%, transparent);
  border: 1px dashed var(--border-light);
}

.theater-variant-backdrop__close {
  display: block;
  margin: 8px auto 0;
  padding: 4px 12px;
  font-size: var(--tool-fs-xs, 11px);
  color: var(--text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  pointer-events: auto;
}

.theater-variant-backdrop__close:hover {
  color: var(--text-primary);
}
</style>
