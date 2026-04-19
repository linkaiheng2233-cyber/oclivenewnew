<script setup lang="ts">
import { inject, onMounted, onUnmounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  getAppearance?: () => { effectiveTheme: "light" | "dark"; scale: number };
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

const relation = ref("—");
const scene = ref("—");
const lifeLabel = ref("—");
const busy = ref(false);
const err = ref("");

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
    const rid = (nextRoleId ?? "").trim() || (await resolveCurrentRoleId());
    if (!rid) {
      relation.value = "—";
      scene.value = "—";
      lifeLabel.value = "—";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    relation.value = relationName(info);
    scene.value = sceneLine(info);
    lifeLabel.value = (info.current_life?.label ?? "").trim() || "暂无日程";
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

onMounted(() => {
  if (!oclive) return;
  oclive.events.on("oclive:role:switched", onRoleSwitched);
  oclive.events.on("oclive:role:info:updated", onRoleSwitched);
  oclive.events.on("oclive:message:sent", onMessageSent);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:role:info:updated", onRoleSwitched);
  oclive.events.off("oclive:message:sent", onMessageSent);
});
</script>

<template>
  <aside class="card mumu-surface" aria-label="此刻情境">
    <header class="sg-head">
      <h3 class="sg-title">此刻</h3>
      <span v-if="busy" class="busy">同步中</span>
    </header>
    <p class="sg-lede">只读情境，不提示台词，避免打断你自己的说法。</p>

    <dl class="ambient" aria-label="当前叙事情境">
      <div class="ambient-row">
        <dt>关系</dt>
        <dd>{{ relation }}</dd>
      </div>
      <div class="ambient-row">
        <dt>所在</dt>
        <dd>{{ scene }}</dd>
      </div>
      <div class="ambient-row">
        <dt>日程</dt>
        <dd>{{ lifeLabel }}</dd>
      </div>
    </dl>

    <p v-if="err" class="err" :title="err">读取失败：{{ err }}</p>
  </aside>
</template>

<style scoped>
.card {
  --ui-trans-fast: 140ms;
  --ui-state-warn-fg: color-mix(in srgb, var(--accent) 88%, var(--text-primary) 12%);
  --ui-state-warn-bg: color-mix(in srgb, var(--accent) 14%, transparent);
  --ui-state-warn-border: color-mix(in srgb, var(--accent) 42%, var(--border-light) 58%);
}
.mumu-surface {
  font-family: var(--font-ui);
  font-size: 0.875rem;
  line-height: 1.45;
  color: var(--text-primary);
  width: 100%;
  box-sizing: border-box;
  padding: 0.75rem;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: var(--shadow-sm), var(--frame-inset-highlight);
  backdrop-filter: blur(12px) saturate(110%);
  -webkit-backdrop-filter: blur(12px) saturate(110%);
  -webkit-font-smoothing: antialiased;
}
.sg-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.375rem;
}
.sg-title {
  margin: 0;
  font-size: 0.8125rem;
  font-weight: 620;
  letter-spacing: 0.02em;
}
.sg-lede {
  margin: 0 0 0.625rem;
  font-size: 0.65625rem;
  line-height: 1.45;
  color: var(--text-secondary);
  pointer-events: none;
  user-select: none;
}
.ambient {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.ambient-row {
  display: grid;
  grid-template-columns: 2.25rem 1fr;
  gap: 0.5rem;
  align-items: start;
  font-size: 0.8125rem;
  line-height: 1.4;
}
.ambient-row dt {
  margin: 0;
  font-weight: 600;
  font-size: 0.625rem;
  letter-spacing: 0.04em;
  color: var(--text-secondary);
  text-transform: uppercase;
}
.ambient-row dd {
  margin: 0;
  color: var(--text-primary);
}
.busy {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--ui-state-warn-border);
  color: var(--ui-state-warn-fg);
  background: var(--ui-state-warn-bg);
  white-space: nowrap;
}
.err {
  margin: 0.625rem 0 0;
  font-size: 0.6875rem;
  color: var(--error);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
