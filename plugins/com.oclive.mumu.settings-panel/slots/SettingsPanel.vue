<script setup lang="ts">
import { inject, onMounted, onUnmounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: {
    emit(event: string, data?: unknown): void;
    on(event: string, handler: (data: unknown) => void): void;
    off(event: string, handler: (data: unknown) => void): void;
  };
};

type RoleSummary = { id: string; name: string };
type RoleInfoPayload = {
  role_id: string;
  role_name?: string;
  remote_life_enabled?: boolean;
  interaction_mode?: "immersive" | "pure_chat";
};

const EVT_SET_REMOTE = "com.oclive.mumu.settings-panel:set_remote_life";
const EVT_SET_MODE = "com.oclive.mumu.settings-panel:set_interaction_mode";
const EVT_CYCLE_THEME = "com.oclive.mumu.settings-panel:cycle_theme";
const EVT_REQUEST_RESET_LAYOUT = "com.oclive.mumu.settings-panel:request_reset_layout";
const EVT_RESET_LAYOUT_RESULT = "com.oclive.mumu.settings-panel:reset_layout_result";

const oclive = inject<OcliveApi | null>("oclive", null);
const roleId = ref("");
const roleName = ref("沐沐");
const remoteLife = ref(false);
const interactionMode = ref<"immersive" | "pure_chat">("immersive");
const loading = ref(false);
const err = ref("");
const actionBusy = ref(false);
const actionHint = ref("");
const actionType = ref<"success" | "error" | "info">("info");

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
  loading.value = true;
  err.value = "";
  try {
    const rid = (nextRoleId ?? roleId.value).trim() || (await resolveCurrentRoleId());
    if (!rid) {
      roleId.value = "";
      roleName.value = "沐沐";
      remoteLife.value = false;
      interactionMode.value = "immersive";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    roleId.value = rid;
    roleName.value = (info.role_name ?? "").trim() || "沐沐";
    remoteLife.value = info.remote_life_enabled === true;
    interactionMode.value =
      info.interaction_mode === "pure_chat" ? "pure_chat" : "immersive";
  } catch (e) {
    err.value = e instanceof Error ? e.message : String(e);
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

function onToggleRemoteLife(ev: Event): void {
  const checked = (ev.target as HTMLInputElement).checked;
  remoteLife.value = checked;
  if (!oclive) return;
  oclive.events.emit(EVT_SET_REMOTE, { enabled: checked });
}

function onModeChange(ev: Event): void {
  const value = (ev.target as HTMLSelectElement).value;
  const mode = value === "pure_chat" ? "pure_chat" : "immersive";
  interactionMode.value = mode;
  if (!oclive) return;
  oclive.events.emit(EVT_SET_MODE, { mode });
}

function onCycleTheme(): void {
  if (!oclive) return;
  oclive.events.emit(EVT_CYCLE_THEME, {});
}

function onResetToPackDefault(): void {
  if (!oclive || actionBusy.value) return;
  const approved = window.confirm(
    "将恢复当前角色包 ui.json 推荐布局，并覆盖你在插件管理里的插槽显示/排序。继续吗？",
  );
  if (!approved) {
    actionType.value = "info";
    actionHint.value = "已取消恢复默认布局。";
    return;
  }
  actionBusy.value = true;
  actionType.value = "info";
  actionHint.value = "正在请求恢复默认布局…";
  oclive.events.emit(EVT_REQUEST_RESET_LAYOUT, {});
}

function onResetResult(payload: unknown): void {
  const ok = (payload as { ok?: boolean } | null)?.ok === true;
  const msg = (payload as { message?: string } | null)?.message;
  actionBusy.value = false;
  actionType.value = ok ? "success" : "error";
  actionHint.value =
    typeof msg === "string" && msg.trim().length > 0
      ? msg.trim()
      : ok
        ? "已恢复为角色包推荐布局。"
        : "恢复默认布局失败，请稍后重试。";
}

onMounted(() => {
  if (!oclive) return;
  oclive.events.on("oclive:role:switched", onRoleSwitched);
  oclive.events.on("oclive:message:sent", onHostRefresh);
  oclive.events.on("oclive:theme:changed", onHostRefresh);
  oclive.events.on(EVT_RESET_LAYOUT_RESULT, onResetResult);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:message:sent", onHostRefresh);
  oclive.events.off("oclive:theme:changed", onHostRefresh);
  oclive.events.off(EVT_RESET_LAYOUT_RESULT, onResetResult);
});
</script>

<template>
  <section class="panel" aria-label="mumu 外观与互动设置">
    <header class="head">
      <div>
        <h3>{{ roleName }} 设置</h3>
        <p>{{ roleId || "未加载角色" }}</p>
      </div>
      <span v-if="loading" class="badge">同步中</span>
    </header>

    <div class="row">
      <div class="left">
        <strong>异地心声</strong>
        <small>异地时启用生活轨迹风格回复</small>
      </div>
      <label class="switch" aria-label="异地心声开关">
        <input type="checkbox" :checked="remoteLife" :disabled="loading" @change="onToggleRemoteLife" />
        <span />
      </label>
    </div>

    <div class="row">
      <div class="left">
        <strong>互动模式</strong>
        <small>沉浸含场景/时间，纯聊仅保留对话</small>
      </div>
      <select class="select" :value="interactionMode" :disabled="loading" @change="onModeChange">
        <option value="immersive">沉浸</option>
        <option value="pure_chat">纯聊</option>
      </select>
    </div>

    <div class="row row-theme">
      <div class="left">
        <strong>主题外观</strong>
        <small>循环：跟随系统 / 浅色 / 深色</small>
      </div>
      <button type="button" class="btn" :disabled="loading" @click="onCycleTheme">
        切换主题
      </button>
    </div>

    <div class="row row-danger">
      <div class="left">
        <strong>恢复默认布局</strong>
        <small>覆盖本角色的插件可见性与排序，回到 ui.json 推荐值</small>
      </div>
      <button type="button" class="btn btn-danger" :disabled="loading || actionBusy" @click="onResetToPackDefault">
        {{ actionBusy ? "处理中…" : "恢复默认" }}
      </button>
    </div>

    <p
      v-if="actionHint"
      class="hint"
      :class="{
        'hint--ok': actionType === 'success',
        'hint--err': actionType === 'error',
      }"
    >
      {{ actionHint }}
    </p>

    <p v-if="err" class="err" :title="err">读取失败：{{ err }}</p>
  </section>
</template>

<style scoped>
.panel {
  --ui-trans-fast: 140ms;
  --ui-state-warn-fg: color-mix(in srgb, var(--accent, #8f7f6a) 88%, black 12%);
  --ui-state-warn-bg: color-mix(in srgb, var(--accent, #8f7f6a) 12%, transparent);
  --ui-state-warn-border: color-mix(in srgb, var(--accent, #8f7f6a) 40%, transparent);
  --ui-state-danger-fg: var(--text-danger, #c33);
  width: 100%;
  box-sizing: border-box;
  padding: 14px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 68%, transparent);
  background:
    linear-gradient(
      170deg,
      color-mix(in srgb, var(--bg-primary, #fffdf9) 82%, white 18%),
      color-mix(in srgb, var(--bg-elevated, #f7f2ea) 88%, white 12%)
    );
  backdrop-filter: blur(14px) saturate(110%);
  -webkit-backdrop-filter: blur(14px) saturate(110%);
  box-shadow:
    0 8px 22px color-mix(in srgb, var(--text-primary, #3f3a33) 7%, transparent),
    inset 0 1px 0 color-mix(in srgb, white 70%, transparent);
  color: var(--text-primary, #3f3a33);
}
.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
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
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-top: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 65%, transparent);
}
.row:first-of-type {
  border-top: none;
  padding-top: 2px;
}
.left {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.left strong {
  font-size: 12px;
  font-weight: 600;
}
.left small {
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
  line-height: 1.3;
}
.select,
.btn {
  min-height: 30px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 78%, transparent);
  background: color-mix(in srgb, var(--bg-primary, #fffdf9) 92%, transparent);
  color: var(--text-primary, #3f3a33);
  font-size: 12px;
  padding: 4px 10px;
  transition:
    border-color var(--ui-trans-fast) ease,
    transform var(--ui-trans-fast) ease,
    filter var(--ui-trans-fast) ease;
}
.btn {
  cursor: pointer;
}
.btn:hover {
  border-color: color-mix(in srgb, var(--accent, #8f7f6a) 55%, transparent);
  transform: translateY(-0.5px);
}
.switch {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.switch input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}
.switch span {
  width: 44px;
  height: 25px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd2c4) 80%, transparent);
  background: color-mix(in srgb, var(--bg-elevated, #f7f2ea) 90%, transparent);
  position: relative;
  transition: all var(--ui-trans-fast) ease;
}
.switch span::before {
  content: "";
  position: absolute;
  top: 2px;
  left: 2px;
  width: 19px;
  height: 19px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
  transition: transform var(--ui-trans-fast) ease;
}
.switch input:checked + span {
  background: color-mix(in srgb, var(--accent, #8f7f6a) 70%, white 30%);
  border-color: color-mix(in srgb, var(--accent, #8f7f6a) 65%, transparent);
}
.switch input:checked + span::before {
  transform: translateX(19px);
}
.row-theme .btn {
  min-width: 88px;
}
.btn-danger {
  border-color: color-mix(in srgb, var(--ui-state-danger-fg) 45%, transparent);
  color: var(--ui-state-danger-fg);
}
.btn-danger:hover {
  border-color: color-mix(in srgb, var(--ui-state-danger-fg) 70%, transparent);
}
.hint {
  margin: 8px 0 0;
  font-size: 11px;
  color: var(--text-secondary, #736a5e);
}
.hint--ok {
  color: color-mix(in srgb, var(--accent, #8f7f6a) 80%, black 20%);
}
.hint--err {
  color: var(--ui-state-danger-fg);
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
