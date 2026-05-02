<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import {
  getHostChatModel,
  ollamaModelsHealth,
  ollamaModelsListNames,
  setHostChatModel,
} from "../utils/tauri-api";

const emit = defineEmits<{ openSettings: [] }>();

const { t } = useI18n();
const { showToast } = useAppToast();

const modelId = ref("");
const ollamaNames = ref<string[]>([]);
const listId = "oclive-chat-model-datalist";
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let lastSaved = "";

async function loadOllamaSuggestions(): Promise<void> {
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

function schedulePersist(): void {
  if (saveTimer != null) {
    window.clearTimeout(saveTimer);
    saveTimer = null;
  }
  saveTimer = window.setTimeout(() => {
    saveTimer = null;
    void persistModel();
  }, 450);
}

async function persistModel(): Promise<void> {
  const m = modelId.value.trim();
  if (!m) {
    showToast("error", String(t("chatModelBar.errEmpty")));
    modelId.value = lastSaved;
    return;
  }
  if (m === lastSaved) return;
  try {
    await setHostChatModel(m);
    lastSaved = m;
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
    modelId.value = lastSaved;
  }
}

onMounted(async () => {
  try {
    const cur = await getHostChatModel();
    modelId.value = cur.trim();
    lastSaved = modelId.value;
  } catch {
    modelId.value = "";
  }
  void loadOllamaSuggestions();
});

function onOpenCloudSettings(): void {
  emit("openSettings");
}
</script>

<template>
  <div class="cmb" role="group" :aria-label="String(t('chatModelBar.aria'))">
    <label class="cmb-label" for="oclive-global-chat-model">{{ t("chatModelBar.label") }}</label>
    <div class="cmb-row">
      <input
        id="oclive-global-chat-model"
        v-model="modelId"
        type="text"
        class="cmb-input"
        spellcheck="false"
        autocomplete="off"
        :list="ollamaNames.length ? listId : undefined"
        :placeholder="String(t('chatModelBar.placeholder'))"
        @input="schedulePersist"
        @change="void persistModel()"
      />
      <datalist v-if="ollamaNames.length" :id="listId">
        <option v-for="n in ollamaNames" :key="n" :value="n" />
      </datalist>
      <button type="button" class="cmb-gear" :title="String(t('chatModelBar.openCloudSettings'))" @click="onOpenCloudSettings">
        {{ t("chatModelBar.gear") }}
      </button>
    </div>
    <p class="cmb-hint">{{ t("chatModelBar.hint") }}</p>
  </div>
</template>

<style scoped>
.cmb {
  padding: 8px 18px 0;
  background: var(--bg-primary);
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
}
.cmb-label {
  display: block;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--text-secondary);
  margin-bottom: 4px;
}
.cmb-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.cmb-input {
  flex: 1 1 auto;
  min-width: 0;
  padding: 6px 10px;
  font-size: 13px;
  border-radius: var(--radius-btn, 8px);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  box-sizing: border-box;
}
.cmb-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 22%, transparent);
}
.cmb-gear {
  flex: 0 0 auto;
  padding: 6px 10px;
  font-size: 12px;
  border-radius: var(--radius-btn, 8px);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  cursor: pointer;
}
.cmb-gear:hover {
  color: var(--text-primary);
  border-color: color-mix(in srgb, var(--accent) 35%, var(--border-light));
}
.cmb-hint {
  margin: 4px 0 0;
  font-size: 11px;
  line-height: 1.35;
  color: var(--text-secondary);
}
</style>
