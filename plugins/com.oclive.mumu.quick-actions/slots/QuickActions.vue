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

const EVT_SEND = "com.oclive.mumu.quick-actions:send_phrase";
const EVT_TRAVEL = "com.oclive.mumu.quick-actions:travel";

const oclive = inject<OcliveApi | null>("oclive", null);
const roleId = ref("");
const scenes = ref<Array<{ id: string; label: string }>>([]);
const selectedSceneId = ref("");
const errText = ref("");
const loading = ref(false);

const quickPhrases = [
  { label: "软一点", text: "你刚才那句再说一遍，我认真听着呢。" },
  { label: "日常聊", text: "今天你这边怎么样，挑一件小事和我说说？" },
  { label: "嘴硬版", text: "我也不是特地等你啦……你先说你想聊啥。" },
] as const;

const sceneOptions = computed(() => scenes.value);

function emitPhrase(text: string): void {
  if (!oclive || !text.trim()) return;
  oclive.events.emit(EVT_SEND, { text: text.trim() });
}

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
  void refresh();
});

onUnmounted(() => {
  if (!oclive) return;
  oclive.events.off("oclive:role:switched", onRoleSwitched);
});
</script>

<template>
  <div class="qa-wrap">
    <div class="group">
      <button
        v-for="p in quickPhrases"
        :key="p.label"
        type="button"
        class="btn"
        :disabled="loading"
        :title="p.text"
        @click="emitPhrase(p.text)"
      >
        {{ p.label }}
      </button>
    </div>
    <div class="group">
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
    <p v-if="errText" class="err" :title="errText">快捷动作不可用：{{ errText }}</p>
  </div>
</template>

<style scoped>
.qa-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 10px;
  align-items: center;
  width: 100%;
  min-height: 34px;
}
.group {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}
.btn,
.sel {
  min-height: 30px;
  border-radius: var(--radius-btn, 8px);
  border: 1px solid var(--border-light, #ddd2c4);
  background: var(--bg-elevated, #f7f2ea);
  color: var(--text-primary, #3f3a33);
  font-size: 12px;
  padding: 4px 10px;
}
.btn {
  cursor: pointer;
}
.btn:hover {
  border-color: var(--accent, #8f7f6a);
}
.btn-primary {
  background: linear-gradient(135deg, var(--btn-grad-a, #c7b79f), var(--btn-grad-b, #b6a189));
  color: var(--text-accent, #2a241d);
  border-color: var(--accent, #8f7f6a);
}
.sel {
  min-width: 130px;
}
.err {
  margin: 0;
  font-size: 11px;
  color: var(--text-danger, #c33);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: min(560px, 100%);
}
</style>
