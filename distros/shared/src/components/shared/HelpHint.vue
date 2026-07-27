<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /** Help text shown on click; split into paragraphs on blank lines */
    text?: string
    paragraphs?: readonly string[]
    /** Popover alignment relative to the button; use end near the right edge to avoid clipping */
    popAlign?: 'start' | 'end'
    /** Smaller hint icon and narrower popover for tight areas such as the top bar */
    compact?: boolean
  }>(),
  { popAlign: 'start', compact: false },
)

const { t } = useI18n()

const segments = computed(() => {
  if (props.paragraphs?.length) {
    return props.paragraphs.map(s => s.trim()).filter(Boolean)
  }
  const raw = props.text?.trim() ?? ''
  if (!raw)
    return []
  return raw
    .split(/\n{2,}/)
    .map(s => s.trim())
    .filter(Boolean)
})

const open = ref(false)
const triggerLabel = computed(() =>
  open.value ? t('app.helpHintCloseAria') : t('app.helpHintAria'),
)
const root = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const popover = ref<HTMLElement | null>(null)
const popoverReady = ref(false)
const popoverPlacement = ref<'top' | 'bottom'>('bottom')
const popoverStyle = ref<Record<string, string>>({})
const popoverId = `help-hint-${useId()}`

let positionFrame: number | null = null
let popoverResizeObserver: ResizeObserver | null = null

function toggle(e: Event) {
  e.stopPropagation()
  if (segments.value.length === 0)
    return
  open.value = !open.value
}

/** Capture phase: runs before subtree @click.stop so clicks on panel chrome (e.g. More menu) still close the popover */
function onDocPointerDownCapture(e: PointerEvent) {
  if (!open.value)
    return
  const el = root.value
  const pop = popover.value
  if (
    el
    && !el.contains(e.target as Node)
    && !pop?.contains(e.target as Node)
  ) {
    open.value = false
  }
}

function onDocKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape' || !open.value)
    return
  e.preventDefault()
  e.stopImmediatePropagation()
  open.value = false
  trigger.value?.focus()
}

function updatePopoverPosition() {
  positionFrame = null
  const button = trigger.value
  const pop = popover.value
  if (!open.value || !button || !pop)
    return

  const buttonRect = button.getBoundingClientRect()
  const viewportWidth = document.documentElement.clientWidth || window.innerWidth
  const viewportHeight = document.documentElement.clientHeight || window.innerHeight
  const viewportMargin = 10
  const gap = 8

  if (
    buttonRect.bottom < viewportMargin
    || buttonRect.top > viewportHeight - viewportMargin
    || buttonRect.right < viewportMargin
    || buttonRect.left > viewportWidth - viewportMargin
  ) {
    open.value = false
    return
  }

  const popRect = pop.getBoundingClientRect()
  const preferredLeft = props.popAlign === 'end'
    ? buttonRect.right - popRect.width
    : buttonRect.left
  const maxLeft = Math.max(viewportMargin, viewportWidth - popRect.width - viewportMargin)
  const left = Math.min(Math.max(preferredLeft, viewportMargin), maxLeft)
  const availableBelow = Math.max(0, viewportHeight - buttonRect.bottom - gap - viewportMargin)
  const availableAbove = Math.max(0, buttonRect.top - gap - viewportMargin)
  const placeBelow = popRect.height <= availableBelow
    || (popRect.height > availableAbove && availableBelow >= availableAbove)
  const availableHeight = placeBelow ? availableBelow : availableAbove

  popoverPlacement.value = placeBelow ? 'bottom' : 'top'
  const maxReadableHeight = Math.min(availableHeight, viewportHeight * 0.78, 544)

  popoverStyle.value = {
    left: `${Math.round(left)}px`,
    top: placeBelow ? `${Math.round(buttonRect.bottom + gap)}px` : 'auto',
    bottom: placeBelow ? 'auto' : `${Math.round(viewportHeight - buttonRect.top + gap)}px`,
    maxHeight: `${Math.max(1, Math.floor(maxReadableHeight))}px`,
  }
  popoverReady.value = true
}

function queuePopoverPosition() {
  if (!open.value || positionFrame !== null)
    return
  positionFrame = window.requestAnimationFrame(updatePopoverPosition)
}

function startPositionTracking() {
  window.addEventListener('resize', queuePopoverPosition)
  document.addEventListener('scroll', queuePopoverPosition, true)
  if (typeof ResizeObserver !== 'undefined' && popover.value) {
    popoverResizeObserver = new ResizeObserver(queuePopoverPosition)
    popoverResizeObserver.observe(popover.value)
  }
}

