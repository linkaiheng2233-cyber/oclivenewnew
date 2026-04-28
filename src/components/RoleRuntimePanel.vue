<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { buildRelationDropdownOptions } from "../utils/relationOptions";
import {
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  createRoleFeedback,
  setEvolutionFactor,
  setUserRelation,
} from "../utils/tauri-api";
import { useAppToast } from "../composables/useAppToast";
import HelpHint from "./HelpHint.vue";

const roleStore = useRoleStore();
const uiStore = useUiStore();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const localFactor = ref(roleStore.roleInfo.eventImpactFactor);
const busy = ref(false);
const feedbackOpen = ref(false);
const feedbackBusy = ref(false);
const feedbackMood = ref("");
const feedbackMessage = ref("");

const personalitySourceLabel = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? "档案（可变正文由对话维护）"
    : "七维向量",
);
const personalitySourceHintParagraphs = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? [
        "人格来源为 profile：运行时以核心性格档案与数据库中的「可变性格档案」为准；界面七维多为从正文归纳的视图。",
        "与 vector 模式（七维直接参与事件演化）不同；设计说明见仓库 docs/personality-archive-notes.md。",
      ]
    : [
        "人格来源为 vector：事件与情绪按七维精细化调整；与 settings 中 evolution.personality_source 一致。",
      ],
);
const relationRows = computed(() =>
  buildRelationDropdownOptions(
    roleStore.roleInfo.userRelations,
    roleStore.roleInfo.defaultRelation,
  ),
);
watch(
  () => [roleStore.currentRoleId, roleStore.roleInfo.eventImpactFactor] as const,
  () => {
    localFactor.value = roleStore.roleInfo.eventImpactFactor;
  },
);
async function onRelationChange(ev: Event) {
  const next = (ev.target as HTMLSelectElement).value;
  if (next === roleStore.relationSelectValue) return;
  busy.value = true;
  try {
    const perScene = roleStore.roleInfo.identityBinding === "per_scene";
    if (next === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) await roleStore.setManifestDefaultIdentity(uiStore.sceneId);
      else await roleStore.setManifestDefaultIdentity();
    } else if (perScene) {
      await roleStore.setSceneUserRelation(uiStore.sceneId, next);
    } else {
      const info = await setUserRelation(roleStore.currentRoleId, next);
      roleStore.applyRoleInfo(info);
    }
  } finally {
    busy.value = false;
  }
}
async function commitFactor() {
  const v = localFactor.value;
  if (
    !Number.isFinite(v) ||
    v < 0.05 ||
    v > 5 ||
    Math.abs(v - roleStore.roleInfo.eventImpactFactor) < 1e-9
  ) {
    return;
  }
  busy.value = true;
  try {
    await setEvolutionFactor(roleStore.currentRoleId, v);
    await roleStore.refreshRoleInfo();
  } finally {
    busy.value = false;
  }
}
function onFactorEnter(ev: KeyboardEvent) {
  (ev.target as HTMLInputElement).blur();
}
function openBackendsPanel(): void {
  void pluginStore.openPanel("backends");
}

function openFeedback(): void {
  feedbackOpen.value = true;
  feedbackMood.value = "";
  feedbackMessage.value = "";
}

function closeFeedback(): void {
  feedbackOpen.value = false;
}

async function submitFeedback(): Promise<void> {
  if (feedbackBusy.value) return;
  feedbackBusy.value = true;
  try {
    await createRoleFeedback({
      role_id: roleStore.currentRoleId,
      mood_tag: feedbackMood.value.trim() ? feedbackMood.value.trim() : null,
      scene_id: uiStore.sceneId ?? null,
      presence_mode: (roleStore.roleInfo as any)?.presenceMode ?? null,
      role_version: roleStore.roleInfo.version ?? null,
      message: feedbackMessage.value,
    });
    showToast("success", "已提交反馈（仅创作者可见）。");
    closeFeedback();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast("error", msg || "提交失败");
  } finally {
    feedbackBusy.value = false;
  }
}
</script>

