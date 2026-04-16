<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: {
    on(event: string, handler: (data: unknown) => void): void;
    off(event: string, handler: (data: unknown) => void): void;
  };
};

type RoleSummary = { id: string; name: string };
type RoleInfoPayload = {
  role_id: string;
  role_name?: string;
  version?: string;
  current_scene?: string | null;
  user_presence_scene?: string | null;
  remote_life_enabled?: boolean;
  current_user_relation?: string;
  default_relation?: string;
  use_manifest_default?: boolean;
  user_relations?: Array<{ id: string; name: string }>;
  relation_state?: string;
  current_life?: { label?: string | null } | null;
};

const oclive = inject<OcliveApi | null>("oclive", null);
const EVT_SET_DRAFT = "com.oclive.mumu.sidebar-glance:set_input_draft";

const roleId = ref("");
const roleName = ref("沐沐");
const relation = ref("—");
const scene = ref("—");
const remotePresence = ref("关");
const relationStage = ref("—");
const lifeLabel = ref("—");
const version = ref("—");
const busy = ref(false);
const err = ref("");

const draftSuggestions = computed(() => [
  `今天看到你在「${scene.value}」，现在心情怎么样？`,
  `关于${relation.value}这件事，我想听你更真实一点的想法。`,
  `你现在是「${lifeLabel.value}」，我可以怎么陪你更舒服？`,
]);

function pushDraft(text: string): void {
  if (!oclive) return;
  const content = text.trim();
  if (!content) return;
  oclive.events.emit(EVT_SET_DRAFT, { text: content });
}

function relationName(info: RoleInfoPayload): string {
  const rows = Array.isArray(info.user_relations) ? info.user_relations : [];
  const cur = (info.current_user_relation ?? "").trim();
  const def = (info.default_relation ?? "").trim();
  const chosen = cur || def;
  const hit = rows.find((x) => x.id === chosen);
  if (info.use_manifest_default) {
    return `默认 · ${(hit?.name ?? chosen) || "—"}`;
  }
  return (hit?.name ?? chosen) || "—";
}

function sceneLine(info: RoleInfoPayload): string {
  const userScene = (info.user_presence_scene ?? "").trim();
  const roleScene = (info.current_scene ?? "").trim();
  if (!userScene && !roleScene) return "—";
  if (userScene && roleScene && userScene !== roleScene) {
    return `${userScene} · 角色在 ${roleScene}`;
  }
  return userScene || roleScene;
}

function remoteLine(info: RoleInfoPayload): string {
  if (!info.remote_life_enabled) return "关";
  const userScene = (info.user_presence_scene ?? "").trim();
  const roleScene = (info.current_scene ?? "").trim();
  if (userScene && roleScene && userScene !== roleScene) return "开（异地）";
  return "开（同场）";
}

async function resolveCurrentRoleId(): Promise<string> {
  if (!oclive) return "";
  const rows = (await oclive.invoke("list_roles", {})) as RoleSummary[];
  if (Array.isArray(rows) && rows.length > 0) {
    return String(rows[0]?.id ?? "").trim();
  }
  return "";
}

