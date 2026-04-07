<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import TimeDial from "./TimeDial.vue";
import { getTimeState, type JumpTimeResponse } from "../utils/tauri-api";

const props = withDefaults(
  defineProps<{ roleId: string; /** 顶栏单行：隐藏「虚拟时间」文案 */ compact?: boolean }>(),
  { compact: false },
);
const emit = defineEmits<{
  notify: [{ type: "success" | "error" | "info" | "warning"; message: string }];
  refreshed: [];
  jumpComplete: [JumpTimeResponse];
}>();

const displayLabel = ref("—");
const loading = ref(false);
const dialOpen = ref(false);
/** 与后端对齐的虚拟时间戳，供拨盘使用 */
const virtualTimeMs = ref(0);

async function loadState() {
  if (!props.roleId) return;
  loading.value = true;
  try {
    const s = await getTimeState(props.roleId);
    virtualTimeMs.value = s.virtual_time_ms;
    displayLabel.value = new Date(s.virtual_time_ms).toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch (e) {
    displayLabel.value = "—";
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  } finally {
    loading.value = false;
  }
}

const dialMs = computed(() =>
  virtualTimeMs.value > 0 ? virtualTimeMs.value : Date.now(),
);

function onDialRefreshed() {
  loadState();
  emit("refreshed");
}

onMounted(() => {
  loadState();
});

watch(
  () => props.roleId,
  () => {
    loadState();
  },
);
</script>

<template>
  <div class="vtime" :class="{ 'vtime--compact': compact }">
    <span v-if="!compact" class="label">虚拟时间</span>
    <span v-else class="label-icon" aria-hidden="true">⏰</span>
    <button
      type="button"
      class="time-display"
      :disabled="loading"
      @click="dialOpen = true"
    >
      {{ loading ? "…" : displayLabel }}
    </button>
    <TimeDial
      :open="dialOpen"
      :role-id="roleId"
      :virtual-time-ms="dialMs"
      @update:open="(v: boolean) => (dialOpen = v)"
      @notify="emit('notify', $event)"
      @jump-complete="emit('jumpComplete', $event)"
      @refreshed="onDialRefreshed"
    />
  </div>
</template>

<style scoped>
.vtime {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-secondary);
}
.label {
  color: var(--text-primary);
  font-weight: 600;
}
.time-display {
  min-width: 140px;
  text-align: left;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 6px 10px;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
}
.time-display:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.vtime--compact {
  padding: 0;
  flex-wrap: nowrap;
  gap: 6px;
}
.vtime--compact .time-display {
  min-width: 0;
  max-width: 160px;
  font-size: 11px;
  padding: 4px 8px;
}
.label-icon {
  font-size: 14px;
  line-height: 1;
}
</style>
