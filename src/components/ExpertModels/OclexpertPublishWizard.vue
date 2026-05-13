<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { open, save } from "@tauri-apps/api/dialog";
import { writeTextFile } from "@tauri-apps/api/fs";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useAppToast } from "../../composables/useAppToast";
import { buildOclexpertPayload, validateExpertGraphNodes } from "../../lib/oclexpert";
import { githubPublishOclexpertRecipe } from "../../utils/tauri-api";
import type { ExpertGraph, PromptStyleOverride } from "../../utils/tauri-api";

const LS_TOKEN_KEY = "oclive_expert_publish_github_token";

const DEFAULT_ISSUE_REPO = "linkaiheng2233-cyber/awesome-oclive-plugins";
const CONTRIBUTING_GUIDE_URL = `https://github.com/${DEFAULT_ISSUE_REPO}/blob/main/CONTRIBUTING.md`;

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    embedded?: boolean;
    graph: ExpertGraph;
    promptStyle: PromptStyleOverride | null;
    initialName: string;
    initialDescription: string;
    initialAuthor: string;
    lastExportPath?: string;
  }>(),
  { embedded: false, lastExportPath: "" },
);

const emit = defineEmits<{
  (e: "update:modelValue", v: boolean): void;
  (e: "sync-drafts", v: { name: string; description: string; author: string }): void;
}>();

const { t } = useI18n();
const { showToast } = useAppToast();

const step = ref<1 | 2 | 3 | 4>(1);
const wName = ref("");
const wDesc = ref("");
const wAuthor = ref("");
const busy = ref(false);

const issueRepo = ref(DEFAULT_ISSUE_REPO);
const githubToken = ref("");
const rememberGithubToken = ref(false);

type DoneKind = "issue" | "manual" | "api" | null;
const doneKind = ref<DoneKind>(null);
/** Primary URL or path to copy (issue URL, draft issue URL, or file path). */
const primaryLink = ref("");
const secondaryLink = ref("");
const secondaryKind = ref<"gist" | "">("");

