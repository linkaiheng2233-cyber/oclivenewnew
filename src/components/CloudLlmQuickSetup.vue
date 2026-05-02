<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import {
  CLOUD_LLM_PRESET_DEFAULTS,
  CLOUD_LLM_PRESET_ORDER,
  type CloudLlmPresetId,
} from "../lib/cloudLlmPresets";

const { t } = useI18n();
const { showToast } = useAppToast();

const presetId = ref<CloudLlmPresetId>("openai");
const baseUrl = ref(CLOUD_LLM_PRESET_DEFAULTS.openai.baseUrl);
const apiKey = ref("");
const model = ref(CLOUD_LLM_PRESET_DEFAULTS.openai.model);

watch(presetId, (id) => {
  if (id === "custom") return;
  const d = CLOUD_LLM_PRESET_DEFAULTS[id];
  baseUrl.value = d.baseUrl;
  model.value = d.model;
});

const baseTrimmed = computed(() => baseUrl.value.trim().replace(/\/+$/, ""));
const modelTrimmed = computed(() => model.value.trim());

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
</script>

<template>
  <div class="clqs">
    <div class="clqs-h">{{ t("settings.cloudLlmQuick.title") }}</div>
    <p class="clqs-muted">{{ t("settings.cloudLlmQuick.lead") }}</p>

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.preset") }}</label>
    <select v-model="presetId" class="clqs-select">
      <option v-for="pid in CLOUD_LLM_PRESET_ORDER" :key="pid" :value="pid">
        {{ t(`settings.cloudLlmQuick.presets.${pid}`) }}
      </option>
    </select>

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.baseUrl") }}</label>
    <input v-model="baseUrl" type="url" autocomplete="off" class="clqs-input" spellcheck="false" />

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.apiKey") }}</label>
    <input v-model="apiKey" type="password" autocomplete="off" class="clqs-input" />

    <label class="clqs-label">{{ t("settings.cloudLlmQuick.model") }}</label>
    <input v-model="model" type="text" autocomplete="off" class="clqs-input" spellcheck="false" />

    <p class="clqs-muted clqs-warn">{{ t("settings.cloudLlmQuick.warnNoPersist") }}</p>

    <div class="clqs-actions">
      <button type="button" class="clqs-btn clqs-btn--primary" @click="onCopyPowerShell">
        {{ t("settings.cloudLlmQuick.copyPs") }}
      </button>
      <button type="button" class="clqs-btn" @click="onCopyDotEnv">
        {{ t("settings.cloudLlmQuick.copyEnv") }}
      </button>
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
.clqs-btn:hover {
  filter: brightness(1.04);
}
</style>
