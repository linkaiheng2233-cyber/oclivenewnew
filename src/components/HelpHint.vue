<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    /** 点击后显示的说明；双换行分段 */
    text?: string;
    paragraphs?: readonly string[];
    /** 气泡相对按钮：向左展开或向右展开，贴右缘时用 end 避免裁切 */
    popAlign?: "start" | "end";
    /** 更小问号与更窄气泡，适合顶栏等窄区域 */
    compact?: boolean;
  }>(),
  { popAlign: "start", compact: false },
);

const segments = computed(() => {
  if (props.paragraphs?.length) {
    return props.paragraphs.map((s) => s.trim()).filter(Boolean);
  }
  const t = props.text?.trim() ?? "";
  if (!t) return [];
  return t
    .split(/\n\n+/)
    .map((s) => s.trim())
    .filter(Boolean);
});

const open = ref(false);
const root = ref<HTMLElement | null>(null);

function toggle(e: Event) {
  e.stopPropagation();
  open.value = !open.value;
}

/** capture：先于子树内 @click.stop，点在「更多」等面板空白处也能收到 */
function onDocPointerDownCapture(e: PointerEvent) {
  if (!open.value) return;
  const el = root.value;
  if (el && !el.contains(e.target as Node)) open.value = false;
}

function onDocKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") open.value = false;
}

function onWindowResize() {
  if (open.value) open.value = false;
}

const CAPTURE_OPTS = true;

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointerDownCapture, CAPTURE_OPTS);
  document.addEventListener("keydown", onDocKeydown);
});

watch(open, (isOpen) => {
  if (isOpen) {
    window.addEventListener("resize", onWindowResize);
  } else {
    window.removeEventListener("resize", onWindowResize);
  }
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointerDownCapture, CAPTURE_OPTS);
  document.removeEventListener("keydown", onDocKeydown);
  window.removeEventListener("resize", onWindowResize);
});
</script>

<template>
  <span
    ref="root"
    class="help-hint"
    :class="{ 'help-hint--open': open, 'help-hint--compact': compact }"
  >
    <button
      type="button"
      class="help-btn"
      :aria-expanded="open"
      aria-label="查看说明"
      @click="toggle"
    >
      ?
    </button>
    <div
      v-if="open && segments.length"
      class="help-pop"
      :class="{
        'help-pop--end': popAlign === 'end',
        'help-pop--compact': compact,
      }"
      role="tooltip"
    >
      <p v-for="(seg, i) in segments" :key="i" class="help-pop-p">{{ seg }}</p>
    </div>
  </span>
</template>

<style scoped>
.help-hint {
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
  margin-left: 0.25rem;
  position: relative;
  z-index: 900;
}

.help-hint.help-hint--open {
  z-index: 980;
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
  position: absolute;
  left: 0;
  top: calc(100% + 8px);
  z-index: 901;
  min-width: min(18rem, calc(100vw - 2rem));
  max-width: min(32rem, calc(100vw - 1.5rem));
  padding: 0.65rem 0.9rem;
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.55;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--card-bg) 92%, transparent);
  backdrop-filter: blur(10px) saturate(106%);
  -webkit-backdrop-filter: blur(10px) saturate(106%);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
  max-height: min(70vh, 24rem);
  overflow-y: auto;
}

.help-pop-p {
  margin: 0 0 0.5rem;
}

.help-pop-p:last-child {
  margin-bottom: 0;
}

.help-pop--end {
  left: auto;
  right: 0;
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
