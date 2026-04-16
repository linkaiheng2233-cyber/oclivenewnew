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
type SceneLabel = { id: string; label: string };
type RelationItem = { id: string; name: string };
type RoleInfoPayload = {
  role_id: string;
  role_name?: string;
  version?: string;
  relation_state?: string;
  interaction_mode?: "immersive" | "pure_chat";
  remote_life_enabled?: boolean;
  current_scene?: string | null;
  user_presence_scene?: string | null;
  current_life?: { label?: string | null } | null;
  scene_labels?: SceneLabel[];
  user_relations?: RelationItem[];
  current_user_relation?: string;
  default_relation?: string;
};

const oclive = inject<OcliveApi | null>("oclive", null);
const roleId = ref("");
const roleName = ref("沐沐");
const version = ref("—");
const relationStage = ref("—");
const relationName = ref("—");
const modeLabel = ref("沉浸");
const remoteLabel = ref("关");
const sceneLabel = ref("—");
const lifeLabel = ref("暂无日程");
const loading = ref(false);
const errText = ref("");

function safeTrim(v?: string | null): string {
  return typeof v === "string" ? v.trim() : "";
}

function humanRelationName(info: RoleInfoPayload): string {
  const current = safeTrim(info.current_user_relation);
  const fallback = safeTrim(info.default_relation);
  const target = current || fallback;
  if (!target) return "—";
  const rows = Array.isArray(info.user_relations) ? info.user_relations : [];
  const hit = rows.find((x) => x.id === target);
  return hit?.name ?? target;
}

function humanScene(info: RoleInfoPayload): string {
  const userScene = safeTrim(info.user_presence_scene);
  const roleScene = safeTrim(info.current_scene);
  const labels = new Map(
    (Array.isArray(info.scene_labels) ? info.scene_labels : []).map((x) => [x.id, x.label]),
  );
  const userLabel = labels.get(userScene) ?? userScene;
  const roleLabel = labels.get(roleScene) ?? roleScene;
  if (!userLabel && !roleLabel) return "—";
  if (userLabel && roleLabel && userLabel !== roleLabel) {
    return `${userLabel}（你） / ${roleLabel}（角色）`;
  }
  return userLabel || roleLabel;
}

function humanRemote(info: RoleInfoPayload): string {
  if (info.remote_life_enabled !== true) return "关";
  const userScene = safeTrim(info.user_presence_scene);
  const roleScene = safeTrim(info.current_scene);
  if (userScene && roleScene && userScene !== roleScene) {
    return "开（异地）";
  }
  return "开（同场）";
}

const compatibilityTip = computed(() => {
  if (version.value === "—") return "建议配套最新主程序，避免 ui_slots 字段不兼容。";
  if (version.value.startsWith("0.")) return "当前为迭代版角色包，发布前建议补齐变更日志。";
  return "版本结构正常，可继续做市场页说明与截图。";
});

const releaseChecklist = computed(() => [
  "核对 role.detail 显示字段与设定一致",
  "确认插件管理中该模块默认可见",
  "导出时同步更新 mumu 版本说明",
]);

async function resolveCurrentRoleId(): Promise<string> {
  if (!oclive) return "";
  const rows = (await oclive.invoke("list_roles", {})) as RoleSummary[];
  if (!Array.isArray(rows) || rows.length === 0) return "";
  return String(rows[0]?.id ?? "").trim();
}

async function refresh(nextRoleId?: string): Promise<void> {
  if (!oclive) return;
  loading.value = true;
  errText.value = "";
  try {
    const rid = safeTrim(nextRoleId ?? roleId.value) || (await resolveCurrentRoleId());
    if (!rid) {
      roleId.value = "";
      roleName.value = "沐沐";
      version.value = "—";
      relationStage.value = "—";
      relationName.value = "—";
      modeLabel.value = "沉浸";
      remoteLabel.value = "关";
      sceneLabel.value = "—";
      lifeLabel.value = "暂无日程";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    roleId.value = rid;
    roleName.value = safeTrim(info.role_name) || "沐沐";
    version.value = safeTrim(info.version) || "—";
    relationStage.value = safeTrim(info.relation_state) || "—";
    relationName.value = humanRelationName(info);
    modeLabel.value = info.interaction_mode === "pure_chat" ? "纯聊" : "沉浸";
    remoteLabel.value = humanRemote(info);
    sceneLabel.value = humanScene(info);
    lifeLabel.value = safeTrim(info.current_life?.label) || "暂无日程";
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function onRoleSwitched(payload: unknown): void {
  const next = (payload as { roleId?: string } | null)?.roleId;
  void refresh(typeof next === "string" ? next : undefined);
}

function onHostRefresh(): void {
  void refresh();
}

onMounted(() => {
  if (!oclive) return;
  oclive.events.on("oclive:role:switched", onRoleSwitched);
  oclive.events.on("oclive:message:sent", onHostRefresh);
  oclive.events.on("oclive:theme:changed", onHostRefresh);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:message:sent", onHostRefresh);
  oclive.events.off("oclive:theme:changed", onHostRefresh);
});
</script>

<template>
  <section class="card" aria-label="mumu 角色详情扩展卡片">
    <header class="head">
      <div>
        <h3>{{ roleName }}</h3>
        <p>{{ roleId || "未加载角色" }} · v{{ version }}</p>
      </div>
      <span v-if="loading" class="badge">同步中</span>
    </header>

    <dl class="meta">
      <div class="item">
        <dt>关系</dt>
        <dd>{{ relationName }}</dd>
      </div>
      <div class="item">
        <dt>阶段</dt>
        <dd>{{ relationStage }}</dd>
      </div>
      <div class="item">
        <dt>模式</dt>
        <dd>{{ modeLabel }}</dd>
      </div>
      <div class="item">
        <dt>异地心声</dt>
        <dd>{{ remoteLabel }}</dd>
      </div>
      <div class="item item-wide">
        <dt>场景</dt>
        <dd>{{ sceneLabel }}</dd>
      </div>
      <div class="item item-wide">
        <dt>此刻状态</dt>
        <dd>{{ lifeLabel }}</dd>
      </div>
    </dl>

    <div class="tips">
      <p class="tip">{{ compatibilityTip }}</p>
      <ul>
        <li v-for="line in releaseChecklist" :key="line">{{ line }}</li>
      </ul>
    </div>

    <p v-if="errText" class="err" :title="errText">读取失败：{{ errText }}</p>
  </section>
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
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 68%, transparent);
  background: linear-gradient(
    170deg,
    color-mix(in srgb, var(--bg-primary, #fffdf9) 82%, white 18%),
    color-mix(in srgb, var(--bg-elevated, #f7f2ea) 88%, white 12%)
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
  gap: 10px;
  margin-bottom: 10px;
}
.head h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 620;
}
.head p {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.badge {
  font-size: 10px;
  border-radius: 999px;
  padding: 2px 8px;
  border: 1px solid var(--ui-state-warn-border);
  color: var(--ui-state-warn-fg);
  background: var(--ui-state-warn-bg);
  white-space: nowrap;
}
.meta {
  margin: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.item {
  min-width: 0;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 70%, transparent);
  background: color-mix(in srgb, var(--bg-primary, #fffdf9) 90%, transparent);
  padding: 8px;
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
.tips {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 70%, transparent);
}
.tip {
  margin: 0 0 6px;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.tips ul {
  margin: 0;
  padding: 0 0 0 16px;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.tips li + li {
  margin-top: 2px;
}
.err {
  margin: 10px 0 0;
  color: var(--ui-state-danger-fg);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
