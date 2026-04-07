<script setup lang="ts">
import { computed, ref } from "vue";
import ChatExportBar from "./ChatExportBar.vue";
import RolePackBar from "./RolePackBar.vue";
import RoleRuntimePanel from "./RoleRuntimePanel.vue";
import { useChatStore } from "../stores/chatStore";
import { useDebugStore } from "../stores/debugStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { generateMonologue } from "../utils/tauri-api";
import {
  PERSONALITY_TRAIT_KEYS,
  PERSONALITY_TRAIT_LABELS_ZH,
  vec7ToRecord,
} from "../utils/personality-traits";

const props = defineProps<{
  visible: boolean;
  loading: boolean;
  favorability: number;
  personality: number[];
  events: Array<{ event_type?: string; timestamp?: string; description?: string | null }>;
  memories: Array<{ content?: string; timestamp?: string; importance?: number }>;
}>();

const roleStore = useRoleStore();
const debugStore = useDebugStore();
const chatStore = useChatStore();
const uiStore = useUiStore();
const monoLoading = ref(false);

const emit = defineEmits<{
  reload: [];
  refresh: [];
  close: [];
  notify: [{ type: "success" | "error" | "info" | "warning"; message: string }];
  imported: [roleId: string];
}>();

async function insertMonologue(): Promise<void> {
  const roleId = roleStore.currentRoleId;
  if (!roleId) return;
  monoLoading.value = true;
  try {
    const text = await generateMonologue(roleId);
    chatStore.addAssistantMessage(`【独白】${text}`, undefined, uiStore.sceneId);
    emit("notify", { type: "info", message: "已插入独白" });
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  } finally {
    monoLoading.value = false;
  }
}

const traits = computed(() => vec7ToRecord(props.personality));

function favEmoji(v: number): string {
  if (v >= 80) return "😍";
  if (v >= 60) return "🥰";
  if (v >= 40) return "😊";
  if (v >= 20) return "😐";
  return "😔";
}

function favStatusText(v: number): string {
  if (v >= 80) return "💖 超级亲密！";
  if (v >= 60) return "💕 关系很好~";
  if (v >= 40) return "👍 还不错";
  if (v >= 20) return "🤝 慢慢熟悉中";
  return "😶 还有点陌生";
}

function traitEmoji(val: number, hi: string, mid: string, low: string): string {
  if (val >= 0.7) return hi;
  if (val >= 0.4) return mid;
  return low;
}

const traitEmojiMap: Record<
  (typeof PERSONALITY_TRAIT_KEYS)[number],
  [string, string, string]
> = {
  stubbornness: ["😤", "🤔", "😌"],
  clinginess: ["🥺", "😊", "😐"],
  sensitivity: ["😢", "😳", "😶"],
  assertiveness: ["👑", "💪", "🍃"],
  forgiveness: ["😇", "🙂", "😤"],
  talkativeness: ["🗣️", "💬", "🤐"],
  warmth: ["🔥", "☀️", "❄️"],
};

function traitEmojiForKey(
  key: (typeof PERSONALITY_TRAIT_KEYS)[number],
  val: number,
): string {
  const [hi, mid, low] = traitEmojiMap[key];
  return traitEmoji(val, hi, mid, low);
}

function presenceLabel(mode: string): string {
  if (mode === "co_present") return "共景";
  if (mode === "remote_stub") return "异地占位";
  if (mode === "remote_life") return "异地心声";
  return mode;
}
</script>

