<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
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
const { t } = useI18n();
const localFactor = ref(roleStore.roleInfo.eventImpactFactor);
const busy = ref(false);
const feedbackOpen = ref(false);
const feedbackBusy = ref(false);
const feedbackMood = ref("");
const feedbackMessage = ref("");

const personalitySourceLabel = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? t("roleRuntimePanel.personality.sourceLabel.profile")
    : t("roleRuntimePanel.personality.sourceLabel.vector"),
);
const personalitySourceHintParagraphs = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? [
        t("roleRuntimePanel.personality.hints.profileP1"),
        t("roleRuntimePanel.personality.hints.profileP2"),
      ]
    : [
        t("roleRuntimePanel.personality.hints.vectorP1"),
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
    showToast("success", t("roleRuntimePanel.toasts.submitted"));
    closeFeedback();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast("error", msg || t("roleRuntimePanel.toasts.submitFailed"));
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
        {{
          t("roleRuntimePanel.meta.versionAuthor", {
            version: roleStore.roleInfo.version || "—",
            author: roleStore.roleInfo.author || "—",
          })
        }}
      </p>
      <p class="sub personality-source-line">
        <span class="ps-inline">
          {{ t("roleRuntimePanel.personality.sourceLabelTitle") }}：<strong>{{ personalitySourceLabel }}</strong>
          <HelpHint :paragraphs="personalitySourceHintParagraphs" />
        </span>
      </p>
    </div>
    <div class="runtime-backend-hint">
      <p class="sub">
        {{ t("roleRuntimePanel.backendHint.prefix") }}
        <button type="button" class="link-open-backends" @click="openBackendsPanel">
          {{ t("roleRuntimePanel.backendHint.linkText") }}
        </button>
        {{ t("roleRuntimePanel.backendHint.suffix") }}
      </p>
    </div>
    <div class="runtime-feedback">
      <p class="sub" v-html="t('roleRuntimePanel.feedback.leadHtml')" />
      <button type="button" class="btn-feedback" @click="openFeedback">
        {{ t("roleRuntimePanel.feedback.openButton") }}
      </button>
    </div>
    <template v-if="roleStore.roleInfo.userRelations.length > 0">
      <div class="row">
        <label for="rel-select">{{ t("roleRuntimePanel.fields.relation") }}</label>
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
        <label for="evolve-factor">{{ t("roleRuntimePanel.fields.eventImpact") }}</label>
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
        <h2 class="modal-title">{{ t("roleRuntimePanel.feedbackModal.title") }}</h2>
        <p class="modal-sub">{{ t("roleRuntimePanel.feedbackModal.sub") }}</p>
        <div class="modal-row">
          <label class="modal-label">{{ t("roleRuntimePanel.feedbackModal.moodLabel") }}</label>
          <input
            v-model="feedbackMood"
            class="modal-input"
            type="text"
            :placeholder="String(t('roleRuntimePanel.feedbackModal.moodPlaceholder'))"
            :disabled="feedbackBusy"
          />
        </div>
        <div class="modal-row">
          <label class="modal-label">{{ t("roleRuntimePanel.feedbackModal.messageLabel") }}</label>
          <textarea
            v-model="feedbackMessage"
            class="modal-textarea"
            rows="4"
            :placeholder="String(t('roleRuntimePanel.feedbackModal.messagePlaceholder'))"
            :disabled="feedbackBusy"
          />
        </div>
        <div class="modal-actions">
          <button type="button" class="btn-secondary" :disabled="feedbackBusy" @click="closeFeedback">
            {{ t("common.cancel") }}
          </button>
          <button
            type="button"
            class="btn-primary"
            :disabled="feedbackBusy || !feedbackMessage.trim()"
            @click="submitFeedback"
          >
            {{
              feedbackBusy
                ? t("roleRuntimePanel.feedbackModal.submitting")
                : t("roleRuntimePanel.feedbackModal.submit")
            }}
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
