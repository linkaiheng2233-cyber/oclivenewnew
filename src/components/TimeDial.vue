<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { jumpTime, type JumpTimeResponse } from "../utils/tauri-api";
import { useChatStore } from "../stores/chatStore";
import { useUiStore } from "../stores/uiStore";

const props = defineProps<{
  open: boolean;
  roleId: string;
  /** 当前虚拟时间（毫秒），用于默认刻度与日期部分 */
  virtualTimeMs: number;
}>();

const emit = defineEmits<{
  "update:open": [boolean];
  notify: [{ type: "success" | "error" | "info" | "warning"; message: string }];
  refreshed: [];
  jumpComplete: [JumpTimeResponse];
}>();

const chatStore = useChatStore();
const uiStore = useUiStore();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const size = 240;
const cx = size / 2;
const cy = size / 2;
const rOuter = 108;
const rInner = 72;

/** 0–1439，顶部为 0:00，顺时针 */
const previewMinutes = ref(0);
const dragging = ref(false);
const showConfirm = ref(false);
const applying = ref(false);

let activePointerId: number | null = null;

function minutesFromVirtualMs(ms: number): number {
  if (!ms) return 0;
  const d = new Date(ms);
  return d.getHours() * 60 + d.getMinutes();
}

function targetTimestampMs(minutes: number): number {
  const base = props.virtualTimeMs > 0 ? props.virtualTimeMs : Date.now();
  const d = new Date(base);
  d.setHours(0, 0, 0, 0);
  return d.getTime() + minutes * 60 * 1000;
}

const previewLabel = computed(() => {
  const m = previewMinutes.value;
  const h = Math.floor(m / 60);
  const min = m % 60;
  return `${String(h).padStart(2, "0")}:${String(min).padStart(2, "0")}`;
});

function angleToMinutes(degFromEastCCW: number): number {
  // atan2: 0 = 东；我们希望北（-90°）= 0:00
  let t = degFromEastCCW + Math.PI / 2;
  while (t < 0) t += Math.PI * 2;
  while (t >= Math.PI * 2) t -= Math.PI * 2;
  const raw = (t / (Math.PI * 2)) * 1440;
  return Math.round(raw) % 1440;
}

function minutesToAngleRad(m: number): number {
  const t = (m / 1440) * Math.PI * 2;
  return t - Math.PI / 2;
}

function getCoords(e: PointerEvent, el: HTMLElement): { x: number; y: number } {
  const rect = el.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  return { x, y };
}

function pointerToMinutes(e: PointerEvent, el: HTMLElement): number {
  const { x, y } = getCoords(e, el);
  const dx = x - cx;
  const dy = y - cy;
  return angleToMinutes(Math.atan2(dy, dx));
}

function cssColor(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return v || fallback;
}

function draw() {
  const c = canvasRef.value;
  if (!c) return;
  const ctx = c.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  c.width = size * dpr;
  c.height = size * dpr;
  c.style.width = `${size}px`;
  c.style.height = `${size}px`;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  ctx.clearRect(0, 0, size, size);

  const cardBg = cssColor("--card-bg", "#1e1e24");
  const bg2 = cssColor("--bg-secondary", "#25252c");
  const border = cssColor("--border-light", "#2c2c34");
  const textPri = cssColor("--text-primary", "#e0e0e0");
  const textSec = cssColor("--text-secondary", "#9a9aae");
  const accent = cssColor("--accent", "#d4a574");

  // 外圈
  ctx.beginPath();
  ctx.arc(cx, cy, rOuter, 0, Math.PI * 2);
  const grad = ctx.createLinearGradient(0, 0, size, size);
  grad.addColorStop(0, cardBg);
  grad.addColorStop(1, bg2);
  ctx.fillStyle = grad;
  ctx.fill();
  ctx.strokeStyle = border;
  ctx.lineWidth = 2;
  ctx.stroke();

  // 刻度（每小时大刻度，每 15 分钟小刻度）
  for (let i = 0; i < 1440; i += 15) {
    const a = minutesToAngleRad(i);
    const major = i % 60 === 0;
    const r1 = major ? rInner - 4 : rInner + 2;
    const r2 = rOuter - (major ? 6 : 10);
    ctx.beginPath();
    ctx.moveTo(cx + Math.cos(a) * r1, cy + Math.sin(a) * r1);
    ctx.lineTo(cx + Math.cos(a) * r2, cy + Math.sin(a) * r2);
    ctx.strokeStyle = major ? textPri : textSec;
    ctx.lineWidth = major ? 2 : 1;
    ctx.stroke();
  }

  // 小时数字（0,3,6…21）
  ctx.font = "600 11px system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = textSec;
  for (let h = 0; h < 24; h += 3) {
    const a = minutesToAngleRad(h * 60);
    const tx = cx + Math.cos(a) * (rInner - 18);
    const ty = cy + Math.sin(a) * (rInner - 18);
    ctx.fillText(String(h), tx, ty);
  }

  // 选中指针
  const aSel = minutesToAngleRad(previewMinutes.value);
  ctx.beginPath();
  ctx.moveTo(cx, cy);
  ctx.lineTo(
    cx + Math.cos(aSel) * (rOuter - 12),
    cy + Math.sin(aSel) * (rOuter - 12),
  );
  ctx.strokeStyle = accent;
  ctx.lineWidth = 3;
  ctx.lineCap = "round";
  ctx.stroke();

  ctx.beginPath();
  ctx.arc(cx, cy, 8, 0, Math.PI * 2);
  ctx.fillStyle = accent;
  ctx.fill();
}