function summarizeNodes(graph: ExpertGraph): string {
  const nodes = graph.nodes ?? [];
  if (nodes.length === 0) return String(t("expertModels.oclexpert.previewGraphEmpty"));
  const counts = new Map<string, number>();
  for (const n of nodes) {
    const ty = String((n as { type?: string }).type ?? "?").trim() || "?";
    counts.set(ty, (counts.get(ty) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([ty, c]) => `${ty}×${c}`)
    .join(" · ");
}

function privacySummary(graph: ExpertGraph): string {
  const hasTriggers = (graph.nodes ?? []).some((n) => (n as { type?: string }).type === "event_trigger");
  const hasCloud = (graph.nodes ?? []).some((n) => (n as { type?: string }).type === "cloud_model");
  const parts: string[] = [String(t("expertModels.oclexpert.previewPrivacyBaseline"))];
  if (hasTriggers) parts.push(String(t("expertModels.oclexpert.previewPrivacyTriggers")));
  if (hasCloud) parts.push(String(t("expertModels.oclexpert.previewPrivacyCloud")));
  return parts.join(" ");
}

const payloadObject = computed(() =>
  buildOclexpertPayload(props.graph, props.promptStyle, {
    name: wName.value.trim(),
    description: wDesc.value.trim(),
    author: wAuthor.value.trim(),
  }),
);

const jsonText = computed(() => JSON.stringify(payloadObject.value, null, 2));

const previewLimit = 8000;
const jsonPreview = computed(() => {
  const s = jsonText.value;
  if (s.length <= previewLimit) return s;
  return `${s.slice(0, previewLimit)}\n…`;
});

const safeFileBase = computed(() => {
  const raw = wName.value.trim() || "recipe";
  const s = raw
    .split("")
    .map((c) => (/[a-zA-Z0-9\u4e00-\u9fff\-_]/.test(c) ? c : "_"))
    .join("")
    .replace(/_+/g, "_")
    .slice(0, 80);
  return s || "recipe";
});

const targetFilename = computed(() => `${safeFileBase.value}.oclexpert`);

function buildIssueBodyMarkdown(): string {
  const lines = [
    String(t("expertModels.oclexpert.publishWizard.issueBodyHead")),
    "",
    `**${String(t("expertModels.oclexpert.previewName"))}** ${wName.value.trim() || "—"}`,
    `**${String(t("expertModels.oclexpert.previewAuthor"))}** ${wAuthor.value.trim() || "—"}`,
    `**${String(t("expertModels.oclexpert.previewDescription"))}** ${wDesc.value.trim() || "—"}`,
    "",
    `**${String(t("expertModels.oclexpert.previewGraphSummary"))}** ${summarizeNodes(props.graph)}`,
    "",
    `**${String(t("expertModels.oclexpert.previewPrivacy"))}** ${privacySummary(props.graph)}`,
    "",
    String(t("expertModels.oclexpert.publishWizard.issueBodyClipboard")),
  ];
  if (props.lastExportPath?.trim()) {
    lines.push("", `${String(t("expertModels.oclexpert.publishWizard.lastExportHint"))} ${props.lastExportPath.trim()}`);
  }
  return lines.join("\n");
}

function buildNewIssueUrl(title: string, body: string): string {
  const repo = DEFAULT_ISSUE_REPO;
  const u = new URL(`https://github.com/${repo}/issues/new`);
  u.searchParams.set("labels", "oclexpert");
  u.searchParams.set("title", title);
  u.searchParams.set("body", body);
  return u.toString();
}

function syncEmitAndClose(): void {
  emit("sync-drafts", {
    name: wName.value.trim(),
    description: wDesc.value.trim(),
    author: wAuthor.value.trim(),
  });
  emit("update:modelValue", false);
}

function close(): void {
  syncEmitAndClose();
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    step.value = 1;
    doneKind.value = null;
    primaryLink.value = "";
    secondaryLink.value = "";
    secondaryKind.value = "";
    wName.value = props.initialName?.trim() ?? "";
    wDesc.value = props.initialDescription?.trim() ?? "";
    wAuthor.value = props.initialAuthor?.trim() ?? "";
    issueRepo.value = DEFAULT_ISSUE_REPO;
    rememberGithubToken.value = false;
    try {
      const saved = localStorage.getItem(LS_TOKEN_KEY);
      githubToken.value = saved?.trim() ?? "";
      if (githubToken.value) rememberGithubToken.value = true;
    } catch {
      githubToken.value = "";
    }
  },
);

function validateMeta(): boolean {
  if (!wName.value.trim() || !wDesc.value.trim() || !wAuthor.value.trim()) {
    showToast("warning", String(t("expertModels.oclexpert.exportRequiredFields")));
    return false;
  }
  return true;
}

function goNextFrom1(): void {
  if (!validateMeta()) return;
  step.value = 2;
}

function goNextFrom2(): void {
  try {
    validateExpertGraphNodes(props.graph);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
    return;
  }
  step.value = 3;
}

function goBack(): void {
  if (step.value === 2) step.value = 1;
  else if (step.value === 3) step.value = 2;
}

async function copyText(text: string): Promise<boolean> {
  try {
    if (!navigator.clipboard?.writeText) throw new Error("clipboard");
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    showToast("error", String(t("expertModels.oclexpert.publishWizard.clipboardFailed")));
    return false;
  }
}

async function runIssueClipboardFlow(): Promise<void> {
  if (!validateMeta()) return;
  busy.value = true;
  try {
    try {
      validateExpertGraphNodes(props.graph);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
      return;
    }
    const ok = await copyText(jsonText.value);
    if (!ok) return;
    showToast("success", String(t("expertModels.oclexpert.publishWizard.clipboardOkToast")));
    const title = `[oclexpert] ${wName.value.trim()}`;
    const body = buildIssueBodyMarkdown();
    const draftUrl = buildNewIssueUrl(title, body);
    await openExternal(draftUrl);
    doneKind.value = "issue";
    primaryLink.value = draftUrl;
    secondaryLink.value = "";
    secondaryKind.value = "";
    step.value = 4;
  } finally {
    busy.value = false;
  }
}

async function runManualExportFlow(): Promise<void> {
  if (!validateMeta()) return;
  busy.value = true;
  try {
    try {
      validateExpertGraphNodes(props.graph);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
      return;
    }
    const path = await save({
      defaultPath: targetFilename.value,
      filters: [
        { name: String(t("expertModels.oclexpert.filterName")), extensions: ["oclexpert"] },
        { name: "JSON", extensions: ["json"] },
      ],
    });
    if (!path) return;
    await writeTextFile(path, jsonText.value);
    await openExternal(CONTRIBUTING_GUIDE_URL);
    doneKind.value = "manual";
    primaryLink.value = path;
    secondaryLink.value = CONTRIBUTING_GUIDE_URL;
    secondaryKind.value = "";
    step.value = 4;
    showToast("success", String(t("expertModels.oclexpert.toastExported")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busy.value = false;
  }
}

async function runGithubApiFlow(): Promise<void> {
  if (!validateMeta()) return;
  const tok = githubToken.value.trim();
  if (!tok) {
    showToast("warning", String(t("expertModels.oclexpert.publishWizard.tokenMissing")));
    return;
  }
  const repo = issueRepo.value.trim();
  if (!repo.includes("/")) {
    showToast("warning", String(t("expertModels.oclexpert.publishWizard.issueRepoInvalid")));
    return;
  }
  busy.value = true;
  try {
    try {
      validateExpertGraphNodes(props.graph);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
      return;
    }
    const intro = buildIssueBodyMarkdown();
    const r = await githubPublishOclexpertRecipe({
      token: tok,
      issueRepo: repo,
      title: `[oclexpert] ${wName.value.trim()}`,
      issueBodyIntro: intro,
      oclexpertFilename: targetFilename.value,
      oclexpertContent: jsonText.value,
      gistDescription: `${wName.value.trim()} — OCLive oclexpert`,
    });
    try {
      if (rememberGithubToken.value) localStorage.setItem(LS_TOKEN_KEY, tok);
      else localStorage.removeItem(LS_TOKEN_KEY);
    } catch {
      /* ignore */
    }
    doneKind.value = "api";
    primaryLink.value = r.issueUrl;
    secondaryLink.value = r.gistUrl;
    secondaryKind.value = "gist";
    step.value = 4;
    showToast("success", String(t("expertModels.oclexpert.publishWizard.apiSuccessToast")));
    await openExternal(r.issueUrl);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busy.value = false;
  }
}

async function copyPrimary(): Promise<void> {
  if (!primaryLink.value) return;
  const ok = await copyText(primaryLink.value);
  if (ok) showToast("success", String(t("expertModels.oclexpert.publishWizard.copiedToast")));
}

async function copySecondary(): Promise<void> {
  if (!secondaryLink.value) return;
  const ok = await copyText(secondaryLink.value);
  if (ok) showToast("success", String(t("expertModels.oclexpert.publishWizard.copiedToast")));
}

const stepTitle = computed(() => {
  if (step.value === 1) return String(t("expertModels.oclexpert.publishWizard.step1Title"));
  if (step.value === 2) return String(t("expertModels.oclexpert.publishWizard.step2Title"));
  if (step.value === 3) return String(t("expertModels.oclexpert.publishWizard.step3Title"));
  return String(t("expertModels.oclexpert.publishWizard.stepDoneTitle"));
});
</script>

<template>
  <Teleport to="body" :disabled="embedded">
    <div
      v-if="modelValue"
      class="em-oclexpert-backdrop"
      :class="{ 'em-oclexpert-backdrop--inplace': embedded }"
      role="dialog"
      aria-modal="true"
      @click.self="close"
    >
      <div class="em-oclexpert-modal em-publish-wizard" @click.stop>
        <div class="em-publish-wizard-head">
          <div class="em-card-h">{{ t("expertModels.oclexpert.publishWizard.title") }}</div>
          <button type="button" class="em-publish-iconbtn" :disabled="busy" @click="close">×</button>
        </div>
        <p class="em-muted em-publish-stepline">{{ stepTitle }}</p>

        <div v-if="step === 1" class="em-publish-body">
          <label class="em-publish-label">{{ t("expertModels.workflows.nameLabel") }}</label>
          <input v-model="wName" class="em-publish-input" type="text" :disabled="busy" />
          <label class="em-publish-label">{{ t("expertModels.oclexpert.metaDescriptionLabel") }}</label>
          <textarea v-model="wDesc" class="em-publish-text" rows="3" :disabled="busy" />
          <label class="em-publish-label">{{ t("expertModels.oclexpert.metaAuthorLabel") }}</label>
          <input v-model="wAuthor" class="em-publish-input" type="text" :disabled="busy" />
          <div class="em-publish-block">
            <div class="em-muted">{{ t("expertModels.oclexpert.previewGraphSummary") }}</div>
            <div class="em-publish-mono">{{ summarizeNodes(graph) }}</div>
          </div>
        </div>

        <div v-else-if="step === 2" class="em-publish-body">
          <div class="em-publish-block">
            <div class="em-muted">{{ t("expertModels.oclexpert.publishWizard.previewFilename") }}</div>
            <div class="em-publish-mono">{{ targetFilename }}</div>
          </div>
          <div class="em-publish-block">
            <div class="em-muted">
              {{ t("expertModels.oclexpert.publishWizard.previewJson") }}
              <span v-if="jsonText.length > previewLimit" class="em-muted2">
                {{ t("expertModels.oclexpert.publishWizard.previewTruncate", { n: previewLimit }) }}
              </span>
            </div>
            <pre class="em-publish-pre">{{ jsonPreview }}</pre>
          </div>
          <div class="em-publish-block">
            <div class="em-muted">{{ t("expertModels.oclexpert.previewPrivacy") }}</div>
            <p class="em-publish-privacy">{{ privacySummary(graph) }}</p>
          </div>
        </div>

        <div v-else-if="step === 3" class="em-publish-body">
          <div class="em-publish-method">
            <div class="em-publish-method-h">{{ t("expertModels.oclexpert.publishWizard.methodIssueTitle") }}</div>
            <p class="em-muted">{{ t("expertModels.oclexpert.publishWizard.methodIssueDesc") }}</p>
            <button type="button" class="em-btn" :disabled="busy" @click="runIssueClipboardFlow">
              {{ t("expertModels.oclexpert.publishWizard.methodIssueGo") }}
            </button>
          </div>
          <div class="em-publish-method">
            <div class="em-publish-method-h">{{ t("expertModels.oclexpert.publishWizard.methodManualTitle") }}</div>
            <p class="em-muted">{{ t("expertModels.oclexpert.publishWizard.methodManualDesc") }}</p>
            <button type="button" class="em-btn secondary" :disabled="busy" @click="runManualExportFlow">
              {{ t("expertModels.oclexpert.publishWizard.methodManualGo") }}
            </button>
          </div>
          <div class="em-publish-method em-publish-method--advanced">
            <div class="em-publish-method-h">{{ t("expertModels.oclexpert.publishWizard.methodApiTitle") }}</div>
            <p class="em-muted">{{ t("expertModels.oclexpert.publishWizard.methodApiDesc") }}</p>
            <label class="em-publish-label">{{ t("expertModels.oclexpert.publishWizard.tokenLabel") }}</label>
            <input
              v-model="githubToken"
              class="em-publish-input"
              type="password"
              autocomplete="off"
              :placeholder="t('expertModels.oclexpert.publishWizard.tokenPlaceholder')"
              :disabled="busy"
            />
            <label class="em-publish-label">{{ t("expertModels.oclexpert.publishWizard.issueRepoLabel") }}</label>
            <input v-model="issueRepo" class="em-publish-input" type="text" :disabled="busy" />
            <label class="em-publish-check">
              <input v-model="rememberGithubToken" type="checkbox" :disabled="busy" />
              {{ t("expertModels.oclexpert.publishWizard.rememberToken") }}
            </label>
            <button type="button" class="em-btn secondary" :disabled="busy" @click="runGithubApiFlow">
              {{ t("expertModels.oclexpert.publishWizard.methodApiGo") }}
            </button>
          </div>
        </div>

        <div v-else class="em-publish-body">
          <p v-if="doneKind === 'issue'" class="em-muted">{{ t("expertModels.oclexpert.publishWizard.issueOpenedHint") }}</p>
          <p v-else-if="doneKind === 'manual'" class="em-muted">{{ t("expertModels.oclexpert.publishWizard.manualDoneHint") }}</p>
          <p v-else-if="doneKind === 'api'" class="em-muted">{{ t("expertModels.oclexpert.publishWizard.apiDoneHint") }}</p>
          <div class="em-publish-block">
            <div class="em-muted">
              {{
                doneKind === "manual"
                  ? t("expertModels.oclexpert.publishWizard.donePrimaryPath")
                  : doneKind === "issue"
                    ? t("expertModels.oclexpert.publishWizard.doneDraftIssueLine")
                    : t("expertModels.oclexpert.publishWizard.doneIssueLine")
              }}
            </div>
            <div class="em-publish-mono em-publish-break">{{ primaryLink }}</div>
            <button type="button" class="em-btn secondary" :disabled="!primaryLink || busy" @click="copyPrimary">
              {{ t("expertModels.oclexpert.publishWizard.copyPrimary") }}
            </button>
          </div>
          <div v-if="secondaryLink" class="em-publish-block">
            <div class="em-muted">
              {{
                secondaryKind === "gist"
                  ? t("expertModels.oclexpert.publishWizard.copyGistHint")
                  : t("expertModels.oclexpert.publishWizard.doneContributing")
              }}
            </div>
            <div class="em-publish-mono em-publish-break">{{ secondaryLink }}</div>
            <button type="button" class="em-btn secondary" :disabled="busy" @click="copySecondary">
              {{
                secondaryKind === "gist"
                  ? t("expertModels.oclexpert.publishWizard.copyGistLink")
                  : t("expertModels.oclexpert.publishWizard.copySecondary")
              }}
            </button>
          </div>
        </div>

        <div class="em-oclexpert-actions em-publish-actions">
          <template v-if="step < 4">
            <button type="button" class="em-btn secondary" :disabled="busy" @click="close">
              {{ t("expertModels.oclexpert.publishWizard.cancel") }}
            </button>
            <button v-if="step > 1" type="button" class="em-btn secondary" :disabled="busy" @click="goBack">
              {{ t("expertModels.oclexpert.publishWizard.back") }}
            </button>
            <button v-if="step === 1" type="button" class="em-btn" :disabled="busy" @click="goNextFrom1">
              {{ t("expertModels.oclexpert.publishWizard.next") }}
            </button>
            <button v-if="step === 2" type="button" class="em-btn" :disabled="busy" @click="goNextFrom2">
              {{ t("expertModels.oclexpert.publishWizard.next") }}
            </button>
          </template>
          <template v-else>
            <button type="button" class="em-btn" :disabled="busy" @click="close">
              {{ t("expertModels.oclexpert.publishWizard.close") }}
            </button>
          </template>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.em-publish-wizard {
  width: min(560px, 100%);
  max-width: min(560px, 96vw);
  max-height: min(88vh, 900px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.em-publish-wizard-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.em-publish-iconbtn {
  flex: 0 0 auto;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  padding: 0 4px;
}
.em-publish-iconbtn:hover {
  color: var(--text-primary);
}
.em-publish-stepline {
  margin: 4px 0 12px;
  font-size: 13px;
}
.em-publish-body {
  flex: 1 1 auto;
  overflow-y: auto;
  padding-right: 4px;
  margin-bottom: 8px;
}
.em-publish-label {
  display: block;
  margin: 10px 0 4px;
  font-size: 13px;
  color: var(--text-secondary);
}
.em-publish-input,
.em-publish-text {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font: inherit;
}
.em-publish-text {
  resize: vertical;
  min-height: 72px;
}
.em-publish-block {
  margin-top: 14px;
}
.em-publish-mono {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  margin-top: 6px;
  word-break: break-word;
}
.em-publish-break {
  white-space: pre-wrap;
}
.em-publish-pre {
  margin: 8px 0 0;
  padding: 10px;
  max-height: 200px;
  overflow: auto;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-size: 11px;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
}
.em-publish-privacy {
  margin: 6px 0 0;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
}
.em-muted2 {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 6px;
}
.em-publish-method {
  padding: 12px;
  margin-bottom: 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.em-publish-method--advanced {
  border-style: dashed;
}
.em-publish-method-h {
  font-weight: 600;
  margin-bottom: 6px;
  font-size: 14px;
}
.em-publish-method .em-btn {
  margin-top: 10px;
}
.em-publish-check {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 0;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
}
.em-publish-actions {
  flex-shrink: 0;
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid var(--border-light);
}
.em-oclexpert-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: color-mix(in srgb, #000 48%, transparent);
}
.em-oclexpert-backdrop--inplace {
  position: absolute;
  inset: 0;
  border-radius: inherit;
}
.em-oclexpert-modal {
  width: min(440px, 100%);
  padding: 16px 18px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.em-oclexpert-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}
</style>
