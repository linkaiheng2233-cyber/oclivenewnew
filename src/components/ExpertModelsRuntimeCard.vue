<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import { useExpertModelsStore } from "../stores/expertModelsStore";
import { openExpertWorkbenchEdit } from "../lib/expertWorkbenchOpen";
import { useRoleStore } from "../stores/roleStore";
import {
  collectExpertRecipeParts,
  formatExpertConfigDetailJson,
  resolveExpertRecipeUiMode,
} from "../lib/expertRecipeUi";

const props = withDefaults(
  defineProps<{
    /** `embedded`：角色运行时侧栏；`pmSection`：插件管理 V1 后端页 */
    layout?: "embedded" | "pmSection";
  }>(),
  { layout: "embedded" },
);

const { t } = useI18n();
const { showToast } = useAppToast();
const roleStore = useRoleStore();
const expertStore = useExpertModelsStore();

const detailOpen = ref(false);
const resetBusy = ref(false);

async function pullEffective(): Promise<void> {
  const rid = (roleStore.currentRoleId ?? "").trim();
  if (!rid) return;
  await expertStore.refresh();
}

watch(
  () => roleStore.currentRoleId,
  () => {
    void pullEffective();
  },
  { immediate: true },
);

const recipeMode = computed(() =>
  resolveExpertRecipeUiMode(expertStore.graphSource, expertStore.effectiveGraph),
);

const statusTitle = computed(() => {
  if (recipeMode.value === "pure") return String(t("expertRuntimeCard.status.pureTitle"));
  if (recipeMode.value === "role_default") return String(t("expertRuntimeCard.status.roleDefaultTitle"));
  return String(t("expertRuntimeCard.status.sessionTitle"));
});

function formatRecipeBitsForCard(): string {
  const p = collectExpertRecipeParts(expertStore.effectiveGraph);
  const bits: string[] = [];
  if (p.cloudModels.length) {
    bits.push(String(t("expertRuntimeCard.summary.cloud", { text: p.cloudModels.join(" · ") })));
  }
  if (p.loraBits.length) {
    bits.push(String(t("expertRuntimeCard.summary.lora", { text: p.loraBits.join("，") })));
  }
  if (p.baseModels.length) {
    bits.push(String(t("expertRuntimeCard.summary.base", { text: p.baseModels.join(" · ") })));
  }
  if (p.eventTriggerCount > 0) {
    bits.push(String(t("expertRuntimeCard.summary.events", { n: p.eventTriggerCount })));
  }
  return bits.length ? bits.join(String(t("expertRuntimeCard.summary.sep"))) : String(t("expertRuntimeCard.summary.empty"));
}

const statusBody = computed(() => {
  if (recipeMode.value === "pure") return String(t("expertRuntimeCard.status.pureBody"));
  const bits = formatRecipeBitsForCard();
  const empty = String(t("expertRuntimeCard.summary.empty"));
  const detail = bits && bits !== empty ? `\n${bits}` : "";
  if (recipeMode.value === "role_default") {
    return String(t("expertRuntimeCard.status.roleDefaultBody", { detail }));
  }
  return String(t("expertRuntimeCard.status.sessionBody", { detail }));
});

const detailJson = computed(() =>
  formatExpertConfigDetailJson(expertStore.effectiveGraph, expertStore.effectivePromptStyle),
);

function openWorkbench(): void {
  openExpertWorkbenchEdit({
    draftMode: recipeMode.value === "role_default" ? "role_default" : "effective",
  });
}

