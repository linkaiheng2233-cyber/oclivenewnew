<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref } from "vue";

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
  version?: string;
  current_scene?: string | null;
  user_presence_scene?: string | null;
  remote_life_enabled?: boolean;
  current_user_relation?: string;
  use_manifest_default?: boolean;
  default_relation?: string;
  user_relations?: Array<{ id: string; name: string }>;
};

const oclive = inject<OcliveApi | null>("oclive", null);

const busy = ref(false);
const roleId = ref("");
const relationText = ref("—");
const sceneText = ref("—");
const remoteText = ref("—");
const versionText = ref("—");
const errText = ref("");

function relationLabel(info: RoleInfoPayload): string {
  const rows = Array.isArray(info.user_relations) ? info.user_relations : [];
  const current = (info.current_user_relation ?? "").trim();
  const fallback = (info.default_relation ?? "").trim();
  const effective = current || fallback;
  const row = rows.find((x) => x.id === effective);
  if (info.use_manifest_default) {
    return `默认身份（${(row?.name ?? effective) || "—"}）`;
  }
  return (row?.name ?? effective) || "—";
}

function sceneLabel(info: RoleInfoPayload): string {
  const userScene = (info.user_presence_scene ?? "").trim();
  const roleScene = (info.current_scene ?? "").trim();
  if (!userScene && !roleScene) return "—";
  if (userScene && roleScene && userScene !== roleScene) {
    return `${userScene}（角色在 ${roleScene}）`;
  }
  return userScene || roleScene;
}

function remoteLabel(info: RoleInfoPayload): string {
  const enabled = info.remote_life_enabled === true;
  if (!enabled) return "异地心声：关";
  const userScene = (info.user_presence_scene ?? "").trim();
  const roleScene = (info.current_scene ?? "").trim();
  if (userScene && roleScene && userScene !== roleScene) {
    return "异地心声：开（异地）";
  }
  return "异地心声：开（同场景）";
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
  errText.value = "";
  try {
    const rid = (nextRoleId ?? roleId.value).trim() || (await resolveCurrentRoleId());
    if (!rid) {
      relationText.value = "—";
      sceneText.value = "—";
      remoteText.value = "—";
      versionText.value = "—";
      roleId.value = "";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    roleId.value = rid;
    relationText.value = relationLabel(info);
    sceneText.value = sceneLabel(info);
    remoteText.value = remoteLabel(info);
    versionText.value = (info.version ?? "").trim() || "—";
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
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

const roleBadge = computed(() => (roleId.value ? `角色：${roleId.value}` : "角色：—"));
</script>

<template>
  <section class="panel mumu-surface">
    <div class="panel-head">
      <strong>聊天头部状态</strong>
      <span v-if="busy" class="sync">同步中</span>
    </div>
    <div class="line">
      <span class="chip">{{ roleBadge }}</span>
      <span class="chip">版本：{{ versionText }}</span>
      <span class="chip">关系：{{ relationText }}</span>
    </div>
    <div class="line">
      <span class="chip">场景：{{ sceneText }}</span>
      <span class="chip">{{ remoteText }}</span>
    </div>
    <p v-if="errText" class="err" :title="errText">状态读取失败：{{ errText }}</p>
  </section>
</template>

<style scoped>
.panel {
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
  min-height: 3.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.4375rem;
  padding: 0.625rem 0.75rem;
  box-sizing: border-box;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: var(--shadow-sm), var(--frame-inset-highlight);
  backdrop-filter: blur(12px) saturate(110%);
  -webkit-backdrop-filter: blur(12px) saturate(110%);
  -webkit-font-smoothing: antialiased;
}
.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}
.panel-head strong {
  font-size: 0.75rem;
  font-weight: 620;
  color: var(--text-secondary);
}
.sync {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--ui-state-warn-border);
  color: var(--ui-state-warn-fg);
  background: var(--ui-state-warn-bg);
}
.line {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}
.chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.4375rem;
  padding: 0.125rem 0.5625rem;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.6875rem;
  line-height: 1.2;
  transition:
    border-color var(--ui-trans-fast) ease,
    background-color var(--ui-trans-fast) ease,
    color var(--ui-trans-fast) ease;
}
.err {
  margin: 0;
  color: var(--error);
  font-size: 0.6875rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
