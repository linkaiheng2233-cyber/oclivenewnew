<script setup lang="ts" generic="T extends Record<string, unknown>">
import type { CSSProperties } from 'vue'
import {
  computed,

  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from 'vue'

const props = withDefaults(
  defineProps<{
    items: T[]
    /** Field name used as Vue `:key` (default `id`). */
    itemKey?: string
    estimatedItemHeight?: number
    /** Extra rows rendered above/below the viewport. */
    buffer?: number
    /** When true, new items scroll into view if the user was already at the bottom. */
    stickToBottom?: boolean
  }>(),
  {
    itemKey: 'id',
    estimatedItemHeight: 96,
    buffer: 3,
    stickToBottom: true,
  },
)

const emit = defineEmits<{
  scroll: [scrollTop: number]
}>()

const rootRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(0)
const heights = ref<number[]>([])
const userPinnedUp = ref(false)

/** Hysteresis: avoid pin/unpin flicker when row heights remeasure near the bottom. */
const PIN_UP_THRESHOLD_PX = 120
const PIN_DOWN_THRESHOLD_PX = 32

const resizeObservers = new Map<number, ResizeObserver>()

function keyOf(item: T, index: number): string | number {
  const k = props.itemKey
  const v = item[k]
  if (typeof v === 'string' || typeof v === 'number')
    return v
  return index
}

function ensureHeights() {
  const est = props.estimatedItemHeight
  while (heights.value.length < props.items.length) {
    heights.value.push(est)
  }
  if (heights.value.length > props.items.length) {
    heights.value.length = props.items.length
  }
}

watch(
  () => props.items.length,
  () => {
    ensureHeights()
  },
  { immediate: true },
)

const prefixOffsets = computed(() => {
  const acc = [0]
  for (let i = 0; i < heights.value.length; i++) {
    acc.push(acc[i]! + (heights.value[i] ?? props.estimatedItemHeight))
  }
  return acc
})

const totalHeight = computed(() => {
  const p = prefixOffsets.value
  return p.length ? p[p.length - 1]! : 0
})

const visibleSlice = computed(() => {
  const n = props.items.length
  if (n === 0) {
    return { start: 0, end: 0, offsetY: 0 }
  }
  const prefixes = prefixOffsets.value
  const buf = props.buffer
  const top = scrollTop.value
  const bottom = top + viewportHeight.value

  let start = 0
  for (let i = 0; i < n; i++) {
    if (prefixes[i + 1]! > top) {
      start = i
      break
    }
  }
  let end = n
  for (let i = start; i < n; i++) {
    if (prefixes[i]! >= bottom) {
      end = i
      break
    }
  }
  const startIdx = Math.max(0, start - buf)
  const endIdx = Math.min(n, end + buf)
  return {
    start: startIdx,
    end: endIdx,
    offsetY: prefixes[startIdx] ?? 0,
  }
})

const visibleItems = computed(() => {
  const { start, end } = visibleSlice.value
  return props.items.slice(start, end).map((item, i) => ({
    item,
    index: start + i,
  }))
})

const innerStyle = computed(
  (): CSSProperties => ({
    height: `${totalHeight.value}px`,
    position: 'relative',
    width: '100%',
  }),
)

const windowStyle = computed(
  (): CSSProperties => ({
    transform: `translateY(${visibleSlice.value.offsetY}px)`,
    willChange: 'transform',
  }),
)

function distanceFromBottom(): number {
  return totalHeight.value - (scrollTop.value + viewportHeight.value)
}

function onScroll() {
  const el = rootRef.value
  if (!el)
    return
  scrollTop.value = el.scrollTop
  emit('scroll', el.scrollTop)
  updatePinnedFromDistance()
}

function updatePinnedFromDistance() {
  const dist = distanceFromBottom()
  if (dist > PIN_UP_THRESHOLD_PX) {
    userPinnedUp.value = true
  }
  else if (dist <= PIN_DOWN_THRESHOLD_PX) {
    userPinnedUp.value = false
  }
}

function onWheel(e: WheelEvent) {
  // User intent: scrolling up to read history must not fight auto stick-to-bottom.
  if (e.deltaY < 0) {
    userPinnedUp.value = true
  }
}

function measureViewport() {
  const el = rootRef.value
  if (!el)
    return
  viewportHeight.value = el.clientHeight
}

function setItemHeight(index: number, height: number) {
  if (index < 0 || index >= props.items.length)
    return
  const h = Math.max(1, Math.ceil(height))
  const prev = heights.value[index] ?? props.estimatedItemHeight
  if (prev === h)
    return
  const topOffset = prefixOffsets.value[index] ?? 0
  const anchorAboveViewport = topOffset < scrollTop.value
  heights.value[index] = h
  if (!anchorAboveViewport)
    return
  const el = rootRef.value
  if (!el)
    return
  const delta = h - prev
  if (delta === 0)
    return
  el.scrollTop += delta
  scrollTop.value = el.scrollTop
}

function observeRow(el: HTMLElement | null, index: number) {
  const prev = resizeObservers.get(index)
  if (prev) {
    prev.disconnect()
    resizeObservers.delete(index)
  }
  if (!el)
    return
  const ro = new ResizeObserver((entries) => {
    const entry = entries[0]
    if (entry) {
      setItemHeight(index, entry.contentRect.height)
    }
  })
  ro.observe(el)
  resizeObservers.set(index, ro)
  setItemHeight(index, el.getBoundingClientRect().height)
}

async function scrollToBottom(force = false): Promise<void> {
  await nextTick()
  const el = rootRef.value
  if (!el)
    return
  if (!force && userPinnedUp.value)
    return
  el.scrollTop = totalHeight.value
  scrollTop.value = el.scrollTop
  userPinnedUp.value = false
}

watch(
  () => props.items.length,
  async (len, prev) => {
    if (!props.stickToBottom || len <= prev)
      return
    if (userPinnedUp.value)
      return
    await scrollToBottom(false)
  },
  { flush: 'post' },
)

watch(
  () => {
    const n = props.items.length
    if (n === 0)
      return null
    return keyOf(props.items[n - 1]!, n - 1)
  },
  async (id, prevId) => {
    if (id == null || id === prevId)
      return
    if (!props.stickToBottom || userPinnedUp.value)
      return
    await scrollToBottom(false)
  },
  { flush: 'post' },
)

let viewportObserver: ResizeObserver | null = null

onMounted(() => {
  measureViewport()
  const el = rootRef.value
  if (el) {
    viewportObserver = new ResizeObserver(() => {
      measureViewport()
    })
    viewportObserver.observe(el)
  }
})

onBeforeUnmount(() => {
  viewportObserver?.disconnect()
  for (const ro of resizeObservers.values()) {
    ro.disconnect()
  }
  resizeObservers.clear()
})

defineExpose({ scrollToBottom, userPinnedUp })
</script>

<template>
  <div
    ref="rootRef"
    class="virtual-scroll-root"
    @scroll.passive="onScroll"
    @wheel.passive="onWheel"
  >
    <div class="virtual-scroll-inner" :style="innerStyle">
      <div class="virtual-scroll-window" :style="windowStyle">
        <div
          v-for="{ item, index } in visibleItems"
          :key="keyOf(item, index)"
          :ref="(el) => observeRow(el as HTMLElement | null, index)"
          class="virtual-scroll-row"
          :data-index="index"
        >
          <slot :item="item" :index="index" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.virtual-scroll-root {
  overflow-y: auto;
  overflow-x: hidden;
  height: 100%;
  min-height: 0;
  position: relative;
  overscroll-behavior: contain;
  overflow-anchor: none;
}
.virtual-scroll-inner {
  width: 100%;
}
.virtual-scroll-window {
  width: 100%;
}
.virtual-scroll-row {
  width: 100%;
}
</style>
