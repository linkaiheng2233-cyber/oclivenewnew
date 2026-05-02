<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import { hostEventBus } from "../lib/hostEventBus";
import { CLOUD_LLM_PRESET_DEFAULTS, CLOUD_LLM_PRESET_ORDER } from "../lib/cloudLlmPresets";
import {
  getHostChatModel,
  getHostCloudLlmPublic,
  ollamaModelsHealth,
  ollamaModelsListNames,
  setHostChatModel,
} from "../utils/tauri-api";

const CUSTOM_SENTINEL = "__oclive_custom_model__";

const props = defineProps<{ loading: boolean }>();

const emit = defineEmits<{
  send: [payload: { content: string }];
  openSettings: [];
}>();

const { t } = useI18n();
const { showToast } = useAppToast();

const text = ref("");
const textAreaEl = ref<HTMLTextAreaElement | null>(null);
const customInputEl = ref<HTMLInputElement | null>(null);

const modelId = ref("");
const lastSaved = ref("");
const ollamaNames = ref<string[]>([]);
const cloudPub = ref<Awaited<ReturnType<typeof getHostCloudLlmPublic>> | null>(null);

const useCustomModel = ref(false);
const selectModel = ref("");

let saveTimer: ReturnType<typeof setTimeout> | null = null;

const cloudSelectOptions = computed(() => {
  const s = new Set<string>();
  for (const pid of CLOUD_LLM_PRESET_ORDER) {
    if (pid === "custom") continue;
    s.add(CLOUD_LLM_PRESET_DEFAULTS[pid].model);
  }
  const saved = cloudPub.value?.model?.trim();
  if (saved) s.add(saved);
  const local = new Set(ollamaNames.value);
  return [...s].filter((m) => !local.has(m)).sort((a, b) => a.localeCompare(b));
});

function syncSelectFromModel(): void {
  const m = modelId.value.trim();
  if (!m) {
    selectModel.value = "";
    useCustomModel.value = false;
    return;
  }
  if (ollamaNames.value.includes(m)) {
    useCustomModel.value = false;
    selectModel.value = m;
    return;
  }
  if (cloudSelectOptions.value.includes(m)) {
    useCustomModel.value = false;
    selectModel.value = m;
    return;
  }
  useCustomModel.value = true;
  selectModel.value = CUSTOM_SENTINEL;
}

function schedulePersistCustom(): void {
  if (saveTimer != null) {
    window.clearTimeout(saveTimer);
    saveTimer = null;
  }
  saveTimer = window.setTimeout(() => {
    saveTimer = null;
    void persistModel();
  }, 400);
}

async function persistModel(): Promise<void> {
  const m = modelId.value.trim();
  if (!m) {
    showToast("error", String(t("chatComposer.errEmpty")));
    modelId.value = lastSaved.value;
    syncSelectFromModel();
    return;
  }
  if (m === lastSaved.value) return;
  try {
    await setHostChatModel(m);
    lastSaved.value = m;
    syncSelectFromModel();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
    modelId.value = lastSaved.value;
    syncSelectFromModel();
  }
}

async function loadOllama(): Promise<void> {
  try {
    const ok = await ollamaModelsHealth();
    if (!ok) {
      ollamaNames.value = [];
      return;
    }
    ollamaNames.value = await ollamaModelsListNames();
  } catch {
    ollamaNames.value = [];
  }
}

async function loadCloudPublic(): Promise<void> {
  try {
    cloudPub.value = await getHostCloudLlmPublic();
  } catch {
    cloudPub.value = null;
  }
}

async function bootstrap(): Promise<void> {
  try {
    const cur = await getHostChatModel();
    modelId.value = cur.trim();
    lastSaved.value = modelId.value;
  } catch {
    modelId.value = "";
  }
  await Promise.all([loadOllama(), loadCloudPublic()]);
  syncSelectFromModel();
}

function onWindowFocus(): void {
  void Promise.all([loadOllama(), loadCloudPublic()]).then(() => {
    syncSelectFromModel();
  });
}

onMounted(() => {
  void bootstrap();
  window.addEventListener("focus", onWindowFocus);
  hostEventBus.on("chat:set_input_draft", onSetDraftInput);
});

onBeforeUnmount(() => {
  window.removeEventListener("focus", onWindowFocus);
  hostEventBus.off("chat:set_input_draft", onSetDraftInput);
});

watch([ollamaNames, cloudPub], () => {
  syncSelectFromModel();
});

function onSetDraftInput(payload: unknown): void {
  const raw = (payload as { text?: string } | null)?.text;
  const next = typeof raw === "string" ? raw.trim() : "";
  if (!next) return;
  text.value = next;
  void nextTick(() => {
    textAreaEl.value?.focus();
    textAreaEl.value?.setSelectionRange(next.length, next.length);
  });
}

function onSelectModel(e: Event): void {
  const el = e.target as HTMLSelectElement;
  const v = el.value;
  if (v === CUSTOM_SENTINEL) {
    useCustomModel.value = true;
    selectModel.value = CUSTOM_SENTINEL;
    void nextTick(() => {
      customInputEl.value?.focus();
      customInputEl.value?.select();
    });
    return;
  }
  useCustomModel.value = false;
  selectModel.value = v;
  modelId.value = v;
  void persistModel();
}

function onCustomModelInput(): void {
  schedulePersistCustom();
}

function onCustomModelBlur(): void {
  void persistModel();
}

function submit(): void {
  const value = text.value.trim();
  if (!value || props.loading) return;
  emit("send", { content: value });
  text.value = "";
}

function onKeydown(e: KeyboardEvent): void {
  if (e.key !== "Enter") return;
  if (e.shiftKey) return;
  e.preventDefault();
  submit();
}

function onOpenSettings(): void {
  emit("openSettings");
}
</script>

