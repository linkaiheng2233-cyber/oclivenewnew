<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { useAppToast } from "../../composables/useAppToast";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import type { ExpertGraph, ExpertNode, PromptStyleOverride } from "../../utils/tauri-api";

const store = useExpertModelsStore();
const { showToast } = useAppToast();
const emit = defineEmits<{
  (e: "open-permissions", payload: { pluginId: string }): void;
}>();

const saving = ref(false);

const sourceLabel = (s: string): string => {
  if (s === "session_override") return "会话覆盖";
  if (s === "role_default") return "角色默认";
  return "角色包默认";
};

const baseModelNode = computed(() => {
  const g = store.draftGraph;
  return g.nodes.find((n) => n.type === "base_model") as
    | { type: "base_model"; id: string; ggufPath: string }
    | undefined;
});

const selectedBaseModelPath = computed({
  get(): string {
    return baseModelNode.value?.ggufPath ?? "";
  },
  set(v: string) {
    const g: ExpertGraph = store.draftGraph;
    const nextNodes: ExpertNode[] = [...(g.nodes ?? [])].filter(
      (n) => n.type !== "base_model",
    );
    const t = (v ?? "").trim();
    if (t) {
      nextNodes.unshift({ type: "base_model", id: "base", ggufPath: t });
    }
    store.draftGraph = { ...g, nodes: nextNodes };
  },
});

type LoraNode = Extract<ExpertNode, { type: "lora_adapter" }>;
const loraNodes = computed<LoraNode[]>(() =>
  (store.draftGraph.nodes ?? [])
    .filter((n) => n.type === "lora_adapter")
    .map((n) => n as LoraNode)
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.id.localeCompare(b.id)),
);

const ensurePromptStyle = (): PromptStyleOverride => {
  if (!store.draftPromptStyle) store.draftPromptStyle = {};
  return store.draftPromptStyle;
};

function addLora(path: string): void {
  const p = (path ?? "").trim();
  if (!p) return;
  const g = store.draftGraph;
  const id = `lora_${Math.random().toString(36).slice(2, 8)}`;
  const order = loraNodes.value.length;
  store.draftGraph = {
    ...g,
    nodes: [
      ...(g.nodes ?? []),
      {
        type: "lora_adapter",
        id,
        ggufPath: p,
        strength: 1.0,
        enabled: true,
        order,
      },
    ],
  };
}

function removeLora(id: string): void {
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).filter((n) => !(n.type === "lora_adapter" && n.id === id)),
  };
}

function moveLora(id: string, dir: -1 | 1): void {
  const list = [...loraNodes.value];
  const idx = list.findIndex((x) => x.id === id);
  if (idx < 0) return;
  const to = idx + dir;
  if (to < 0 || to >= list.length) return;
  const a = list[idx]!;
  const b = list[to]!;
  const next = list.map((x) => ({ ...x }));
  next[idx] = { ...b, order: a.order };
  next[to] = { ...a, order: b.order };
  const others = (store.draftGraph.nodes ?? []).filter((n) => n.type !== "lora_adapter");
  store.draftGraph = { ...store.draftGraph, nodes: [...others, ...next] };
}

const strengthWarning = (v: number): string | null => {
  if (!Number.isFinite(v)) return "强度必须是数字。";
  if (v < 0) return "强度 < 0 通常不合理。";
  if (v > 2) return "强度 > 2 可能导致输出劣化或不稳定。";
  if (v > 1.4) return "强度偏高，建议先从 1.0–1.4 试起。";
  return null;
};

async function onRefresh(): Promise<void> {
  await store.refresh();
  if (store.error) showToast("error", store.error);
}