function stopPositionTracking() {
  window.removeEventListener('resize', queuePopoverPosition)
  document.removeEventListener('scroll', queuePopoverPosition, true)
  popoverResizeObserver?.disconnect()
  popoverResizeObserver = null
  if (positionFrame !== null) {
    window.cancelAnimationFrame(positionFrame)
    positionFrame = null
  }
}

const CAPTURE_OPTS = true

onMounted(() => {
  document.addEventListener('pointerdown', onDocPointerDownCapture, CAPTURE_OPTS)
  document.addEventListener('keydown', onDocKeydown, CAPTURE_OPTS)
})

watch(open, (isOpen) => {
  if (isOpen) {
    popoverReady.value = false
    void nextTick(() => {
      if (!open.value)
        return
      updatePopoverPosition()
      startPositionTracking()
    })
  }
  else {
    stopPositionTracking()
  }
})

watch(segments, () => {
  if (open.value)
    void nextTick(queuePopoverPosition)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocPointerDownCapture, CAPTURE_OPTS)
  document.removeEventListener('keydown', onDocKeydown, CAPTURE_OPTS)
  stopPositionTracking()
})
</script>

<template>
  <span
    v-if="segments.length"
    ref="root"
    class="help-hint"
    :class="{ 'help-hint--open': open, 'help-hint--compact': compact }"
  >
    <button
      ref="trigger"
      type="button"
      class="help-btn"
      :aria-expanded="open"
      :aria-controls="popoverId"
      :aria-describedby="open ? popoverId : undefined"
      :aria-label="triggerLabel"
      @click="toggle"
    >
      ?
    </button>
  </span>
  <Teleport to="body">
    <div
      v-if="open && segments.length"
      :id="popoverId"
      ref="popover"
      class="help-pop"
      :class="{
        'help-pop--compact': compact,
      }"
      :data-placement="popoverPlacement"
      :style="{
        ...popoverStyle,
        visibility: popoverReady ? 'visible' : 'hidden',
      }"
      role="tooltip"
    >
      <p v-for="(seg, i) in segments" :key="i" class="help-pop-p">
        {{ seg }}
      </p>
    </div>
  </Teleport>
</template>

<style scoped>
.help-hint {
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
  margin-left: 0.25rem;
  position: relative;
}

.help-btn {
  width: 1.2rem;
  height: 1.2rem;
  border-radius: 50%;
  border: 1px solid color-mix(in srgb, var(--border-light) 90%, var(--text-primary) 10%);
  background: color-mix(in srgb, var(--bg-elevated) 88%, transparent);
  color: var(--text-secondary);
  font-size: 0.68rem;
  font-weight: 700;
  cursor: pointer;
  padding: 0;
  line-height: 1;
  flex-shrink: 0;
  box-shadow: var(--shadow-sm);
  transition: var(--control-transition);
}

.help-btn:hover {
  border-color: var(--border-focus);
  color: var(--text-primary);
  background: var(--bg-secondary);
}

.help-btn:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 2px var(--bg-page),
    0 0 0 4px var(--focus-ring-color);
}

.help-pop {
  position: fixed;
  z-index: 11000;
  box-sizing: border-box;
  /* Comfortable reading width: ~55–65 chars per line, scales with viewport */
  width: min(65ch, calc(100vw - 2rem));
  max-width: min(40rem, calc(100vw - 1.25rem));
  min-width: min(14rem, calc(100vw - 2rem));
  padding: clamp(0.55rem, 0.45rem + 0.35vw, 0.85rem)
    clamp(0.75rem, 0.55rem + 0.6vw, 1.15rem);
  font-size: clamp(0.8125rem, 0.76rem + 0.25vw, 0.9375rem);
  font-weight: 400;
  line-height: 1.65;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--card-bg) 92%, transparent);
  backdrop-filter: blur(10px) saturate(106%);
  -webkit-backdrop-filter: blur(10px) saturate(106%);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
  max-height: min(78vh, 34rem);
  overflow-y: auto;
  overflow-x: hidden;
  overscroll-behavior: contain;
  text-wrap: pretty;
}

.help-pop-p {
  margin: 0 0 0.55em;
  text-align: start;
  hyphens: auto;
}

.help-pop-p:last-child {
  margin-bottom: 0;
}

.help-pop--compact {
  min-width: unset;
  max-width: min(16rem, calc(100vw - 1.25rem));
  padding: 0.5rem 0.7rem;
  font-size: 0.75rem;
  line-height: 1.5;
}

.help-hint--compact .help-btn {
  width: 1.05rem;
  height: 1.05rem;
  font-size: 0.6rem;
}

.help-hint--compact {
  margin-left: 0.15rem;
}
</style>