<template>
  <transition name="slide">
    <aside v-if="visible" class="debug debug-scroll">
      <div class="title">
        <strong>🎛️ 开发面板</strong>
        <button type="button" aria-label="关闭" @click="emit('close')">✕</button>
      </div>

      <div class="debug-toolbar">
        <RolePackBar
          @notify="emit('notify', $event)"
          @imported="emit('imported', $event)"
        />
        <button
          type="button"
          class="btn-mono"
          :disabled="loading || monoLoading"
          @click="insertMonologue"
        >
          {{ monoLoading ? "生成中…" : "插入独白" }}
        </button>
      </div>

      <RoleRuntimePanel class="debug-runtime" />

      <ChatExportBar
        class="debug-export"
        :role-id="roleStore.currentRoleId"
        @notify="emit('notify', $event)"
      />

      <div class="dev-card knowledge-card">
        <div class="dev-title"><span>📚</span> 世界观知识</div>
        <p class="knowledge-line">
          包内索引：
          <strong>{{
            roleStore.roleInfo.knowledgeEnabled ? "已加载" : "未加载"
          }}</strong>
          · 共 {{ roleStore.roleInfo.knowledgeChunkCount }} 块
        </p>
        <p class="knowledge-line">
          上一句注入 Prompt：
          <strong>{{ debugStore.lastKnowledgeChunksInPrompt }}</strong> 块
          <span
            v-if="debugStore.lastKnowledgePresenceMode"
            class="knowledge-mode"
          >
            （{{ presenceLabel(debugStore.lastKnowledgePresenceMode) }}）
          </span>
        </p>
        <p class="knowledge-hint">
          发话后更新「上一句」；点「刷新调试数据」同步包内块数（改磁盘后请先
          load_role）。
        </p>
      </div>

      <div class="dev-card">
        <div class="dev-title">
          <span>❤️</span> 好感度
          <span class="dev-emoji">{{ favEmoji(favorability) }}</span>
        </div>
        <div class="fav-value">{{ Math.round(favorability) }}</div>
        <div class="fav-bar">
          <div
            class="fav-fill"
            :style="{ width: `${Math.max(0, Math.min(100, favorability))}%` }"
          />
        </div>
        <div class="fav-status">{{ favStatusText(favorability) }}</div>
      </div>

      <div class="dev-card">
        <div class="dev-title"><span>🎭</span> 性格向量</div>
        <div class="trait-grid">
          <div
            v-for="key in PERSONALITY_TRAIT_KEYS"
            :key="key"
            class="trait-item"
          >
            <span class="trait-name">
              {{ PERSONALITY_TRAIT_LABELS_ZH[key] }} {{ traitEmojiForKey(key, traits[key]) }}
            </span>
            <span class="trait-value">{{ traits[key].toFixed(2) }}</span>
          </div>
        </div>
      </div>

      <p class="meta-line">事件数: {{ events.length }} · 记忆数: {{ memories.length }}</p>

      <details>
        <summary>最近事件</summary>
        <ul>
          <li v-for="(e, i) in events.slice(0, 5)" :key="`e-${i}`">
            {{ e.event_type ?? "unknown" }} · {{ e.timestamp ?? "-" }}
          </li>
        </ul>
      </details>
      <details>
        <summary>最近记忆</summary>
        <ul>
          <li v-for="(m, i) in memories.slice(0, 5)" :key="`m-${i}`">
            {{ m.content ?? "-" }} ({{ Number(m.importance ?? 0).toFixed(2) }})
          </li>
        </ul>
      </details>

      <div class="btns">
        <button type="button" :disabled="loading" @click="emit('refresh')">
          刷新调试数据
        </button>
        <button type="button" :disabled="loading" @click="emit('reload')">重载策略</button>
      </div>

      <div class="dev-footer">💡 Ctrl+Shift+D 开关面板 · 角色包与独白已收在此</div>
    </aside>
  </transition>
</template>

<style scoped>
.debug-toolbar {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 14px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border-light);
}
.debug-toolbar :deep(.pack-bar) {
  width: 100%;
}
.btn-mono {
  width: 100%;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 10px 14px;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
}
.btn-mono:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text-accent);
}
.btn-mono:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.debug-runtime :deep(.runtime) {
  background: transparent;
  border: none;
  margin: 0;
  padding: 0 0 12px;
}

.debug-runtime {
  margin: 0 0 4px;
}

.debug-export {
  margin-bottom: 14px;
}

.debug-export :deep(.export-bar) {
  padding: 0;
}

/* 对齐 oclive-new updateDevPanel 内联样式 */
.dev-card {
  background: var(--bg-primary);
  border-radius: 16px;
  padding: 14px;
  margin-bottom: 16px;
  border: 1px solid var(--border-light);
}
.dev-title {
  font-size: 13px;
  color: var(--text-accent);
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.dev-emoji {
  margin-left: auto;
  font-size: 20px;
}
.fav-value {
  font-size: 32px;
  font-weight: bold;
  text-align: center;
  color: var(--text-accent);
}
.fav-bar {
  height: 6px;
  background: #2a2a30;
  border-radius: 3px;
  overflow: hidden;
  margin: 10px 0 6px;
}
.fav-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent), #c98a5c);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.fav-status {
  font-size: 10px;
  color: var(--text-light);
  text-align: center;
  margin-top: 6px;
}
.trait-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}
.trait-item {
  background: var(--bg-elevated);
  border-radius: 10px;
  padding: 6px 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  border: 1px solid #2f2f38;
}
.trait-name {
  color: var(--text-secondary);
}
.trait-value {
  font-weight: bold;
  color: var(--text-accent);
}
.meta-line {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0 0 8px;
}

.knowledge-card .knowledge-line {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  line-height: 1.45;
}
.knowledge-card .knowledge-line strong {
  color: var(--text-accent);
}
.knowledge-mode {
  color: var(--text-light);
  font-size: 12px;
}
.knowledge-hint {
  font-size: 11px;
  color: var(--text-light);
  margin: 0;
  line-height: 1.4;
}

/* 对齐 oclive-new #devPanel */
.debug {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: min(420px, calc(100vw - 24px));
  max-height: 85vh;
  overflow-y: auto;
  z-index: 10000;
  padding: 20px;
  background: var(--bg-primary);
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  color: var(--text-primary);
}
.title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}
.title strong {
  color: var(--text-accent);
  font-size: 15px;
}
.title button {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 20px;
  cursor: pointer;
  line-height: 1;
}
.title button:hover {
  color: var(--text-accent);
}
.btns {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 12px;
}
.btns button {
  padding: 6px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 13px;
}
.btns button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
ul {
  margin: 6px 0 8px 16px;
  padding: 0;
}
li {
  margin: 4px 0;
  color: var(--text-secondary);
  font-size: 13px;
}
details {
  margin: 8px 0;
  color: var(--text-secondary);
  font-size: 13px;
}
summary {
  cursor: pointer;
  color: var(--text-accent);
}
.dev-footer {
  margin-top: 16px;
  font-size: 11px;
  color: var(--text-light);
  text-align: center;
  border-top: 1px solid var(--border-light);
  padding-top: 12px;
}
.slide-enter-active,
.slide-leave-active {
  transition: opacity 220ms ease;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}
</style>
