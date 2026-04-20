<script setup lang="ts">
import type { PluginProcessDebugInfo } from "../utils/tauri-api";

defineProps<{
  pluginId: string;
  processInfo: PluginProcessDebugInfo | null;
  allProcesses: PluginProcessDebugInfo[];
  busy: boolean;
}>();

const emit = defineEmits<{
  spawn: [];
  kill: [];
  restart: [];
  refreshAll: [];
  killManaged: [id: string];
}>();
</script>

<template>
  <div class="pm-dbg-proc">
    <div class="pm-dbg-proc-row">
      <span class="pm-dbg-label">本插件</span>
      <span v-if="processInfo" class="pm-dbg-pill ok">
        运行中 · PID {{ processInfo.pid }}
      </span>
      <span v-else class="pm-dbg-pill">未启动</span>
    </div>
    <p v-if="processInfo" class="pm-dbg-mono">{{ processInfo.rpcUrl }}</p>
    <div class="pm-dbg-actions">
      <button type="button" class="pm-dbg-btn" :disabled="busy" @click="emit('spawn')">
        启动
      </button>
      <button type="button" class="pm-dbg-btn" :disabled="busy || !processInfo" @click="emit('kill')">
        停止
      </button>
      <button type="button" class="pm-dbg-btn" :disabled="busy || !processInfo" @click="emit('restart')">
        重启
      </button>
      <button type="button" class="pm-dbg-btn secondary" :disabled="busy" @click="emit('refreshAll')">
        刷新进程列表
      </button>
    </div>
    <div v-if="allProcesses.length" class="pm-dbg-global">
      <div class="pm-dbg-sub">宿主管理的插件进程</div>
      <ul class="pm-dbg-plist">
        <li v-for="p in allProcesses" :key="p.pluginId" class="pm-dbg-pli">
          <span class="pm-dbg-mono">{{ p.pluginId }}</span>
          <span class="pm-dbg-muted">PID {{ p.pid }}</span>
          <button
            type="button"
            class="pm-dbg-btn danger"
            :disabled="busy"
            @click="emit('killManaged', p.pluginId)"
          >
            终止
          </button>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-proc {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12px;
}
.pm-dbg-proc-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm-dbg-label {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-dbg-pill {
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-dbg-pill.ok {
  border-color: color-mix(in srgb, #16a34a 40%, var(--border-light));
  color: color-mix(in srgb, #166534 90%, var(--text-primary));
}
.pm-dbg-mono {
  margin: 0;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
  color: var(--text-secondary);
}
.pm-dbg-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.pm-dbg-btn {
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  cursor: pointer;
}
.pm-dbg-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.pm-dbg-btn.secondary {
  background: transparent;
}
.pm-dbg-btn.danger {
  border-color: color-mix(in srgb, #dc2626 35%, var(--border-light));
  color: var(--text-danger, #b91c1c);
}
.pm-dbg-global {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--border-light);
}
.pm-dbg-sub {
  font-weight: 600;
  margin-bottom: 6px;
}
.pm-dbg-plist {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-dbg-pli {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm-dbg-muted {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
