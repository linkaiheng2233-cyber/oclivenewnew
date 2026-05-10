<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../../composables/useAppToast";
import {
  collectExpertRecipeParts,
  formatExpertConfigDetailJson,
  resolveExpertRecipeUiMode,
} from "../../lib/expertRecipeUi";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import { useRoleStore } from "../../stores/roleStore";
import type { ExpertWorkbenchDraftMode } from "../../lib/expertWorkbenchOpen";

const props = defineProps<{
  active: boolean;
}>();

const emit = defineEmits<{
  openExpertWorkbench: [opts: { draftMode: ExpertWorkbenchDraftMode }];
}>();

const { t } = useI18n();
const { showToast } = useAppToast();
const roleStore = useRoleStore();
const expertStore = useExpertModelsStore();

const detailOpen = ref(false);
const resetBusy = ref(false);

async function refreshIfNeeded(): Promise<void> {
  const rid = (roleStore.currentRoleId ?? "").trim();
  if (!rid) return;
  await expertStore.refresh();
}

watch(
  () => [props.active, roleStore.currentRoleId] as const,
  ([active]) => {
    if (active) void refreshIfNeeded();
  },
  { immediate: true },
);

const recipeMode = computed(() =>
  resolveExpertRecipeUiMode(expertStore.graphSource, expertStore.effectiveGraph),
);

const hasCurrentRole = computed(() => Boolean((roleStore.currentRoleId ?? "").trim()));

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

const graphSourceLabelKey = computed(() => {
  switch (expertStore.graphSource) {
    case "session_override":
      return "settings.expertHub.graphSource.sessionOverride";
    case "role_default":
      return "settings.expertHub.graphSource.roleDefault";
    default:
      return "settings.expertHub.graphSource.packDefault";
  }
});

const nodeCount = computed(() => expertStore.effectiveGraph?.nodes?.length ?? 0);

const detailJson = computed(() =>
  formatExpertConfigDetailJson(expertStore.effectiveGraph, expertStore.effectivePromptStyle),
);

const canResetToPack = computed(
  () =>
    hasCurrentRole.value &&
    (expertStore.graphSource === "session_override" || expertStore.graphSource === "role_default"),
);

const workbenchDraftMode = computed<ExpertWorkbenchDraftMode>(() =>
  recipeMode.value === "role_default" ? "role_default" : "effective",
);

function openWorkbench(): void {
  emit("openExpertWorkbench", { draftMode: workbenchDraftMode.value });
}