<template>
  <section class="runtime">
    <div class="meta">
      <p v-if="roleStore.roleInfo.description" class="desc">{{ roleStore.roleInfo.description }}</p>
      <p class="sub">
        版本 {{ roleStore.roleInfo.version || "—" }} · 作者 {{ roleStore.roleInfo.author || "—" }}
      </p>
      <p class="sub personality-source-line">
        <span class="ps-inline">
          人格来源：<strong>{{ personalitySourceLabel }}</strong>
          <HelpHint :paragraphs="personalitySourceHintParagraphs" />
        </span>
      </p>
    </div>
    <div class="runtime-backend-hint">
      <p class="sub">
        模块后端、异地心声、会话覆盖与调试快照已迁至
        <button type="button" class="link-open-backends" @click="openBackendsPanel">
          插件与后端管理 → 后端模块
        </button>
        （Ctrl+Shift+F）
      </p>
    </div>
    <div class="runtime-feedback">
      <p class="sub">
        用完觉得哪里不对？可以提交一条<strong>半私密反馈</strong>给创作者（本地保存，不公开展示）。
      </p>
      <button type="button" class="btn-feedback" @click="openFeedback">反馈此角色包</button>
    </div>
    <template v-if="roleStore.roleInfo.userRelations.length > 0">
      <div class="row">
        <label for="rel-select">关系</label>
        <select
          id="rel-select"
          class="select"
          :disabled="busy"
          :value="roleStore.relationSelectValue"
          @change="onRelationChange"
        >
          <option v-for="r in relationRows" :key="r.id" :value="r.id">{{ r.name || r.id }}</option>
        </select>
      </div>
      <div class="row">
        <label for="evolve-factor">事件影响</label>
        <input
          id="evolve-factor"
          v-model.number="localFactor"
          class="input-num"
          type="number"
          min="0.05"
          max="5"
          step="0.05"
          :disabled="busy"
          @blur="commitFactor"
          @keydown.enter.prevent="onFactorEnter"
        />
      </div>
    </template>
  </section>

  <Teleport to="body">
    <div
      v-if="feedbackOpen"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      @click="closeFeedback"
    >
      <div class="modal-card modal-card--wide" @click.stop>
        <h2 class="modal-title">反馈此角色包</h2>
        <p class="modal-sub">
          这条反馈默认仅创作者可见（半私密），用于迭代角色包。请避免填写个人隐私信息。
        </p>
        <div class="modal-row">
          <label class="modal-label">情绪标签（可选）</label>
          <input
            v-model="feedbackMood"
            class="modal-input"
            type="text"
            placeholder="例如：开心 / 难过 / 生气 / 困惑 / 无"
            :disabled="feedbackBusy"
          />
        </div>
        <div class="modal-row">
          <label class="modal-label">留言</label>
          <textarea
            v-model="feedbackMessage"
            class="modal-textarea"
            rows="4"
            placeholder="写下你遇到的问题/建议（必填）"
            :disabled="feedbackBusy"
          />
        </div>
        <div class="modal-actions">
          <button type="button" class="btn-secondary" :disabled="feedbackBusy" @click="closeFeedback">
            取消
          </button>
          <button
            type="button"
            class="btn-primary"
            :disabled="feedbackBusy || !feedbackMessage.trim()"
            @click="submitFeedback"
          >
            {{ feedbackBusy ? "提交中…" : "提交反馈" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.runtime {
  padding: 10px 18px 12px;
  margin: 0;
  font-size: 13px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-light);
}
.meta {
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}
.desc {
  margin: 0 0 6px;
  line-height: 1.45;
  color: var(--text-secondary);
  font-size: 12px;
}
.sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.personality-source-line {
  margin-top: 8px;
}
.ps-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.runtime-backend-hint {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px dashed var(--border-light);
}
.runtime-feedback {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px dashed var(--border-light);
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: space-between;
  flex-wrap: wrap;
}
.btn-feedback {
  padding: 6px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
  font: inherit;
}
.btn-feedback:hover {
  border-color: var(--accent, #6b8cff);
}
.link-open-backends {
  margin: 0 2px;
  padding: 0;
  border: none;
  background: none;
  color: var(--accent, #6b8cff);
  text-decoration: underline;
  cursor: pointer;
  font: inherit;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
label {
  min-width: 72px;
  color: var(--text-secondary);
}
.select {
  flex: 1;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.input-num {
  width: 100px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10002;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.5));
}
.modal-card {
  width: 100%;
  max-width: 520px;
  padding: 18px 18px 14px;
  border-radius: 12px;
  background: var(--bg-panel, #1a1a22);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.modal-title {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 650;
  color: var(--text-primary);
}
.modal-sub {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.modal-row {
  margin-bottom: 10px;
}
.modal-label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.modal-input,
.modal-textarea {
  width: 100%;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font: inherit;
}
.modal-textarea {
  resize: vertical;
}
.modal-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 12px;
}
.btn-secondary,
.btn-primary {
  padding: 8px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  cursor: pointer;
  font: inherit;
}
.btn-secondary {
  background: transparent;
  color: var(--text-primary);
}
.btn-primary {
  background: var(--accent, #6b8cff);
  border-color: transparent;
  color: #fff;
}
.btn-primary:disabled,
.btn-secondary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
