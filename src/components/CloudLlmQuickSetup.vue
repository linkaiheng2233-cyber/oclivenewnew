<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import {
  CLOUD_LLM_PRESET_DEFAULTS,
  CLOUD_LLM_PRESET_ORDER,
  type CloudLlmPresetId,
} from "../lib/cloudLlmPresets";
import { getHostCloudLlmPublic, setHostCloudLlm } from "../utils/tauri-api";

/** `pureChat`：隐藏复制脚本/env，适合纯聊浮层精简流程。 */
const props = withDefaults(defineProps<{ variant?: "default" | "pureChat" }>(), {
  variant: "default",
});

const emit = defineEmits<{ saved: [] }>();

const { t } = useI18n();
const { showToast } = useAppToast();

const presetId = ref<CloudLlmPresetId>("openai");
const baseUrl = ref(CLOUD_LLM_PRESET_DEFAULTS.openai.baseUrl);
const apiKey = ref("");
const model = ref(CLOUD_LLM_PRESET_DEFAULTS.openai.model);
const timeoutMsStr = ref("");
const saving = ref(false);
const hasSavedKey = ref(false);

function matchPresetFromBaseUrl(url: string): CloudLlmPresetId {
  const u = url.trim().replace(/\/+$/, "");
  for (const pid of CLOUD_LLM_PRESET_ORDER) {
    if (pid === "custom") continue;
    const d = CLOUD_LLM_PRESET_DEFAULTS[pid];
    if (u === d.baseUrl.trim().replace(/\/+$/, "")) return pid;
  }
  return "custom";
}

onMounted(async () => {
  try {
    const pub = await getHostCloudLlmPublic();
    hasSavedKey.value = pub.hasApiKey === true;
    if (pub.baseUrl?.trim()) {
      baseUrl.value = pub.baseUrl.trim();
      model.value = (pub.model ?? "").trim() || CLOUD_LLM_PRESET_DEFAULTS.openai.model;
      presetId.value = matchPresetFromBaseUrl(pub.baseUrl);
      if (pub.timeoutMs != null && pub.timeoutMs > 0) {
        timeoutMsStr.value = String(pub.timeoutMs);
      }
    }
  } catch {
    /* 忽略首次读取失败 */
  }
});

watch(presetId, (id) => {
  if (id === "custom") return;
  const d = CLOUD_LLM_PRESET_DEFAULTS[id];
  baseUrl.value = d.baseUrl;
  model.value = d.model;
});

const baseTrimmed = computed(() => baseUrl.value.trim().replace(/\/+$/, ""));
const modelTrimmed = computed(() => model.value.trim());

const isPureChat = computed(() => props.variant === "pureChat");

function psSingleQuoted(s: string): string {
  return `'${s.replace(/'/g, "''")}'`;
}

function buildPowerShellBlock(): string {
  const b = baseTrimmed.value;
  const k = apiKey.value.trim();
  const m = modelTrimmed.value;
  if (!b || !k) {
    throw new Error(String(t("settings.cloudLlmQuick.errNeedUrlKey")));
  }
  const lines = [
    `$env:OCLIVE_CLOUD_LLM_BASE_URL = ${psSingleQuoted(b)}`,
    `$env:OCLIVE_CLOUD_LLM_API_KEY = ${psSingleQuoted(k)}`,
  ];
  if (m) lines.push(`$env:OCLIVE_CLOUD_LLM_MODEL = ${psSingleQuoted(m)}`);
  lines.push(`Write-Host ${psSingleQuoted(String(t("settings.cloudLlmQuick.psDoneHint")))}`);
  return lines.join("\n");
}

