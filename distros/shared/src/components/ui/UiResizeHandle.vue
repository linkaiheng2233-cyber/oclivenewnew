<script setup lang="ts">
import { onBeforeUnmount } from 'vue'

const props = defineProps<{
  edge: 'left' | 'right'
  ariaLabel?: string
}>()

const emit = defineEmits<{
  resize: [deltaX: number]
  resizeStart: []
  resizeEnd: []
}>()

const KEY_STEP = 8

let dragging = false
let startX = 0

function onPointerDown(e: PointerEvent): void {
  if (e.button !== 0)
    return
  dragging = true
  startX = e.clientX
  emit('resizeStart')
  e.currentTarget?.setPointerCapture(e.pointerId)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function onPointerMove(e: PointerEvent): void {
  if (!dragging)
    return
  const delta = e.clientX - startX
  startX = e.clientX
  if (delta !== 0)
    emit('resize', delta)
}

function endDrag(e: PointerEvent): void {
  if (!dragging)
    return
  dragging = false
  try {
    e.currentTarget?.releasePointerCapture(e.pointerId)
  }
  catch {
    /* ignore */
  }
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  emit('resizeEnd')
}

function onKeyDown(e: KeyboardEvent): void {
  if (e.key === 'ArrowLeft') {
    e.preventDefault()
    emit('resize', props.edge === 'left' ? -KEY_STEP : KEY_STEP)
  }
  else if (e.key === 'ArrowRight') {
    e.preventDefault()
    emit('resize', props.edge === 'left' ? KEY_STEP : -KEY_STEP)
  }
}

onBeforeUnmount(() => {
  if (dragging) {
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
})
</script>

<template>
  <div
    class="ui-resize-handle"
    role="separator"
    aria-orientation="vertical"
    :aria-label="ariaLabel"
    tabindex="0"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="endDrag"
    @pointercancel="endDrag"
    @keydown="onKeyDown"
  />
</template>

<style scoped>
.ui-resize-handle {
  flex: 0 0 4px;
  width: 4px;
  min-width: 4px;
  align-self: stretch;
  cursor: col-resize;
  touch-action: none;
  background: transparent;
  position: relative;
  z-index: 2;
  box-shadow: inset -1px 0 0 var(--tool-divider, var(--tool-border, var(--border-light)));
}

.ui-resize-handle::after {
  content: '';
  position: absolute;
  inset: 0 -2px;
}

.ui-resize-handle:hover,
.ui-resize-handle:focus-visible {
  box-shadow: none;
  background: color-mix(in srgb, var(--tool-accent, var(--accent)) 40%, transparent);
}

.ui-resize-handle:focus-visible {
  outline: none;
}
</style>