async function onResetSession(): Promise<void> {
  if (recipeMode.value !== "session_override") return;
  const ok = window.confirm(String(t("expertRuntimeCard.confirmResetSession")));
  if (!ok) return;
  resetBusy.value = true;
  try {
    const r = await expertStore.clearSessionOverrideAndApply();
    await pullEffective();
    if (r.ok) {
      showToast("success", String(t("expertRuntimeCard.toastResetOk")));
    } else {
      showToast("error", String(t("expertRuntimeCard.toastResetApplyWarn")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    resetBusy.value = false;
  }
}

const rootClass = computed(() =>
  props.layout === "pmSection" ? "expert-runtime expert-runtime--pm" : "expert-runtime expert-runtime--embed",
);
</script>

<template>
  <section :class="rootClass" :aria-label="String(t('expertRuntimeCard.aria'))">
    <div class="expert-runtime__head">
      <h3 class="expert-runtime__title">{{ t("expertRuntimeCard.title") }}</h3>
      <span
        class="expert-runtime__pill"
        :class="{
          'expert-runtime__pill--pure': recipeMode === 'pure',
          'expert-runtime__pill--role': recipeMode === 'role_default',
          'expert-runtime__pill--sess': recipeMode === 'session_override',
        }"
      >
        {{
          recipeMode === "pure"
            ? t("expertRuntimeCard.pill.pure")
            : recipeMode === "role_default"
              ? t("expertRuntimeCard.pill.roleDefault")
              : t("expertRuntimeCard.pill.session")
        }}
      </span>
    </div>
    <p v-if="expertStore.loading" class="expert-runtime__muted">{{ t("expertRuntimeCard.loading") }}</p>
    <p v-else-if="expertStore.error" class="expert-runtime__err">{{ expertStore.error }}</p>
    <template v-else>
      <p class="expert-runtime__status-title">{{ statusTitle }}</p>
      <p class="expert-runtime__status-body">{{ statusBody }}</p>
      <div class="expert-runtime__actions">
        <button type="button" class="expert-runtime__btn" @click="detailOpen = true">
          {{ t("expertRuntimeCard.btnDetail") }}
        </button>
        <button
          type="button"
          class="expert-runtime__btn expert-runtime__btn--primary"
          :title="String(t('expertWorkbench.editButtonTitle'))"
          @click="openWorkbench"
        >
          {{ t("expertWorkbench.editButton") }}
        </button>
        <button
          v-if="recipeMode === 'session_override'"
          type="button"
          class="expert-runtime__btn expert-runtime__btn--danger"
          :disabled="resetBusy"
          @click="onResetSession"
        >
          {{ resetBusy ? t("expertRuntimeCard.resetting") : t("expertRuntimeCard.btnReset") }}
        </button>
      </div>
    </template>

    <Teleport to="body">
      <div
        v-if="detailOpen"
        class="expert-runtime__backdrop"
        role="dialog"
        aria-modal="true"
        @click="detailOpen = false"
      >
        <div class="expert-runtime__modal" @click.stop>
          <h4 class="expert-runtime__modal-title">{{ t("expertRuntimeCard.detailTitle") }}</h4>
          <pre class="expert-runtime__pre">{{ detailJson }}</pre>
          <div class="expert-runtime__modal-actions">
            <button type="button" class="expert-runtime__btn" @click="detailOpen = false">
              {{ t("common.close") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.expert-runtime {
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 10px 12px;
  background: var(--bg-elevated, rgba(255, 255, 255, 0.04));
}
.expert-runtime--embed {
  margin-bottom: 12px;
}
.expert-runtime--pm {
  margin-top: 0;
}
.expert-runtime__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
}
.expert-runtime__title {
  margin: 0;
  font-size: 13px;
  font-weight: 650;
  color: var(--text-primary);
}
.expert-runtime__pill {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 600;
  white-space: nowrap;
}
.expert-runtime__pill--pure {
  background: color-mix(in srgb, var(--text-secondary) 18%, transparent);
  color: var(--text-secondary);
}
.expert-runtime__pill--role {
  background: color-mix(in srgb, #3b82f6 22%, transparent);
  color: color-mix(in srgb, #1d4ed8 90%, var(--text-primary));
}
.expert-runtime__pill--sess {
  background: color-mix(in srgb, #a855f7 24%, transparent);
  color: color-mix(in srgb, #6b21a8 88%, var(--text-primary));
}
.expert-runtime__muted,
.expert-runtime__err {
  margin: 4px 0 0;
  font-size: 12px;
}
.expert-runtime__err {
  color: var(--danger, #f87171);
}
.expert-runtime__status-title {
  margin: 6px 0 4px;
  font-size: 12px;
  font-weight: 650;
  color: var(--text-primary);
}
.expert-runtime__status-body {
  margin: 0 0 8px;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
  word-break: break-word;
}
.expert-runtime__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.expert-runtime__btn {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}
.expert-runtime__btn:hover {
  border-color: var(--accent, #6b8cff);
}
.expert-runtime__btn--primary {
  border-color: transparent;
  background: var(--accent, #6b8cff);
  color: #fff;
}
.expert-runtime__btn--danger {
  border-color: color-mix(in srgb, #ef4444 45%, var(--border-light));
  color: color-mix(in srgb, #fecaca 90%, var(--text-primary));
  background: color-mix(in srgb, #ef4444 12%, var(--bg-primary));
}
.expert-runtime__btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.expert-runtime__backdrop {
  position: fixed;
  inset: 0;
  z-index: 10003;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.55));
}
.expert-runtime__modal {
  width: min(720px, 100%);
  max-height: min(80vh, 720px);
  display: flex;
  flex-direction: column;
  padding: 14px;
  border-radius: 12px;
  background: var(--bg-panel, #1a1a22);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.expert-runtime__modal-title {
  margin: 0 0 8px;
  font-size: 14px;
  font-weight: 650;
}
.expert-runtime__pre {
  flex: 1;
  min-height: 0;
  overflow: auto;
  margin: 0;
  padding: 10px;
  border-radius: 8px;
  background: var(--bg-primary);
  font-size: 11px;
  line-height: 1.4;
  border: 1px solid var(--border-light);
}
.expert-runtime__modal-actions {
  margin-top: 10px;
  display: flex;
  justify-content: flex-end;
}
</style>
