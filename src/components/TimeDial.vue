<script setup lang="ts">
import { computed, ref, watch } from "vue";
import PickerWheel from "./PickerWheel.vue";
import { jumpTime, type JumpTimeResponse } from "../utils/tauri-api";
import { useChatStore } from "../stores/chatStore";
import { useUiStore } from "../stores/uiStore";

const props = defineProps<{
  open: boolean;
  roleId: string;
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

const applying = ref(false);

const pickYear = ref(2025);
const pickMonth = ref(1);
const pickDay = ref(1);
const pickHour = ref(0);
const pickMinute = ref(0);

function daysInMonth(y: number, m: number): number {
  return new Date(y, m, 0).getDate();
}

const yearItems = computed(() => {
  const base = new Date(
    props.virtualTimeMs > 0 ? props.virtualTimeMs : Date.now(),
  ).getFullYear();
  const from = base - 2;
  const to = base + 12;
  const out: { value: number; label: string }[] = [];
  for (let y = from; y <= to; y++) out.push({ value: y, label: String(y) });
  return out;
});

const monthItems = computed(() =>
  Array.from({ length: 12 }, (_, i) => ({
    value: i + 1,
    label: `${i + 1}月`,
  })),
);

const dayItems = computed(() => {
  const max = daysInMonth(pickYear.value, pickMonth.value);
  return Array.from({ length: max }, (_, i) => ({
    value: i + 1,
    label: `${i + 1}日`,
  }));
});

const hourItems = computed(() =>
  Array.from({ length: 24 }, (_, i) => ({
    value: i,
    label: String(i).padStart(2, "0"),
  })),
);

const minuteItems = computed(() =>
  Array.from({ length: 60 }, (_, i) => ({
    value: i,
    label: String(i).padStart(2, "0"),
  })),
);

watch([pickYear, pickMonth], () => {
  const max = daysInMonth(pickYear.value, pickMonth.value);
  if (pickDay.value > max) pickDay.value = max;
});

function initFromMs(ms: number) {
  const d = new Date(ms > 0 ? ms : Date.now());
  pickYear.value = d.getFullYear();
  pickMonth.value = d.getMonth() + 1;
  pickDay.value = d.getDate();
  pickHour.value = d.getHours();
  pickMinute.value = d.getMinutes();
}

const previewTimestamp = computed(() => {
  const d = new Date(
    pickYear.value,
    pickMonth.value - 1,
    pickDay.value,
    pickHour.value,
    pickMinute.value,
    0,
    0,
  );
  return d.getTime();
});

const previewLabel = computed(() =>
  new Date(previewTimestamp.value).toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }),
);

watch(
  () => props.open,
  (v) => {
    if (v) {
      initFromMs(props.virtualTimeMs > 0 ? props.virtualTimeMs : Date.now());
    }
  },
);

function close() {
  emit("update:open", false);
}

function cancelPick() {
  close();
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

async function confirmPick() {
  await doJump(previewTimestamp.value, undefined);
}

async function applyPreset(preset: "+2h" | "+6h" | "next_morning" | "skip_idle_time") {
  await doJump(undefined, preset);
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="backdrop"
      role="dialog"
      aria-modal="true"
      aria-labelledby="time-dial-title"
      @click.self="cancelPick"
    >
      <div class="panel" @click.stop>
        <h2 id="time-dial-title" class="title">调节虚拟时间</h2>
        <p class="hint">上下拨动选择日期与时刻（类似系统滚轮），精确到分钟。</p>

        <div class="wheels" role="group" aria-label="日期与时间">
          <div class="wheel-col">
            <span class="wheel-label">年</span>
            <PickerWheel v-model="pickYear" :items="yearItems" />
          </div>
          <div class="wheel-col">
            <span class="wheel-label">月</span>
            <PickerWheel v-model="pickMonth" :items="monthItems" />
          </div>
          <div class="wheel-col">
            <span class="wheel-label">日</span>
            <PickerWheel v-model="pickDay" :items="dayItems" />
          </div>
          <div class="wheel-col wheel-col--time">
            <span class="wheel-label">时</span>
            <PickerWheel v-model="pickHour" :items="hourItems" />
          </div>
          <div class="wheel-col wheel-col--time">
            <span class="wheel-label">分</span>
            <PickerWheel v-model="pickMinute" :items="minuteItems" />
          </div>
        </div>

        <div class="preview">{{ previewLabel }}</div>

        <div class="preset-actions">
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('+2h')">
            +2h
          </button>
          <button type="button" class="btn ghost" :disabled="applying" @click="applyPreset('+6h')">
            +6h
          </button>
          <button
            type="button"
            class="btn ghost"
            :disabled="applying"
            @click="applyPreset('next_morning')"
          >
            次日早晨
          </button>
          <button
            type="button"
            class="btn ghost"
            :disabled="applying"
            @click="applyPreset('skip_idle_time')"
          >
            跳过空窗
          </button>
        </div>

        <div class="actions">
          <button type="button" class="btn ghost" :disabled="applying" @click="cancelPick">
            取消
          </button>
          <button type="button" class="btn primary" :disabled="applying" @click="confirmPick">
            {{ applying ? "…" : "确认所选时间" }}
          </button>
        </div>

        <button type="button" class="close-x" aria-label="关闭" @click="cancelPick">×</button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 1200;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.45));
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.panel {
  position: relative;
  width: min(520px, 96vw);
  padding: 20px 22px 16px;
  border-radius: var(--radius-card);
  background: var(--dialog-panel-bg, var(--card-bg));
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
}

.title {
  font-weight: 700;
  font-size: 16px;
  color: var(--text-primary);
  margin: 0 0 6px;
}

.hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0 0 14px;
  line-height: 1.45;
}

.wheels {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px 10px;
  margin-bottom: 10px;
}

.wheel-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex: 1 1 4.5rem;
  min-width: 3.5rem;
  max-width: 5.5rem;
}

.wheel-col--time {
  flex: 0 0 3.75rem;
  max-width: 3.75rem;
}

.wheel-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.preview {
  text-align: center;
  font-size: 17px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
  margin: 4px 0 12px;
}

.actions {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 8px;
}

.preset-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
  margin-bottom: 10px;
}

.btn {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 8px 16px;
  font-size: 13px;
  cursor: pointer;
  font-family: var(--font-ui);
  transition: var(--control-transition);
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn.primary {
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
}

.btn.ghost {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.btn:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--border-light) 60%, var(--text-secondary) 40%);
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
  border-radius: var(--radius-btn);
}

.close-x:hover {
  color: var(--text-primary);
  background: color-mix(in srgb, var(--text-primary) 6%, transparent);
}
</style>
