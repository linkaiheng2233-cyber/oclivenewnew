<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { RpcHistoryItem } from "../composables/usePluginDebug";
import { usePluginDebug } from "../composables/usePluginDebug";
import { getPluginLogs, killPluginProcess } from "../utils/tauri-api";
import LogViewer from "./LogViewer.vue";
import ProcessMonitor from "./ProcessMonitor.vue";
import RpcTester from "./RpcTester.vue";

const props = withDefaults(
  defineProps<{
    pluginId: string;
    expanded: boolean;
    /** manifest 是否含 `process`；为 false 时「启动」会失败，仍可 RPC 测已运行的实例 */
    spawnSupported?: boolean;
  }>(),
  { spawnSupported: true },
);

const section = ref<"process" | "rpc" | "logs">("process");
const rpcMethod = ref("");
const rpcParams = ref("{}\n");

const pluginIdRef = computed(() => props.pluginId);
const dbg = usePluginDebug(pluginIdRef);
const { processInfo, allProcesses, methods, logs, lastResponse, busy } = dbg;

const statusLabel = computed(() => {
  if (processInfo.value) return `运行中 · PID ${processInfo.value.pid}`;
  return "未运行";
});

watch(
  () => props.expanded,
  async (v) => {
    if (v) {
      dbg.startLogPolling();
      await dbg.refreshProcess();
      try {
        await dbg.refreshMethods();
      } catch {
        /* optional */
      }
      try {
        logs.value = await getPluginLogs(props.pluginId, 500);
      } catch {
        /* ignore */
      }
    } else {
      dbg.stopLogPolling();
    }
  },
  { immediate: true },
);

watch(
  () => props.pluginId,
  async () => {
    if (props.expanded) {
      await dbg.refreshProcess();
    }
  },
);

async function onKillManaged(id: string) {
  if (id === props.pluginId) {
    await dbg.onKill();
    return;
  }
  busy.value = true;
  try {
    await killPluginProcess(id);
    await dbg.refreshProcess();
  } finally {
    busy.value = false;
  }
}

function onApplyHistory(item: RpcHistoryItem) {
  rpcMethod.value = item.method;
  rpcParams.value = item.paramsText.endsWith("\n") ? item.paramsText : `${item.paramsText}\n`;
}
</script>