function buildDotEnvSnippet(): string {
  const b = baseTrimmed.value;
  const k = apiKey.value.trim();
  const m = modelTrimmed.value;
  if (!b || !k) {
    throw new Error(String(t("settings.cloudLlmQuick.errNeedUrlKey")));
  }
  const esc = (v: string) => v.replace(/\\/g, "\\\\").replace(/\n/g, "\\n").replace(/"/g, '\\"');
  const lines = [`OCLIVE_CLOUD_LLM_BASE_URL="${esc(b)}"`, `OCLIVE_CLOUD_LLM_API_KEY="${esc(k)}"`];
  if (m) lines.push(`OCLIVE_CLOUD_LLM_MODEL="${esc(m)}"`);
  return `${lines.join("\n")}\n`;
}

async function copyText(label: "ps" | "env", text: string): Promise<void> {
  if (!navigator.clipboard?.writeText) {
    showToast("error", String(t("settings.cloudLlmQuick.errClipboard")));
    return;
  }
  await navigator.clipboard.writeText(text);
  showToast(
    "success",
    String(t(label === "ps" ? "settings.cloudLlmQuick.toastCopiedPs" : "settings.cloudLlmQuick.toastCopiedEnv")),
  );
}

async function onCopyPowerShell(): Promise<void> {
  try {
    await copyText("ps", buildPowerShellBlock());
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onCopyDotEnv(): Promise<void> {
  try {
    await copyText("env", buildDotEnvSnippet());
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

function parseTimeoutMs(): number | undefined {
  const s = timeoutMsStr.value.trim();
  if (!s) return undefined;
  const n = Number(s);
  if (!Number.isFinite(n) || n < 1000) {
    throw new Error(String(t("settings.cloudLlmQuick.errTimeout")));
  }
  return Math.min(600_000, Math.floor(n));
}

async function onSaveToHost(): Promise<void> {
  const b = baseTrimmed.value;
  if (!b) {
    showToast("error", String(t("settings.cloudLlmQuick.errNeedUrl")));
    return;
  }
  saving.value = true;
  try {
    const timeoutMs = parseTimeoutMs();
    await setHostCloudLlm({
      baseUrl: b,
      apiKey: apiKey.value.trim(),
      model: modelTrimmed.value ? modelTrimmed.value : null,
      timeoutMs: timeoutMs ?? null,
    });
    apiKey.value = "";
    hasSavedKey.value = true;
    showToast("success", String(t("settings.cloudLlmQuick.toastSavedHost")));
    emit("saved");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

async function onClearHost(): Promise<void> {
  saving.value = true;
  try {
    await setHostCloudLlm({
      baseUrl: "",
      apiKey: "",
      model: null,
      timeoutMs: null,
    });
    presetId.value = "openai";
    baseUrl.value = CLOUD_LLM_PRESET_DEFAULTS.openai.baseUrl;
    model.value = CLOUD_LLM_PRESET_DEFAULTS.openai.model;
    apiKey.value = "";
    timeoutMsStr.value = "";
    hasSavedKey.value = false;
    showToast("success", String(t("settings.cloudLlmQuick.toastClearedHost")));
    emit("saved");
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="clqs" :class="{ 'clqs--pure-chat': isPureChat }">
    <div class="clqs-h">{{ t("settings.cloudLlmQuick.title") }}</div>
    <p class="clqs-muted">{{ isPureChat ? t("settings.cloudLlmQuick.pureChatLead") : t("settings.cloudLlmQuick.lead") }}</p>
    <p v-if="!isPureChat" class="clqs-muted clqs-priority">{{ t("settings.cloudLlmQuick.priorityHint") }}</p>

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.preset") }}</label>
    <select v-model="presetId" class="clqs-select">
      <option v-for="pid in CLOUD_LLM_PRESET_ORDER" :key="pid" :value="pid">
        {{ t(`settings.cloudLlmQuick.presets.${pid}`) }}
      </option>
    </select>

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.baseUrl") }}</label>
    <input v-model="baseUrl" type="url" autocomplete="off" class="clqs-input" spellcheck="false" />

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.model") }}</label>
    <input v-model="model" type="text" autocomplete="off" class="clqs-input" spellcheck="false" />

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.timeoutMs") }}</label>
    <input
      v-model="timeoutMsStr"
      type="text"
      inputmode="numeric"
      autocomplete="off"
      class="clqs-input"
      :placeholder="String(t('settings.cloudLlmQuick.timeoutPlaceholder'))"
    />

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.apiKey") }}</label>
    <input
      v-model="apiKey"
      type="password"
      autocomplete="off"
      class="clqs-input"
      :placeholder="
        hasSavedKey ? String(t('settings.cloudLlmQuick.apiKeyPlaceholderKeep')) : ''
      "
    />
    <p class="clqs-muted clqs-key-hint">{{ t("settings.cloudLlmQuick.apiKeyHint") }}</p>

    <p class="clqs-muted clqs-warn">{{ t("settings.cloudLlmQuick.warnPersist") }}</p>

    <div class="clqs-actions">
      <button
        type="button"
        class="clqs-btn clqs-btn--primary"
        :disabled="saving"
        @click="void onSaveToHost()"
      >
        {{ t("settings.cloudLlmQuick.saveHost") }}
      </button>
      <button type="button" class="clqs-btn" :disabled="saving" @click="void onClearHost()">
        {{ t("settings.cloudLlmQuick.clearHost") }}
      </button>
      <template v-if="!isPureChat">
        <button type="button" class="clqs-btn" :disabled="saving" @click="onCopyPowerShell">
          {{ t("settings.cloudLlmQuick.copyPs") }}
        </button>
        <button type="button" class="clqs-btn" :disabled="saving" @click="onCopyDotEnv">
          {{ t("settings.cloudLlmQuick.copyEnv") }}
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.clqs {
  margin-top: 10px;
  padding: 10px 12px 12px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--accent, #3b82f6) 28%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 6%, var(--bg-elevated));
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.clqs-h {
  font-size: 13px;
  font-weight: 650;
  color: var(--text-primary);
}
.clqs-muted {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.clqs-priority {
  margin-top: 2px;
}
.clqs-key-hint {
  margin: 0;
  font-size: 11px;
}
.clqs-warn {
  margin-top: 4px;
}
.clqs-label {
  margin-top: 4px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}
.clqs-select,
.clqs-input {
  width: 100%;
  padding: 6px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  box-sizing: border-box;
}
.clqs-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}
.clqs-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.clqs-btn--primary {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 18%, var(--bg-primary));
}
.clqs-btn:hover:not(:disabled) {
  filter: brightness(1.04);
}
.clqs-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.clqs--pure-chat .clqs-actions {
  margin-top: 6px;
}
</style>