<template>
  <section class="composer" role="region" :aria-label="String(t('chatComposer.aria'))">
    <div class="composer-toolbar">
      <label class="composer-model-label" for="oclive-composer-model-select">{{ t("chatComposer.modelLabel") }}</label>
      <div class="composer-toolbar-mid">
        <select
          id="oclive-composer-model-select"
          class="composer-select"
          :value="selectModel"
          :disabled="loading"
          @change="onSelectModel"
        >
          <optgroup :label="String(t('chatComposer.localGroup'))">
            <option v-if="!ollamaNames.length" disabled value="__none__">{{ t("chatComposer.offlineLocal") }}</option>
            <option v-for="n in ollamaNames" :key="'loc-' + n" :value="n">{{ n }}</option>
          </optgroup>
          <optgroup :label="String(t('chatComposer.cloudGroup'))">
            <option v-for="n in cloudSelectOptions" :key="'cld-' + n" :value="n">{{ n }}</option>
          </optgroup>
          <optgroup :label="String(t('chatComposer.customGroup'))">
            <option :value="CUSTOM_SENTINEL">{{ t("chatComposer.customOption") }}</option>
          </optgroup>
        </select>
        <div v-if="useCustomModel" class="composer-custom-wrap">
          <input
            ref="customInputEl"
            v-model="modelId"
            type="text"
            class="composer-custom-input"
            spellcheck="false"
            autocomplete="off"
            :placeholder="String(t('chatComposer.customPlaceholder'))"
            :disabled="loading"
            @input="onCustomModelInput"
            @blur="onCustomModelBlur"
          />
        </div>
      </div>
      <button
        type="button"
        class="composer-gear"
        :title="String(t('chatComposer.openSettings'))"
        :disabled="loading"
        @click="onOpenSettings"
      >
        {{ t("chatComposer.gear") }}
      </button>
    </div>
    <p class="composer-hint">{{ t("chatComposer.hint") }}</p>

    <div class="composer-body">
      <div class="composer-input-col">
        <label class="sr-only" for="chat-user-message">{{ t("chat.input.label") }}</label>
        <textarea
          id="chat-user-message"
          ref="textAreaEl"
          v-model="text"
          class="composer-textarea"
          name="user_message"
          rows="2"
          autocomplete="off"
          :placeholder="String(t('chat.input.placeholder'))"
          :disabled="loading"
          @keydown="onKeydown"
        />
      </div>
      <button type="button" class="composer-send" :disabled="loading || !text.trim()" @click="submit">
        {{ t("chat.input.send") }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.composer {
  margin: 0 18px 14px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: 0 1px 0 color-mix(in srgb, var(--text-primary) 4%, transparent);
  overflow: hidden;
}
.composer-toolbar {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 85%, transparent);
  background: color-mix(in srgb, var(--bg-primary) 55%, var(--bg-elevated));
}
.composer-model-label {
  flex: 0 0 auto;
  margin-top: 7px;
  font-size: 12px;
  font-weight: 650;
  color: var(--text-secondary);
  white-space: nowrap;
}
.composer-toolbar-mid {
  flex: 1 1 auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.composer-select {
  width: 100%;
  max-width: 100%;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  box-sizing: border-box;
  cursor: pointer;
}
.composer-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
}
.composer-custom-wrap {
  width: 100%;
}
.composer-custom-input {
  width: 100%;
  box-sizing: border-box;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px dashed color-mix(in srgb, var(--accent) 35%, var(--border-light));
  background: var(--bg-primary);
  color: var(--text-primary);
}
.composer-custom-input:focus {
  outline: none;
  border-style: solid;
  border-color: var(--accent);
}
.composer-gear {
  flex: 0 0 auto;
  margin-top: 4px;
  padding: 6px 10px;
  font-size: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
}
.composer-gear:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: color-mix(in srgb, var(--accent) 35%, var(--border-light));
}
.composer-gear:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.composer-hint {
  margin: 0;
  padding: 4px 12px 6px;
  font-size: 11px;
  line-height: 1.35;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--bg-primary) 40%, var(--bg-elevated));
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
}
.composer-body {
  display: flex;
  gap: 10px;
  padding: 10px 12px 12px;
  align-items: flex-start;
  background: var(--bg-primary);
}
.composer-input-col {
  flex: 1 1 auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.composer-textarea {
  width: 100%;
  border: none;
  border-radius: 8px;
  padding: 10px 12px;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-elevated) 92%, var(--bg-primary));
  resize: none;
  outline: none;
  font-size: 14px;
  box-sizing: border-box;
  min-height: 52px;
  transition: box-shadow var(--ease, 0.2s ease);
}
.composer-textarea::placeholder {
  color: var(--text-light);
}
.composer-textarea:focus {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 18%, transparent);
}
.composer-textarea:focus-visible {
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 28%, transparent),
    0 0 0 4px color-mix(in srgb, var(--focus-ring-color) 10%, transparent);
}
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
.composer-send {
  min-width: 88px;
  align-self: flex-start;
  margin-top: 2px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn, 8px);
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  padding: 10px 14px;
  transition: border-color var(--ease, 0.2s ease), background var(--ease, 0.2s ease),
    transform var(--ease, 0.2s ease), box-shadow var(--ease, 0.2s ease);
}
.composer-send:hover:not(:disabled) {
  background: linear-gradient(
    135deg,
    var(--btn-primary-hover-a),
    var(--btn-primary-hover-b)
  );
  border-color: var(--accent);
  transform: translateY(-1px);
  box-shadow: var(--shadow-btn-hover);
}
.composer-send:focus-visible {
  border-color: var(--accent);
  box-shadow:
    var(--shadow-btn-hover),
    0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 45%, transparent);
}
.composer-send:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}
</style>