<template>
  <div class="pm-dbg-root">
    <header class="pm-dbg-chrome">
      <div class="pm-dbg-chrome-left">
        <span class="pm-dbg-chrome-label">目标</span>
        <code class="pm-dbg-chrome-id">{{ pluginId }}</code>
        <span
          class="pm-dbg-chrome-status"
          :class="{ live: !!processInfo }"
          :title="statusLabel"
        >{{ statusLabel }}</span>
      </div>
      <nav class="pm-dbg-tabs" aria-label="调试分区">
        <button
          type="button"
          class="pm-dbg-tab"
          :class="{ active: section === 'process' }"
          @click="section = 'process'"
        >
          进程
        </button>
        <button
          type="button"
          class="pm-dbg-tab"
          :class="{ active: section === 'rpc' }"
          @click="section = 'rpc'"
        >
          RPC
        </button>
        <button
          type="button"
          class="pm-dbg-tab"
          :class="{ active: section === 'logs' }"
          @click="section = 'logs'"
        >
          控制台
        </button>
      </nav>
    </header>

    <div class="pm-dbg-workspace">
      <div class="pm-dbg-editor">
        <ProcessMonitor
          v-show="section === 'process'"
          :plugin-id="pluginId"
          :spawn-supported="props.spawnSupported"
          :process-info="processInfo"
          :all-processes="allProcesses"
          :busy="busy"
          @spawn="dbg.onSpawn()"
          @kill="dbg.onKill()"
          @restart="dbg.onRestart()"
          @refresh-all="dbg.refreshProcess()"
          @kill-managed="onKillManaged"
        />
        <RpcTester
          v-show="section === 'rpc'"
          v-model:method="rpcMethod"
          v-model:params="rpcParams"
          :datalist-id="`pm-dbg-methods-${pluginId}`"
          :methods="methods"
          :busy="busy"
          :history="dbg.history"
          @discover="dbg.refreshMethods()"
          @send="dbg.runRpc(rpcMethod, rpcParams)"
          @apply-history="onApplyHistory"
        />
        <div v-show="section === 'logs'" class="pm-dbg-console-tab">
          <p class="pm-dbg-console-hint">
            下方 <strong>Output</strong> 为环形缓冲（约 1000 行），宿主轮询刷新；与主流 IDE 底部控制台类似。
          </p>
        </div>
      </div>

      <aside class="pm-dbg-inspector" aria-label="响应与日志输出">
        <section class="pm-dbg-panel">
          <div class="pm-dbg-panel-h">
            <span class="pm-dbg-panel-title">响应</span>
            <span class="pm-dbg-panel-meta">JSON-RPC</span>
          </div>
          <pre class="pm-dbg-pre">{{ lastResponse || "（尚无响应）" }}</pre>
        </section>
        <section class="pm-dbg-panel pm-dbg-panel--grow">
          <div class="pm-dbg-panel-h">
            <span class="pm-dbg-panel-title">输出</span>
            <span class="pm-dbg-panel-meta">stdout / stderr</span>
          </div>
          <LogViewer
            :lines="logs"
            @clear="dbg.onClearLogs()"
            @export="
              () => {
                const blob = new Blob([dbg.exportLogsText()], { type: 'text/plain;charset=utf-8' });
                const a = document.createElement('a');
                a.href = URL.createObjectURL(blob);
                a.download = `${pluginId}-plugin-debug.log`;
                a.click();
                URL.revokeObjectURL(a.href);
              }
            "
          />
        </section>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-root {
  display: flex;
  flex-direction: column;
  min-height: 320px;
  max-height: min(70vh, 640px);
  margin-top: 8px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
}
.pm-dbg-chrome {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-dbg-chrome-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  min-width: 0;
}
.pm-dbg-chrome-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-dbg-chrome-id {
  font-size: 12px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  padding: 3px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pm-dbg-chrome-status {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  max-width: 220px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pm-dbg-chrome-status.live {
  border-color: color-mix(in srgb, var(--success) 40%, var(--border-light));
  color: var(--success);
  background: color-mix(in srgb, var(--success) 10%, var(--bg-elevated));
}
.pm-dbg-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.pm-dbg-tab {
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
  transition: var(--control-transition, border-color 0.18s ease, background 0.18s ease);
}
.pm-dbg-tab:hover {
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-dbg-tab.active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
.pm-dbg-workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 34%);
  gap: 0;
}
@media (max-width: 820px) {
  .pm-dbg-workspace {
    grid-template-columns: 1fr;
  }
  .pm-dbg-inspector {
    border-left: none !important;
    border-top: 1px solid var(--border-light);
    max-height: 280px;
  }
}
.pm-dbg-editor {
  min-width: 0;
  min-height: 0;
  padding: 10px 12px;
  overflow: auto;
  border-right: 1px solid var(--border-light);
}
.pm-dbg-console-tab {
  padding: 4px 0 8px;
}
.pm-dbg-console-hint {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.pm-dbg-inspector {
  display: flex;
  flex-direction: column;
  gap: 0;
  min-width: 0;
  min-height: 0;
  background: var(--bg-secondary);
}
.pm-dbg-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-light);
  min-height: 0;
}
.pm-dbg-panel--grow {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-bottom: none;
  min-height: 140px;
}
.pm-dbg-panel--grow :deep(.pm-dbg-log) {
  flex: 1;
  min-height: 0;
}
.pm-dbg-panel--grow :deep(.pm-dbg-log-body) {
  max-height: none;
  flex: 1;
  min-height: 120px;
}
.pm-dbg-panel-h {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.pm-dbg-panel-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.pm-dbg-panel-meta {
  font-size: 11px;
  color: var(--text-secondary);
  font-family: ui-monospace, Menlo, Consolas, monospace;
}
.pm-dbg-pre {
  margin: 0;
  flex: 1;
  min-height: 100px;
  max-height: 200px;
  padding: 8px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.45;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-primary);
}
</style>
