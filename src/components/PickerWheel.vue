<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

export type PickerWheelItem = { value: number; label: string };

const props = defineProps<{
  modelValue: number;
  items: readonly PickerWheelItem[];
}>();

const emit = defineEmits<{ "update:modelValue": [number] }>();

const ITEM_H = 40;
const viewportRef = ref<HTMLDivElement | null>(null);
const scrollRef = ref<HTMLDivElement | null>(null);

const pad = ref(0);

function indexOfValue(v: number): number {
  const i = props.items.findIndex((x) => x.value === v);
  return i >= 0 ? i : 0;
}

function setPadFromViewport() {
  const h = viewportRef.value?.clientHeight ?? 200;
  pad.value = Math.max(0, (h - ITEM_H) / 2);
}

function scrollToIndex(i: number, behavior: ScrollBehavior = "auto") {
  const el = scrollRef.value;
  if (!el || props.items.length === 0) return;
  const clamped = Math.max(0, Math.min(props.items.length - 1, i));
  el.scrollTo({ top: clamped * ITEM_H, behavior });
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function onScroll() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(snapFromScroll, 48);
}

function snapFromScroll() {
  debounceTimer = null;
  const el = scrollRef.value;
  if (!el || props.items.length === 0) return;
  const idx = Math.round(el.scrollTop / ITEM_H);
  const clamped = Math.max(0, Math.min(props.items.length - 1, idx));
  const nextTop = clamped * ITEM_H;
  if (Math.abs(el.scrollTop - nextTop) > 0.5) {
    el.scrollTo({ top: nextTop, behavior: "smooth" });
  }
  const val = props.items[clamped]?.value;
  if (val !== undefined && val !== props.modelValue) {
    emit("update:modelValue", val);
  }
}

function onScrollEnd() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
  }
  snapFromScroll();
}

watch(
  () => props.modelValue,
  (v) => {
    const i = indexOfValue(v);
    nextTick(() => scrollToIndex(i, "auto"));
  },
);

watch(
  () => props.items,
  () => {
    nextTick(() => {
      setPadFromViewport();
      const i = indexOfValue(props.modelValue);
      scrollToIndex(i, "auto");
    });
  },
  { deep: true },
);

onMounted(() => {
  nextTick(() => {
    setPadFromViewport();
    scrollToIndex(indexOfValue(props.modelValue), "auto");
  });
});

function syncScrollPosition() {
  setPadFromViewport();
  scrollToIndex(indexOfValue(props.modelValue), "auto");
}

defineExpose({ syncScrollPosition });

onBeforeUnmount(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>

<template>
  <div ref="viewportRef" class="picker-viewport">
    <div class="picker-window" aria-hidden="true" />
    <div
      ref="scrollRef"
      class="picker-scroll"
      @scroll.passive="onScroll"
      @scrollend="onScrollEnd"
    >
      <div
        class="picker-pad"
        :style="{ paddingTop: `${pad}px`, paddingBottom: `${pad}px` }"
      >
        <button
          v-for="it in items"
          :key="it.value"
          type="button"
          class="picker-item"
          :class="{ 'picker-item--active': it.value === modelValue }"
          :style="{ height: `${ITEM_H}px` }"
          @click="
            emit('update:modelValue', it.value);
            scrollToIndex(indexOfValue(it.value), 'smooth');
          "
        >
          {{ it.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.picker-viewport {
  position: relative;
  height: 200px;
  width: 100%;
  min-width: 3.25rem;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 94%, var(--border-light) 6%);
  overflow: hidden;
}

.picker-window {
  pointer-events: none;
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  height: 40px;
  margin-top: -20px;
  border-top: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  background: color-mix(in srgb, var(--text-primary) 3%, transparent);
  z-index: 1;
}

.picker-scroll {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: none;
}

.picker-scroll::-webkit-scrollbar {
  display: none;
}

.picker-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  margin: 0;
  padding: 0 6px;
  border: none;
  background: transparent;
  font-size: 15px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
  box-sizing: border-box;
}

.picker-item:hover {
  color: var(--text-primary);
}

.picker-item--active {
  color: var(--text-primary);
  font-weight: 600;
}
</style>
