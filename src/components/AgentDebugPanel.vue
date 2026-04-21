<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  callMcpTool,
  clearAgentDebugTraces,
  getAgentDebugTraces,
  listMcpServers,
  type AgentDebugTrace,
  type McpServerManifest,
} from "../utils/tauri-api";

const busy = ref(false);
const servers = ref<McpServerManifest[]>([]);
const traces = ref<AgentDebugTrace[]>([]);
const selectedServerId = ref("");
const selectedToolName = ref("");
const paramsText = ref('{"city":"深圳"}');
const callResultText = ref("");

async function refreshServers(): Promise<void> {
  servers.value = await listMcpServers();
  if (!selectedServerId.value && servers.value.length > 0) {
    selectedServerId.value = servers.value[0].id;
  }
}

async function refreshTraces(): Promise<void> {
  traces.value = await getAgentDebugTraces();
}

async function runToolCall(): Promise<void> {
  if (!selectedServerId.value || !selectedToolName.value) return;
  busy.value = true;
  try {
    const params = paramsText.value.trim() ? JSON.parse(paramsText.value) : {};
    const r = await callMcpTool(
      selectedServerId.value,
      selectedToolName.value,
      params,
    );
    callResultText.value = JSON.stringify(r.result, null, 2);
    await refreshTraces();
  } finally {
    busy.value = false;
  }
}

async function clearTraces(): Promise<void> {
  busy.value = true;
  try {
    await clearAgentDebugTraces();
    traces.value = [];
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  busy.value = true;
  try {
    await Promise.all([refreshServers(), refreshTraces()]);
  } finally {
    busy.value = false;
  }
});
</script>

<template>
  <section class="adp">
    <h3 class="adp-h">Agent 调试链路</h3>
    <p class="adp-sub">
      可查看 MCP Server、手动调用工具，并查看最近 Agent 任务拆解与工具调用轨迹。
    </p>

    <div class="adp-row">
      <button type="button" class="adp-btn" :disabled="busy" @click="refreshServers">
        刷新 MCP Server
      </button>
      <button type="button" class="adp-btn" :disabled="busy" @click="refreshTraces">
        刷新 Agent Trace
      </button>
      <button type="button" class="adp-btn danger" :disabled="busy" @click="clearTraces">
        清空 Trace
      </button>
    </div>

    <div class="adp-form">
      <label>
        Server
        <select v-model="selectedServerId" class="adp-input">
          <option value="">请选择</option>
          <option v-for="s in servers" :key="s.id" :value="s.id">
            {{ s.id }} ({{ s.transport || "http" }})
          </option>
        </select>
      </label>
      <label>
        Tool
        <input
          v-model="selectedToolName"
          class="adp-input"
          type="text"
          placeholder="例如 get_weather"
        />
      </label>
      <label>
        Params(JSON)
        <textarea v-model="paramsText" class="adp-textarea" rows="3" />
      </label>
      <button type="button" class="adp-btn primary" :disabled="busy" @click="runToolCall">
        调用工具
      </button>
    </div>

    <pre v-if="callResultText" class="adp-pre">{{ callResultText }}</pre>

    <div class="adp-traces">
      <h4 class="adp-h4">最近任务</h4>
      <div v-for="(t, i) in traces.slice().reverse()" :key="`${t.timestamp_ms}-${i}`" class="adp-trace">
        <div class="adp-trace-line">
          <strong>{{ new Date(t.timestamp_ms).toLocaleString() }}</strong> · {{ t.message }}
        </div>
        <div class="adp-trace-line">plan: {{ t.plan }}</div>
        <div class="adp-trace-line">reply: {{ t.reply }}</div>
        <div v-if="t.error" class="adp-trace-line err">error: {{ t.error }}</div>
        <pre v-if="t.tool_calls?.length" class="adp-pre">{{ JSON.stringify(t.tool_calls, null, 2) }}</pre>
      </div>
      <p v-if="traces.length === 0" class="adp-empty">暂无 Agent 执行轨迹。</p>
    </div>
  </section>
</template>

<style scoped>
.adp {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px dashed var(--border-light);
}
.adp-h {
  margin: 0;
  font-size: 14px;
}
.adp-sub {
  margin: 6px 0 10px;
  font-size: 12px;
  color: var(--text-secondary);
}
.adp-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 10px;
}
.adp-form {
  display: grid;
  gap: 8px;
}
.adp-input,
.adp-textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.adp-btn {
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  border-radius: 8px;
  padding: 6px 10px;
  cursor: pointer;
}
.adp-btn.primary {
  background: var(--accent);
  color: #fff;
}
.adp-btn.danger {
  color: var(--error);
}
.adp-pre {
  margin: 8px 0 0;
  padding: 8px 10px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--panel-bg-soft);
  white-space: pre-wrap;
  word-break: break-word;
}
.adp-traces {
  margin-top: 10px;
}
.adp-h4 {
  margin: 0 0 8px;
  font-size: 13px;
}
.adp-trace {
  border: 1px solid var(--border-light);
  border-radius: 8px;
  padding: 8px;
  margin-bottom: 8px;
}
.adp-trace-line {
  font-size: 12px;
  line-height: 1.4;
}
.adp-trace-line.err {
  color: var(--error);
}
.adp-empty {
  margin: 0;
  color: var(--text-secondary);
  font-size: 12px;
}
</style>