function onCanvasPointerDown(e: PointerEvent) {
  if (!canvasRef.value || applying.value) return;
  e.preventDefault();
  dragging.value = true;
  showConfirm.value = false;
  activePointerId = e.pointerId;
  canvasRef.value.setPointerCapture(e.pointerId);
  previewMinutes.value = pointerToMinutes(e, canvasRef.value);
  draw();
}

function onCanvasPointerMove(e: PointerEvent) {
  if (!dragging.value || !canvasRef.value) return;
  previewMinutes.value = pointerToMinutes(e, canvasRef.value);
  draw();
}

function onCanvasPointerUp(e: PointerEvent) {
  if (!canvasRef.value) return;
  if (activePointerId === e.pointerId) {
    canvasRef.value.releasePointerCapture(e.pointerId);
    activePointerId = null;
  }
  if (dragging.value) {
    dragging.value = false;
    showConfirm.value = true;
  }
}

function close() {
  showConfirm.value = false;
  emit("update:open", false);
}

function cancelPick() {
  showConfirm.value = false;
  previewMinutes.value = minutesFromVirtualMs(props.virtualTimeMs);
  draw();
  close();
}

async function confirmPick() {
  await doJump(targetTimestampMs(previewMinutes.value), undefined);
}

async function doJump(
  ts?: number,
  preset?: "+2h" | "+6h" | "next_morning" | "skip_idle_time",
) {
  if (!props.roleId || applying.value) return;
  applying.value = true;
  try {
    const res = await jumpTime(props.roleId, ts, preset);
    for (const line of res.monologues ?? []) {
      if (line.trim()) {
        chatStore.addAssistantMessage(
          line.trim(),
          undefined,
          uiStore.sceneId,
        );
      }
    }
    emit("jumpComplete", res);
    emit("refreshed");
    emit("notify", { type: "success", message: "虚拟时间已更新" });
    if (Math.abs(res.favorability_delta) > 1e-5) {
      const sign = res.favorability_delta > 0 ? "+" : "";
      emit("notify", {
        type: "info",
        message: `好感度变化 ${sign}${res.favorability_delta.toFixed(2)}（当前 ${res.favorability_current.toFixed(1)}）`,
      });
    }
    showConfirm.value = false;
    emit("update:open", false);
  } catch (err) {
    emit("notify", {
      type: "error",
      message: err instanceof Error ? err.message : String(err),
    });
  } finally {
    applying.value = false;
  }
}

async function applyPreset(preset: "+2h" | "+6h" | "next_morning" | "skip_idle_time") {
  await doJump(undefined, preset);
}

watch(
  () => props.open,
  async (v) => {
    if (v) {
      previewMinutes.value = minutesFromVirtualMs(
        props.virtualTimeMs > 0 ? props.virtualTimeMs : Date.now(),
      );
      showConfirm.value = false;
      dragging.value = false;
      await nextTick();
      draw();
    }
  },
);

watch(
  () => [props.virtualTimeMs, props.open] as const,
  () => {
    if (props.open) {
      draw();
    }
  },
);

watch(previewMinutes, () => {
  if (props.open) draw();
});

onBeforeUnmount(() => {
  dragging.value = false;
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="backdrop"
      role="dialog"
      aria-modal="true"
      @click.self="cancelPick"
    >
      <div class="panel card" @click.stop>
        <div class="title">调节虚拟时间</div>
        <p class="hint">拖拽圆盘选择时刻（精确到分钟），松开后确认</p>
        <canvas
          ref="canvasRef"
          class="dial"
          :width="size"
          :height="size"
          @pointerdown="onCanvasPointerDown"
          @pointermove="onCanvasPointerMove"
          @pointerup="onCanvasPointerUp"
          @pointercancel="onCanvasPointerUp"
        />
        <div class="preview">{{ previewLabel }}</div>
        <div class="preset-actions">
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('+2h')">+2h</button>
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('+6h')">+6h</button>
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('next_morning')">次日早晨</button>
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('skip_idle_time')">跳过空窗</button>
        </div>
        <div v-if="showConfirm" class="actions">
          <button
            type="button"
            class="btn ghost"
            :disabled="applying"
            @click="cancelPick"
          >
            取消
          </button>
          <button
            type="button"
            class="btn primary"
            :disabled="applying"
            @click="confirmPick"
          >
            {{ applying ? "…" : "确认" }}
          </button>
        </div>
        <button type="button" class="close-x" aria-label="关闭" @click="cancelPick">
          ×
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 1200;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.panel {
  position: relative;
  padding: 20px 24px 16px;
  max-width: 92vw;
  box-shadow: var(--shadow-sm, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.title {
  font-weight: 700;
  font-size: 16px;
  color: var(--text-primary);
  margin-bottom: 6px;
}
.hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0 0 12px;
}
.dial {
  display: block;
  margin: 0 auto;
  cursor: grab;
  touch-action: none;
  border-radius: 50%;
}
.dial:active {
  cursor: grabbing;
}
.preview {
  text-align: center;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-primary);
  margin-top: 4px;
}
.actions {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 14px;
}
.preset-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
  margin-top: 10px;
}
.btn {
  border: none;
  border-radius: 16px;
  padding: 8px 18px;
  font-size: 14px;
  cursor: pointer;
}
.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.btn.primary {
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  border: 1px solid var(--border-light);
  color: var(--text-accent);
}
.btn.ghost {
  background: var(--bubble-bot-bg);
  color: var(--text-primary);
}
.close-x {
  position: absolute;
  top: 10px;
  right: 12px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  padding: 4px 8px;
}
.close-x:hover {
  color: var(--text-primary);
}
</style>
