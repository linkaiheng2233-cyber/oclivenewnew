<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: {
    emit(event: string, data?: unknown): void;
    on(event: string, handler: (data: unknown) => void): void;
    off(event: string, handler: (data: unknown) => void): void;
  };
};

type RoleSummary = { id: string; name: string };
type SceneLabel = { id: string; label: string };
type RoleInfoPayload = {
  role_id: string;
  scene_labels?: SceneLabel[];
  scenes?: string[];
  user_presence_scene?: string | null;
};

const EVT_TRAVEL = "com.oclive.mumu.quick-actions:travel";

const oclive = inject<OcliveApi | null>("oclive", null);
const roleId = ref("");
const scenes = ref<Array<{ id: string; label: string }>>([]);
const selectedSceneId = ref("");
const errText = ref("");
const loading = ref(false);

const sceneOptions = computed(() => scenes.value);

function emitTravel(together: boolean): void {
  if (!oclive) return;
  const sceneId = selectedSceneId.value.trim();
  if (!sceneId) return;
  oclive.events.emit(EVT_TRAVEL, { sceneId, together });
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
  loading.value = true;
  errText.value = "";
  try {
    const rid = (nextRoleId ?? roleId.value).trim() || (await resolveCurrentRoleId());
    if (!rid) {
      roleId.value = "";
      scenes.value = [];
      selectedSceneId.value = "";
      return;
    }
    const info = (await oclive.invoke("get_role_info", {
      role_id: rid,
      session_id: null,
    })) as RoleInfoPayload;
    roleId.value = rid;
    const labels = Array.isArray(info.scene_labels) ? info.scene_labels : [];
    const fallback = Array.isArray(info.scenes)
      ? info.scenes.map((id) => ({ id, label: id }))
      : [];
    const merged = (labels.length > 0 ? labels : fallback).filter(
      (x) => typeof x?.id === "string" && x.id.trim().length > 0,
    );
    scenes.value = merged;
    const preferred = (info.user_presence_scene ?? "").trim();
    const first = merged[0]?.id ?? "";
    selectedSceneId.value = merged.some((x) => x.id === preferred) ? preferred : first;
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

onMounted(() => {
  if (!oclive) return;
  oclive.events.on("oclive:role:switched", onRoleSwitched);
  oclive.events.on("oclive:role:info:updated", onRoleSwitched);
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
  oclive.events.off("oclive:role:info:updated", onRoleSwitched);
});
</script>

<template>
  <section class="panel mumu-surface" aria-label="场景位移">
    <p class="panel-lede">仅切换叙事所在，不代你发言。</p>
    <div class="group group-travel">
      <select v-model="selectedSceneId" class="sel" :disabled="loading || sceneOptions.length === 0">
        <option v-for="s in sceneOptions" :key="s.id" :value="s.id">
          {{ s.label }}
        </option>
      </select>
      <button
        type="button"
        class="btn"
        :disabled="loading || !selectedSceneId"
        @click="emitTravel(false)"
      >
        仅我过去
      </button>
      <button
        type="button"
        class="btn btn-primary"
        :disabled="loading || !selectedSceneId"
        @click="emitTravel(true)"
      >
        同行前往
      </button>
    </div>
    <p v-if="errText" class="err" :title="errText">场景位移不可用：{{ errText }}</p>
  </section>
</template>

<style scoped>
.panel {
  --ui-trans-fast: 140ms;
}
.mumu-surface {
  font-family: var(--font-ui);
  font-size: 0.875rem;
  line-height: 1.45;
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
  box-sizing: border-box;
  padding: 0.625rem;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: var(--shadow-sm), var(--frame-inset-highlight);
  backdrop-filter: blur(12px) saturate(110%);
  -webkit-backdrop-filter: blur(12px) saturate(110%);
  -webkit-font-smoothing: antialiased;
}
.panel-lede {
  margin: 0;
  font-size: 0.65625rem;
  line-height: 1.45;
  color: var(--text-secondary);
}
.group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  align-items: center;
  width: 100%;
}
.group-travel {
  justify-content: flex-start;
}
.btn,
.sel {
  min-height: 1.9375rem;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.71875rem;
  padding: 0.25rem 0.625rem;
  transition:
    border-color var(--ui-trans-fast) ease,
    transform var(--ui-trans-fast) ease,
    filter var(--ui-trans-fast) ease;
}
.btn {
  cursor: pointer;
}
.btn:hover {
  border-color: color-mix(in srgb, var(--accent) 55%, var(--border-light) 45%);
  transform: translateY(-0.5px);
}
.btn-primary {
  background: linear-gradient(140deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  border-color: color-mix(in srgb, var(--accent) 58%, var(--border-light) 42%);
}
.btn:disabled,
.sel:disabled {
  cursor: not-allowed;
  opacity: 0.62;
  transform: none;
}
.btn:disabled:hover {
  border-color: var(--border-light);
}
.sel {
  min-width: 9.375rem;
}
.err {
  margin: 0;
  font-size: 0.6875rem;
  color: var(--error);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: min(35rem, 100%);
}
</style>
