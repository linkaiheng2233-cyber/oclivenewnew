<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { RpcHistoryItem } from "../composables/usePluginDebug";
import { usePluginDebug } from "../composables/usePluginDebug";
import { getPluginLogs, killPluginProcess } from "../utils/tauri-api";
import LogViewer from "./LogViewer.vue";
import ProcessMonitor from "./ProcessMonitor.vue";
import RpcTester from "./RpcTester.vue";

const props = defineProps<{
  pluginId: string;
  expanded: boolean;
}>();

const section = ref<"process" | "rpc" | "logs">("process");
const rpcMethod = ref("");
const rpcParams = ref("{}\n");

const pluginIdRef = computed(() => props.pluginId);
const dbg = usePluginDebug(pluginIdRef);
const { processInfo, allProcesses, methods, logs, lastResponse, busy } = dbg;

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
    <nav class="pm-dbg-nav" aria-label="调试分区">
      <button
        type="button"
        class="pm-dbg-nav-btn"
        :class="{ active: section === 'process' }"
        @click="section = 'process'"
      >
        进程状态
      </button>
      <button
        type="button"
        class="pm-dbg-nav-btn"
        :class="{ active: section === 'rpc' }"
        @click="section = 'rpc'"
      >
        RPC 测试
      </button>
      <button
        type="button"
        class="pm-dbg-nav-btn"
        :class="{ active: section === 'logs' }"
        @click="section = 'logs'"
      >
        日志
      </button>
    </nav>
    <div class="pm-dbg-mid">
      <ProcessMonitor
        v-show="section === 'process'"
        :plugin-id="pluginId"
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
        :history="dbg.loadHistory()"
        @discover="dbg.refreshMethods()"
        @send="dbg.runRpc(rpcMethod, rpcParams)"
        @apply-history="onApplyHistory"
      />
      <p v-show="section === 'logs'" class="pm-dbg-hint">
        日志由宿主环形缓冲提供（约 1000 行）；轮询刷新。
      </p>
    </div>
    <div class="pm-dbg-right">
      <div class="pm-dbg-resp">
        <div class="pm-dbg-resp-h">响应</div>
        <pre class="pm-dbg-pre">{{ lastResponse || "（尚无 RPC 响应）" }}</pre>
      </div>
      <div class="pm-dbg-resp log-block">
        <div class="pm-dbg-resp-h">实时日志</div>
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
      </div>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-root {
  display: grid;
  grid-template-columns: minmax(140px, 180px) minmax(0, 1fr) minmax(200px, 280px);
  gap: 10px;
  margin-top: 10px;
  padding: 10px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border-light) 80%, var(--accent) 15%);
  background: color-mix(in srgb, var(--bg-primary) 94%, var(--accent-soft) 6%);
  min-height: 280px;
}
@media (max-width: 900px) {
  .pm-dbg-root {
    grid-template-columns: 1fr;
  }
}
.pm-dbg-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.pm-dbg-nav-btn {
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  color: var(--text-secondary);
}
.pm-dbg-nav-btn.active {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-weight: 600;
}
.pm-dbg-mid {
  min-width: 0;
  min-height: 200px;
}
.pm-dbg-hint {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-dbg-right {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  min-height: 0;
}
.pm-dbg-resp {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
}
.pm-dbg-resp.log-block {
  flex: 1;
  min-height: 120px;
}
.pm-dbg-resp-h {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-secondary);
}
.pm-dbg-pre {
  margin: 0;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 11px;
  max-height: 160px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
