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
    return `默认身份（${row?.name ?? effective || "—"}）`;
  }
  return row?.name ?? effective || "—";
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
  oclive.events.on("oclive:message:sent", onMessageSent);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:message:sent", onMessageSent);
});

const roleBadge = computed(() => (roleId.value ? `角色：${roleId.value}` : "角色：—"));
</script>

<template>
  <div class="status-wrap">
    <div class="line">
      <span class="pill">{{ roleBadge }}</span>
      <span class="pill">版本：{{ versionText }}</span>
      <span class="pill">关系：{{ relationText }}</span>
    </div>
    <div class="line">
      <span class="pill">场景：{{ sceneText }}</span>
      <span class="pill">{{ remoteText }}</span>
      <span v-if="busy" class="tip">更新中…</span>
    </div>
    <p v-if="errText" class="err" :title="errText">状态读取失败：{{ errText }}</p>
  </div>
</template>

<style scoped>
.status-wrap {
  width: 100%;
  min-height: 52px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 6px 8px;
  box-sizing: border-box;
  border: 1px solid var(--border-light, #ddd2c4);
  border-radius: 10px;
  background: var(--bg-elevated, #f7f2ea);
  color: var(--text-secondary, #5c564c);
  font-size: 12px;
}
.line {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.pill {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 1px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light, #ddd2c4);
  background: var(--bg-primary, #fffdf9);
  color: var(--text-primary, #3f3a33);
}
.tip {
  color: var(--text-secondary, #7a7369);
  font-size: 11px;
  align-self: center;
}
.err {
  margin: 0;
  color: var(--text-danger, #c33);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
