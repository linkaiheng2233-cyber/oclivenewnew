import {
  onUnmounted,
  ref,
  shallowRef,
  toValue,
  watch,
  type MaybeRefOrGetter,
} from "vue";
import type { PluginProcessDebugInfo } from "../utils/tauri-api";
import {
  clearPluginLogs,
  discoverPluginMethods,
  getPluginLogs,
  killPluginProcess,
  listPluginProcesses,
  spawnPluginForTest,
  testPluginMethod,
} from "../utils/tauri-api";

const historyKey = (id: string) => `oclive_plugin_debug_rpc_history:${id}`;
const HISTORY_CAP = 50;

export interface RpcHistoryItem {
  id: string;
  method: string;
  paramsText: string;
  at: number;
}

export function usePluginDebug(pluginId: MaybeRefOrGetter<string>) {
  const processInfo = shallowRef<PluginProcessDebugInfo | null>(null);
  const allProcesses = ref<PluginProcessDebugInfo[]>([]);
  const methods = ref<string[]>([]);
  const logs = ref<string[]>([]);
  const lastResponse = ref("");
  const busy = ref(false);
  const history = ref<RpcHistoryItem[]>([]);
  let logTimer: ReturnType<typeof setInterval> | null = null;

  const pid = () => String(toValue(pluginId)).trim();

  function loadHistory(): RpcHistoryItem[] {
    const id = pid();
    if (!id) return [];
    try {
      const raw = sessionStorage.getItem(historyKey(id));
      if (!raw) return [];
      const v = JSON.parse(raw) as RpcHistoryItem[];
      return Array.isArray(v) ? v : [];
    } catch {
      return [];
    }
  }

  function refreshHistory() {
    history.value = loadHistory();
  }

  watch(
    () => toValue(pluginId),
    () => {
      refreshHistory();
    },
    { immediate: true },
  );

  function saveHistoryItem(method: string, paramsText: string) {
    const id = pid();
    if (!id) return;
    const cur = loadHistory();
    const item: RpcHistoryItem = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      method,
      paramsText,
      at: Date.now(),
    };
    cur.unshift(item);
    sessionStorage.setItem(historyKey(id), JSON.stringify(cur.slice(0, HISTORY_CAP)));
    refreshHistory();
  }

  function startLogPolling() {
    if (logTimer) return;
    logTimer = setInterval(async () => {
      const id = pid();
      if (!id) return;
      try {
        logs.value = await getPluginLogs(id, 500);
      } catch {
        /* ignore */
      }
    }, 900);
  }

  function stopLogPolling() {
    if (logTimer) {
      clearInterval(logTimer);
      logTimer = null;
    }
  }

  async function refreshProcess() {
    const id = pid();
    if (!id) return;
    try {
      const list = await listPluginProcesses();
      allProcesses.value = list;
      processInfo.value = list.find((p) => p.pluginId === id) ?? null;
    } catch {
      processInfo.value = null;
    }
  }

  async function onSpawn(configJson?: string) {
    const id = pid();
    if (!id) return;
    busy.value = true;
    try {
      processInfo.value = await spawnPluginForTest(id, configJson ?? null);
      await refreshProcess();
      await refreshMethods();
    } finally {
      busy.value = false;
    }
  }

  async function onKill() {
    const id = pid();
    if (!id) return;
    busy.value = true;
    try {
      await killPluginProcess(id);
      processInfo.value = null;
      await refreshProcess();
    } finally {
      busy.value = false;
    }
  }

  async function onRestart() {
    await onKill();
    await onSpawn();
  }

  async function refreshMethods() {
    const id = pid();
    if (!id) return;
    try {
      methods.value = await discoverPluginMethods(id);
    } catch {
      methods.value = [];
    }
  }

  async function runRpc(method: string, paramsText: string) {
    const id = pid();
    if (!id) return;
    if (!method.trim()) {
      lastResponse.value = JSON.stringify({ error: "请填写 RPC 方法名" }, null, 2);
      return;
    }
    const t0 = performance.now();
    let params: unknown = {};
    try {
      params = paramsText.trim() ? JSON.parse(paramsText) : {};
    } catch {
      lastResponse.value = JSON.stringify({ error: "参数不是合法 JSON" }, null, 2);
      return;
    }
    busy.value = true;
    try {
      const res = await testPluginMethod(id, method, params);
      const ms = Math.round(performance.now() - t0);
      lastResponse.value = JSON.stringify(
        { ok: true, durationMs: ms, result: res },
        null,
        2,
      );
      saveHistoryItem(method, paramsText);
    } catch (e) {
      const ms = Math.round(performance.now() - t0);
      lastResponse.value = JSON.stringify(
        {
          ok: false,
          durationMs: ms,
          error: e instanceof Error ? e.message : String(e),
        },
        null,
        2,
      );
    } finally {
      busy.value = false;
    }
  }

  async function onClearLogs() {
    const id = pid();
    if (!id) return;
    await clearPluginLogs(id);
    logs.value = [];
  }

  onUnmounted(() => stopLogPolling());

  return {
    processInfo,
    allProcesses,
    methods,
    logs,
    lastResponse,
    busy,
    history,
    loadHistory,
    startLogPolling,
    stopLogPolling,
    refreshProcess,
    refreshMethods,
    onSpawn,
    onKill,
    onRestart,
    runRpc,
    onClearLogs,
    exportLogsText: () => logs.value.join("\n"),
  };
}
