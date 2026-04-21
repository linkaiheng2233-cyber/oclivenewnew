<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  callMcpTool,
  clearAgentDebugTraces,
  getAgentDebugTraces,
  listMcpTools,
  listMcpServers,
  type AgentDebugTrace,
  type McpToolManifest,
  type McpServerManifest,
} from "../utils/tauri-api";
import EnvVarManager from "./EnvVarManager.vue";

const busy = ref(false);
const servers = ref<McpServerManifest[]>([]);
const traces = ref<AgentDebugTrace[]>([]);
const selectedServerId = ref("");
const selectedToolName = ref("");
const paramsText = ref('{"city":"深圳"}');
const callResultText = ref("");
const availableTools = ref<McpToolManifest[]>([]);
const callResultHistory = ref<Array<{ id: string; label: string; payload: string }>>([]);
const compareLeftId = ref("");
const compareRightId = ref("");
const diffText = ref("");

const selectedTemplateId = ref("");
const customTemplateName = ref("");
const CUSTOM_TEMPLATE_KEY = "oclive.agent.templates.v1";
type TemplateItem = {
  id: string;
  label: string;
  toolName: string;
  params: Record<string, unknown>;
  serverHint?: string;
};
const templates = ref<TemplateItem[]>([
  { id: "weather", label: "天气查询", toolName: "get_weather", params: { city: "北京" } },
  { id: "file-read", label: "文件读取", toolName: "read_file", params: { path: "./README.md" } },
  { id: "web-fetch", label: "网页抓取", toolName: "web_fetch", params: { url: "https://example.com" } },
]);

async function refreshServers(): Promise<void> {
  servers.value = await listMcpServers();
  if (!selectedServerId.value && servers.value.length > 0) {
    selectedServerId.value = servers.value[0].id;
  }
  if (selectedServerId.value) {
    availableTools.value = await listMcpTools(selectedServerId.value);
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
    const id = `${Date.now()}`;
    const label = `${new Date().toLocaleTimeString()} ${selectedServerId.value}.${selectedToolName.value}`;
    callResultHistory.value.unshift({ id, label, payload: callResultText.value });
    callResultHistory.value = callResultHistory.value.slice(0, 10);
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

function loadCustomTemplates(): void {
  try {
    const raw = localStorage.getItem(CUSTOM_TEMPLATE_KEY);
    if (!raw) return;
    const arr = JSON.parse(raw) as TemplateItem[];
    if (Array.isArray(arr)) {
      templates.value = [...templates.value, ...arr];
    }
  } catch {
    // ignore
  }
}

function applyTemplate(): void {
  const item = templates.value.find((x) => x.id === selectedTemplateId.value);
  if (!item) return;
  if (item.serverHint) {
    const hit = servers.value.find((s) => s.id.includes(item.serverHint ?? ""));
    if (hit) selectedServerId.value = hit.id;
  }
  selectedToolName.value = item.toolName;
  paramsText.value = JSON.stringify(item.params, null, 2);
}

function saveCurrentTemplate(): void {
  const label = customTemplateName.value.trim();
  if (!label) return;
  let paramsObj: Record<string, unknown> = {};
  try {
    paramsObj = paramsText.value.trim()
      ? (JSON.parse(paramsText.value) as Record<string, unknown>)
      : {};
  } catch {
    return;
  }
  const item: TemplateItem = {
    id: `custom:${Date.now()}`,
    label,
    toolName: selectedToolName.value.trim() || "tool_name",
    params: paramsObj,
    serverHint: selectedServerId.value || undefined,
  };
  const base = templates.value.filter((x) => !x.id.startsWith("custom:"));
  const customs = templates.value.filter((x) => x.id.startsWith("custom:"));
  const next = [...customs, item];
  templates.value = [...base, ...next];
  localStorage.setItem(CUSTOM_TEMPLATE_KEY, JSON.stringify(next));
  customTemplateName.value = "";
}

const compareLeft = computed(() =>
  callResultHistory.value.find((x) => x.id === compareLeftId.value) ?? null,
);
const compareRight = computed(() =>
  callResultHistory.value.find((x) => x.id === compareRightId.value) ?? null,
);

function runDiff(): void {
  if (!compareLeft.value || !compareRight.value) return;
  let left: unknown = compareLeft.value.payload;
  let right: unknown = compareRight.value.payload;
  try {
    left = JSON.parse(compareLeft.value.payload);
    right = JSON.parse(compareRight.value.payload);
  } catch {
    // fallback textual
  }
  const lines: string[] = [];
  function walk(path: string, a: unknown, b: unknown): void {
    const aObj = a && typeof a === "object" && !Array.isArray(a);
    const bObj = b && typeof b === "object" && !Array.isArray(b);
    if (aObj && bObj) {
      const keys = new Set([
        ...Object.keys(a as Record<string, unknown>),
        ...Object.keys(b as Record<string, unknown>),
      ]);
      for (const k of keys) {
        walk(path ? `${path}.${k}` : k, (a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]);
      }
      return;
    }
    if (JSON.stringify(a) !== JSON.stringify(b)) {
      lines.push(`${path || "(root)"}\n  - ${JSON.stringify(a)}\n  + ${JSON.stringify(b)}`);
    }
  }
  walk("", left, right);
  diffText.value = lines.length > 0 ? lines.join("\n") : "无差异";
}

watch(selectedServerId, async (id) => {
  if (!id) {
    availableTools.value = [];
    return;
  }
  availableTools.value = await listMcpTools(id);
});

onMounted(async () => {
  loadCustomTemplates();
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
        模板库
        <select v-model="selectedTemplateId" class="adp-input" @change="applyTemplate">
          <option value="">选择常用模板</option>
          <option v-for="t in templates" :key="t.id" :value="t.id">{{ t.label }}</option>
        </select>
      </label>
      <div class="adp-row">
        <input v-model="customTemplateName" class="adp-input" placeholder="保存当前请求为自定义模板" />
        <button type="button" class="adp-btn" @click="saveCurrentTemplate">保存模板</button>
      </div>
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
        <div class="adp-tool-wrap">
          <input
            v-model="selectedToolName"
            class="adp-input"
            type="text"
            placeholder="例如 get_weather"
          />
          <select v-if="availableTools.length" v-model="selectedToolName" class="adp-input">
            <option value="">从 Server 工具列表选择</option>
            <option v-for="t in availableTools" :key="t.name" :value="t.name">
              {{ t.name }}
            </option>
          </select>
        </div>
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

    <div v-if="callResultHistory.length >= 2" class="adp-diff">
      <h4 class="adp-h4">响应 Diff 对比</h4>
      <div class="adp-row">
        <select v-model="compareLeftId" class="adp-input">
          <option value="">左侧响应</option>
          <option v-for="h in callResultHistory" :key="`l-${h.id}`" :value="h.id">{{ h.label }}</option>
        </select>
        <select v-model="compareRightId" class="adp-input">
          <option value="">右侧响应</option>
          <option v-for="h in callResultHistory" :key="`r-${h.id}`" :value="h.id">{{ h.label }}</option>
        </select>
        <button type="button" class="adp-btn" @click="runDiff">对比</button>
      </div>
      <pre v-if="diffText" class="adp-pre">{{ diffText }}</pre>
    </div>

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

    <EnvVarManager />
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
.adp-tool-wrap {
  display: grid;
  gap: 6px;
}
.adp-diff {
  margin-top: 10px;
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