async function refresh(nextRoleId?: string): Promise<void> {
  if (!oclive) return;
  busy.value = true;
  err.value = "";
  try {
    const rid = (nextRoleId ?? roleId.value).trim() || (await resolveCurrentRoleId());
    if (!rid) {
      roleId.value = "";
      roleName.value = "沐沐";
      relation.value = "—";
      scene.value = "—";
      remotePresence.value = "关";
      relationStage.value = "—";
      lifeLabel.value = "—";
      version.value = "—";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    roleId.value = rid;
    roleName.value = (info.role_name ?? "").trim() || "沐沐";
    relation.value = relationName(info);
    scene.value = sceneLine(info);
    remotePresence.value = remoteLine(info);
    relationStage.value = (info.relation_state ?? "").trim() || "—";
    lifeLabel.value = (info.current_life?.label ?? "").trim() || "暂无日程";
    version.value = (info.version ?? "").trim() || "—";
  } catch (e) {
    err.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function onRoleSwitched(payload: unknown): void {
  const next = (payload as { roleId?: string } | null)?.roleId;
  void refresh(typeof next === "string" ? next : undefined);
}

function onMessageSent(): void {
  void refresh();
}

function onThemeChanged(): void {
  void refresh();
}

onMounted(() => {
  if (!oclive) return;
  oclive.events.on("oclive:role:switched", onRoleSwitched);
  oclive.events.on("oclive:message:sent", onMessageSent);
  oclive.events.on("oclive:theme:changed", onThemeChanged);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:message:sent", onMessageSent);
  oclive.events.off("oclive:theme:changed", onThemeChanged);
});
</script>

<template>
  <aside class="card" aria-label="mumu 侧栏概览">
    <header class="head">
      <div class="title-wrap">
        <h3 class="title">{{ roleName }}</h3>
        <p class="sub">v{{ version }} · {{ roleId || "未加载角色" }}</p>
      </div>
      <span v-if="busy" class="busy">同步中</span>
    </header>

    <dl class="grid">
      <div class="item">
        <dt>关系</dt>
        <dd>{{ relation }}</dd>
      </div>
      <div class="item">
        <dt>关系阶段</dt>
        <dd>{{ relationStage }}</dd>
      </div>
      <div class="item item-wide">
        <dt>叙事场景</dt>
        <dd>{{ scene }}</dd>
      </div>
      <div class="item">
        <dt>异地心声</dt>
        <dd>{{ remotePresence }}</dd>
      </div>
      <div class="item item-wide">
        <dt>此刻状态</dt>
        <dd>{{ lifeLabel }}</dd>
      </div>
    </dl>

    <section class="draft-box" aria-label="建议下一句（仅填充输入框）">
      <p class="draft-title">建议下一句（不会自动发送）</p>
      <div class="draft-actions">
        <button
          v-for="line in draftSuggestions"
          :key="line"
          type="button"
          class="draft-btn"
          :disabled="busy"
          :title="line"
          @click="pushDraft(line)"
        >
          {{ line }}
        </button>
      </div>
    </section>

    <p v-if="err" class="err" :title="err">读取失败：{{ err }}</p>
  </aside>
</template>

<style scoped>
.card {
  --ui-trans-fast: 140ms;
  --ui-state-warn-fg: color-mix(in srgb, var(--accent, #8f7f6a) 88%, black 12%);
  --ui-state-warn-bg: color-mix(in srgb, var(--accent, #8f7f6a) 12%, transparent);
  --ui-state-warn-border: color-mix(in srgb, var(--accent, #8f7f6a) 40%, transparent);
  --ui-state-danger-fg: var(--text-danger, #c33);
  width: 100%;
  box-sizing: border-box;
  padding: 12px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 72%, transparent);
  background:
    linear-gradient(
      165deg,
      color-mix(in srgb, var(--bg-primary, #fffdf9) 80%, white 20%),
      color-mix(in srgb, var(--bg-elevated, #f7f2ea) 86%, white 14%)
    );
  backdrop-filter: blur(14px) saturate(112%);
  -webkit-backdrop-filter: blur(14px) saturate(112%);
  box-shadow:
    0 8px 20px color-mix(in srgb, var(--text-primary, #3f3a33) 7%, transparent),
    inset 0 1px 0 color-mix(in srgb, white 68%, transparent);
  color: var(--text-primary, #3f3a33);
}
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 10px;
}
.title {
  margin: 0;
  font-size: 14px;
  font-weight: 620;
  letter-spacing: 0.01em;
}
.sub {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.busy {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--ui-state-warn-border);
  color: var(--ui-state-warn-fg);
  background: var(--ui-state-warn-bg);
  white-space: nowrap;
}
.grid {
  margin: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.item {
  min-width: 0;
  padding: 8px;
  border-radius: 11px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 72%, transparent);
  background: color-mix(in srgb, var(--bg-primary, #fffdf9) 88%, transparent);
  transition:
    border-color var(--ui-trans-fast) ease,
    background-color var(--ui-trans-fast) ease;
}
.item-wide {
  grid-column: 1 / -1;
}
dt {
  margin: 0 0 3px;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
dd {
  margin: 0;
  font-size: 12px;
  line-height: 1.35;
  word-break: break-word;
}
.err {
  margin: 10px 0 0;
  font-size: 11px;
  color: var(--ui-state-danger-fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.draft-box {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 70%, transparent);
}
.draft-title {
  margin: 0 0 6px;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.draft-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.draft-btn {
  width: 100%;
  text-align: left;
  font-size: 11px;
  line-height: 1.35;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 74%, transparent);
  background: color-mix(in srgb, var(--bg-primary, #fffdf9) 90%, transparent);
  color: var(--text-primary, #3f3a33);
  padding: 7px 9px;
  cursor: pointer;
  transition:
    border-color var(--ui-trans-fast) ease,
    transform var(--ui-trans-fast) ease;
}
.draft-btn:hover {
  border-color: color-mix(in srgb, var(--accent, #8f7f6a) 56%, transparent);
  transform: translateY(-0.5px);
}
.draft-btn:disabled {
  cursor: not-allowed;
  opacity: 0.62;
  transform: none;
}
</style>