async function onApplySession(): Promise<void> {
  saving.value = true;
  try {
    const r = await store.applyToSession();
    showToast(
      "success",
      `已应用到当前会话（将触发本地 llama 重启）。\nmodelPath=${r.modelPath ?? "(未设置)"}\nllamaArgs=${r.llamaArgs ?? "(空)"}`,
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onImportBase(): Promise<void> {
  const picked = await open({
    title: "选择一个 Base GGUF（将复制到 models/gguf）",
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importBaseGguf(p);
    showToast("success", "已导入 Base 模型到 models/gguf。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onImportLora(): Promise<void> {
  const picked = await open({
    title: "选择一个 LoRA GGUF（将复制到 models/loras）",
    multiple: false,
    directory: false,
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  const p = typeof picked === "string" ? picked : null;
  if (!p) return;
  saving.value = true;
  try {
    await store.importLoraGguf(p);
    showToast("success", "已导入 LoRA 到 models/loras。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onSetRoleDefault(): Promise<void> {
  saving.value = true;
  try {
    await store.setRoleDefault();
    showToast("success", "已设置为角色默认。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearSessionOverride(): Promise<void> {
  const ok = window.confirm("将清除当前会话的 Expert Models 覆盖，并回退到角色默认/角色包默认。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearSessionOverrideAndApply();
    showToast("success", "已清除会话覆盖并重新应用。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearRoleDefault(): Promise<void> {
  const ok = window.confirm("将清除该角色的 Expert Models 默认配置（不会影响角色包原文件）。继续吗？");
  if (!ok) return;
  saving.value = true;
  try {
    await store.clearRoleDefault();
    showToast("success", "已清除角色默认。");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  if (!store.baseModels.length && !store.loading) void onRefresh();
});
</script>

<template>
  <section class="em-root" aria-label="Expert Models（Module 9）">
    <header class="em-h">
      <div>
        <h3 class="em-title">Expert Models（Module 9）</h3>
        <p class="em-sub">
          选择 Base GGUF + LoRA 强度，并可选覆盖 PromptStyle。会话覆盖优先于角色默认；不设置时不改变现有行为。
        </p>
      </div>
      <div class="em-actions">
        <button class="em-btn secondary" type="button" :disabled="store.loading || saving" @click="onRefresh">
          刷新
        </button>
        <button class="em-btn" type="button" :disabled="store.loading || saving" @click="store.setDraftFromEffective">
          从有效配置回填编辑器
        </button>
      </div>
    </header>

    <div class="em-meta">
      <div class="em-pill">
        Graph 来源：<b>{{ sourceLabel(store.graphSource) }}</b>
      </div>
      <div class="em-pill">
        PromptStyle 来源：<b>{{ sourceLabel(store.promptStyleSource) }}</b>
      </div>
      <div v-if="store.llamaMissingMechanismPerms.length" class="em-warnbar">
        <div>
          <b>本地 Llama 尚未授权必要权限</b>
          <span class="em-muted2">
            缺少：{{ store.llamaMissingMechanismPerms.join("、") }}。未授权时会回退其他 LLM 或调用被拦截。
          </span>
        </div>
        <button
          type="button"
          class="em-btn danger"
          @click="emit('open-permissions', { pluginId: 'com.oclive.llama.local' })"
        >
          去授权
        </button>
      </div>
      <div v-if="store.error" class="em-err">{{ store.error }}</div>
    </div>

    <div class="em-grid">
      <div class="em-card">
        <div class="em-card-h">Base 模型（GGUF）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportBase">
            导入 GGUF…
          </button>
        </div>
        <select v-model="selectedBaseModelPath" class="em-select">
          <option value="">（不设置 / 保持当前）</option>
          <option v-for="m in store.baseModels" :key="m.path" :value="m.path">
            {{ m.name }}
          </option>
        </select>
        <div class="em-muted">目录：`{app_data}/models/gguf/*.gguf`</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">LoRA（可多选）</div>
        <div class="em-row3">
          <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onImportLora">
            导入 LoRA…
          </button>
        </div>
        <div class="em-lora-add">
          <select class="em-select" @change="addLora(($event.target as HTMLSelectElement).value)">
            <option value="">添加一个 LoRA…</option>
            <option v-for="m in store.loras" :key="m.path" :value="m.path">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div v-if="loraNodes.length === 0" class="em-muted">尚未添加 LoRA。</div>
        <ul v-else class="em-lora-list">
          <li v-for="n in loraNodes" :key="n.id" class="em-lora">
            <label class="em-row">
              <input
                type="checkbox"
                :checked="n.enabled"
                @change="
                  store.draftGraph = {
                    ...store.draftGraph,
                    nodes: store.draftGraph.nodes.map((x) =>
                      x.type === 'lora_adapter' && x.id === n.id
                        ? { ...x, enabled: ($event.target as HTMLInputElement).checked }
                        : x,
                    ),
                  }
                "
              />
              <span class="em-mono">{{ n.ggufPath.split(/[\\/]/).slice(-1)[0] }}</span>
            </label>

            <div class="em-row em-row2">
              <label class="em-muted">
                强度
                <input
                  class="em-num"
                  type="number"
                  step="0.05"
                  :value="n.strength"
                  @input="
                    store.draftGraph = {
                      ...store.draftGraph,
                      nodes: store.draftGraph.nodes.map((x) =>
                        x.type === 'lora_adapter' && x.id === n.id
                          ? { ...x, strength: Number(($event.target as HTMLInputElement).value) }
                          : x,
                      ),
                    }
                  "
                />
              </label>
              <span v-if="strengthWarning(n.strength)" class="em-warn">
                {{ strengthWarning(n.strength) }}
              </span>
            </div>

            <div class="em-lora-actions">
              <button class="em-mini" type="button" @click="moveLora(n.id, -1)">上移</button>
              <button class="em-mini" type="button" @click="moveLora(n.id, 1)">下移</button>
              <button class="em-mini danger" type="button" @click="removeLora(n.id)">移除</button>
            </div>
          </li>
        </ul>

        <div class="em-muted">目录：`{app_data}/models/loras/*.gguf`（也兼容放在 gguf 目录）</div>
      </div>

      <div class="em-card">
        <div class="em-card-h">PromptStyle（可选覆盖）</div>
        <label class="em-field">
          <div class="em-label">回复质量锚点（覆盖角色包/默认）</div>
          <textarea
            class="em-text"
            rows="4"
            :value="store.draftPromptStyle?.replyQualityAnchor ?? ''"
            @input="ensurePromptStyle().replyQualityAnchor = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">核心人设（覆盖 role.core_personality）</div>
          <textarea
            class="em-text"
            rows="3"
            :value="store.draftPromptStyle?.corePersonality ?? ''"
            @input="ensurePromptStyle().corePersonality = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <label class="em-field">
          <div class="em-label">描述（覆盖 role.description）</div>
          <textarea
            class="em-text"
            rows="2"
            :value="store.draftPromptStyle?.description ?? ''"
            @input="ensurePromptStyle().description = ($event.target as HTMLTextAreaElement).value"
            placeholder="留空表示不覆盖"
          />
        </label>
        <div class="em-muted">提示：未设置时，Prompt 行为与当前版本完全一致。</div>
      </div>
    </div>

    <div class="em-footer">
      <button class="em-btn" type="button" :disabled="saving || store.loading" @click="onApplySession">
        {{ saving ? "应用中…" : "应用到当前会话（重启本地 llama）" }}
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onSetRoleDefault">
        设为角色默认
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearSessionOverride">
        清除会话覆盖
      </button>
      <button class="em-btn secondary" type="button" :disabled="saving || store.loading" @click="onClearRoleDefault">
        清除角色默认
      </button>
    </div>
  </section>
</template>

<style scoped>
.em-root {
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.em-h {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}
.em-title {
  margin: 0 0 6px;
  font-size: 16px;
}
.em-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.em-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.em-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 13px;
}
.em-btn.secondary {
  background: transparent;
}
.em-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
  align-items: center;
}
.em-pill {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  color: var(--text-secondary);
}
.em-warnbar {
  flex: 1 1 520px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--danger-600, #c0392b) 30%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 10%, var(--bg-elevated));
  color: var(--text-primary);
  font-size: 12px;
}
.em-muted2 {
  margin-left: 6px;
  color: var(--text-secondary);
}
.em-btn.danger {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
}
.em-err {
  color: var(--danger-600, #c0392b);
  font-size: 12px;
}
.em-grid {
  margin-top: 10px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
  gap: 12px;
}
.em-card {
  min-width: 0;
  padding: 10px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.em-card-h {
  font-size: 13px;
  font-weight: 700;
  margin-bottom: 8px;
}
.em-row3 {
  display: flex;
  justify-content: flex-end;
  margin: -2px 0 8px;
}
.em-select {
  width: 100%;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.em-muted {
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}
.em-lora-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.em-lora {
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 8px;
  background: var(--bg-primary);
}
.em-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.em-row2 {
  margin-top: 6px;
  align-items: baseline;
  flex-wrap: wrap;
}
.em-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.em-num {
  width: 120px;
  margin-left: 6px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.em-warn {
  color: color-mix(in srgb, #f59e0b 75%, var(--text-primary));
  font-size: 12px;
}
.em-lora-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}
.em-mini {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
}
.em-mini.danger {
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
  color: var(--danger-600, #c0392b);
}
.em-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 10px;
}
.em-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.em-text {
  width: 100%;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.45;
  resize: vertical;
}
.em-footer {
  margin-top: 12px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
@media (max-width: 1080px) {
  .em-grid {
    grid-template-columns: 1fr;
  }
}
</style>