async function onResetToPackDefault(): Promise<void> {
  if (!canResetToPack.value) return;
  const ok = window.confirm(String(t("settings.expertHub.confirmResetPackDefault")));
  if (!ok) return;
  resetBusy.value = true;
  try {
    const r = await expertStore.resetExpertGraphToPackDefaultFully();
    if (r.ok) {
      showToast("success", String(t("settings.expertHub.toastResetOk")));
    } else {
      showToast("warning", String(t("settings.expertHub.toastResetApplyWarn")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    resetBusy.value = false;
  }
}
</script>

<template>
  <div class="emh">
    <div v-if="!hasCurrentRole" class="emh-empty">
      <p class="emh-muted">{{ t("settings.expertHub.noRole") }}</p>
    </div>

    <template v-else>
      <p v-if="expertStore.loading" class="emh-muted">{{ t("expertRuntimeCard.loading") }}</p>
      <div v-else-if="expertStore.error" class="emh-err-row">
        <span class="emh-err">{{ expertStore.error }}</span>
        <button type="button" class="emh-retry" @click="refreshIfNeeded">
          {{ t("settings.modelSelector.retry") }}
        </button>
      </div>

      <template v-else>
        <div class="emh-head">
          <span
            class="emh-pill"
            :class="{
              'emh-pill--pure': recipeMode === 'pure',
              'emh-pill--role': recipeMode === 'role_default',
              'emh-pill--sess': recipeMode === 'session_override',
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
          <span class="emh-meta">{{ t(graphSourceLabelKey) }} · {{ t("settings.expertHub.nodeCount", { n: nodeCount }) }}</span>
        </div>

        <p class="emh-status-title">{{ statusTitle }}</p>
        <p class="emh-status-body">{{ statusBody }}</p>

        <div class="emh-actions">
          <button type="button" class="emh-btn emh-btn--ghost" @click="detailOpen = true">
            {{ t("expertRuntimeCard.btnDetail") }}
          </button>
          <button type="button" class="emh-btn emh-btn--primary" @click="openWorkbench">
            {{ t("settings.expertHub.openWorkbench") }}
          </button>
          <button
            type="button"
            class="emh-btn emh-btn--danger"
            :disabled="resetBusy || !canResetToPack"
            :title="canResetToPack ? '' : String(t('settings.expertHub.resetDisabledHint'))"
            @click="onResetToPackDefault"
          >
            {{ resetBusy ? t("expertRuntimeCard.resetting") : t("settings.expertHub.resetToPack") }}
          </button>
        </div>
      </template>
    </template>

    <Teleport to="body">
      <div
        v-if="detailOpen"
        class="emh-backdrop"
        role="dialog"
        aria-modal="true"
        @click="detailOpen = false"
      >
        <div class="emh-modal" @click.stop>
          <h4 class="emh-modal-title">{{ t("expertRuntimeCard.detailTitle") }}</h4>
          <pre class="emh-pre">{{ detailJson }}</pre>
          <div class="emh-modal-actions">
            <button type="button" class="emh-btn" @click="detailOpen = false">
              {{ t("common.close") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.emh {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 720px;
}
.emh-empty {
  padding: 16px;
  border-radius: 10px;
  border: 1px dashed var(--border-light);
  background: var(--bg-elevated, rgba(255, 255, 255, 0.03));
}
.emh-muted,
.emh-meta {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
}
.emh-meta {
  font-size: 12px;
}
.emh-err-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.emh-err {
  font-size: 13px;
  color: var(--text-accent, #b45309);
}
.emh-retry {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.emh-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.emh-pill {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 600;
  white-space: nowrap;
}
.emh-pill--pure {
  background: color-mix(in srgb, var(--text-secondary) 18%, transparent);
  color: var(--text-secondary);
}
.emh-pill--role {
  background: color-mix(in srgb, #3b82f6 22%, transparent);
  color: color-mix(in srgb, #1d4ed8 90%, var(--text-primary));
}
.emh-pill--sess {
  background: color-mix(in srgb, #a855f7 22%, transparent);
  color: color-mix(in srgb, #6b21a8 90%, var(--text-primary));
}
.emh-status-title {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  color: var(--text-primary);
}
.emh-status-body {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-line;
  color: var(--text-secondary);
}
.emh-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.emh-btn {
  padding: 8px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.emh-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.emh-btn--ghost {
  background: transparent;
}
.emh-btn--primary {
  border-color: color-mix(in srgb, var(--accent, #6366f1) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #6366f1) 16%, transparent);
}
.emh-btn--danger {
  border-color: color-mix(in srgb, #ef4444 45%, var(--border-light));
  color: #b91c1c;
}
.emh-backdrop {
  position: fixed;
  inset: 0;
  z-index: 12000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  padding: 24px;
}
.emh-modal {
  max-width: min(920px, 100%);
  max-height: min(86vh, 100%);
  overflow: auto;
  border-radius: 12px;
  padding: 16px 18px;
  background: var(--bg-primary, #111);
  border: 1px solid var(--border-light);
}
.emh-modal-title {
  margin: 0 0 10px;
  font-size: 14px;
}
.emh-pre {
  margin: 0;
  font-size: 11px;
  line-height: 1.35;
  white-space: pre-wrap;
  word-break: break-word;
}
.emh-modal-actions {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
