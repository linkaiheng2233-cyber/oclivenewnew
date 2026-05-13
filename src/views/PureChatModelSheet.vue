<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import CloudLlmQuickSetup from "../components/CloudLlmQuickSetup.vue";
import { useHostModelPick } from "../composables/useHostModelPick";
import { ollamaModelsHealth } from "../utils/tauri-api";

const props = defineProps<{ visible: boolean }>();

const emit = defineEmits<{
  close: [];
  openSettings: [];
}>();

const { t } = useI18n();
const pick = useHostModelPick();
const ollamaNames = pick.ollamaNames;

const ollamaOnline = computed(() => ollamaNames.value.length > 0);
const currentModel = computed(() => pick.modelId.value.trim());

const customOllama = ref("");
const applying = ref(false);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      customOllama.value = "";
      void pick.bootstrap();
      void ollamaModelsHealth().then(() => {
        void pick.loadOllama();
      });
    }
  },
);

async function onPickOllama(name: string): Promise<void> {
  if (applying.value) return;
  applying.value = true;
  try {
    await pick.applyChatModelId(name);
  } finally {
    applying.value = false;
  }
}

async function onApplyCustomOllama(): Promise<void> {
  await onPickOllama(customOllama.value);
  customOllama.value = "";
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="pcm-stack">
      <div class="pcm-dim" role="presentation" @click.self="emit('close')">
        <div class="pcm-dialog" @click.stop>
          <header class="pcm-head">
            <div class="pcm-head-text">
              <h2 class="pcm-title">{{ t("pureChatModelSheet.title") }}</h2>
              <p class="pcm-lead">{{ t("pureChatModelSheet.lead") }}</p>
            </div>
            <button type="button" class="pcm-close" @click="emit('close')">
              {{ t("pureChatModelSheet.close") }}
            </button>
          </header>

          <div class="pcm-body">
            <section class="pcm-sec pcm-card">
              <h3 class="pcm-h3">{{ t("pureChatModelSheet.sectionOllama") }}</h3>
              <p class="pcm-muted">{{ t("pureChatModelSheet.sectionOllamaHint") }}</p>
              <div class="pcm-pill" :class="ollamaOnline ? 'pcm-pill--ok' : 'pcm-pill--off'">
                {{
                  ollamaOnline
                    ? t("pureChatModelSheet.ollamaOnline")
                    : t("pureChatModelSheet.ollamaOffline")
                }}
              </div>
              <p v-if="currentModel" class="pcm-current">
                {{ t("pureChatModelSheet.currentModel", { id: currentModel }) }}
              </p>
              <div v-if="ollamaNames.length" class="pcm-btn-grid" role="list">
                <button
                  v-for="n in ollamaNames"
                  :key="n"
                  type="button"
                  class="pcm-model-btn"
                  :class="{ 'pcm-model-btn--active': currentModel === n }"
                  :disabled="applying"
                  role="listitem"
                  @click="void onPickOllama(n)"
                >
                  {{ n }}
                </button>
              </div>
              <p v-else class="pcm-muted pcm-tiny">{{ t("pureChatModelSheet.noLocalModels") }}</p>

              <div class="pcm-custom-row">
                <input
                  v-model="customOllama"
                  type="text"
                  class="pcm-custom-input"
                  spellcheck="false"
                  autocomplete="off"
                  :disabled="applying"
                  :placeholder="String(t('pureChatModelSheet.customOllamaPlaceholder'))"
                  @keydown.enter.prevent="void onApplyCustomOllama()"
                />
                <button
                  type="button"
                  class="pcm-custom-apply"
                  :disabled="applying || !customOllama.trim()"
                  @click="void onApplyCustomOllama()"
                >
                  {{ t("pureChatModelSheet.customOllamaApply") }}
                </button>
              </div>

              <button type="button" class="pcm-linkish" @click="emit('openSettings')">
                {{ t("pureChatModelSheet.openFullSettings") }}
              </button>
            </section>

            <section class="pcm-sec pcm-card pcm-sec--cloud">
              <h3 class="pcm-h3">{{ t("pureChatModelSheet.sectionCloud") }}</h3>
              <p class="pcm-muted">{{ t("pureChatModelSheet.sectionCloudHint") }}</p>
              <CloudLlmQuickSetup variant="pureChat" />
            </section>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pcm-stack {
  position: fixed;
  inset: 0;
  z-index: 10061;
  isolation: isolate;
  pointer-events: auto;
}
.pcm-dim {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.pcm-dialog {
  width: min(560px, 100%);
  max-height: min(88vh, 720px);
  display: flex;
  flex-direction: column;
  border-radius: 14px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  box-shadow: 0 16px 48px color-mix(in srgb, #000 22%, transparent);
  overflow: hidden;
}
.pcm-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 40%, var(--bg-elevated));
}
.pcm-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}
.pcm-lead {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.pcm-close {
  flex-shrink: 0;
  padding: 6px 12px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.pcm-body {
  padding: 12px 16px 16px;
  overflow-y: auto;
  flex: 1 1 auto;
  min-height: 0;
}
.pcm-sec {
  margin-bottom: 12px;
}
.pcm-sec:last-child {
  margin-bottom: 0;
}
.pcm-card {
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 55%, var(--bg-elevated));
}
.pcm-h3 {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 650;
  color: var(--text-primary);
}
.pcm-muted {
  margin: 0 0 8px;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.pcm-tiny {
  font-size: 11px;
}
.pcm-current {
  margin: 0 0 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}
.pcm-pill {
  display: inline-block;
  margin-bottom: 8px;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 600;
  border-radius: 999px;
  border: 1px solid var(--border-light);
}
.pcm-pill--ok {
  color: color-mix(in srgb, #16a34a 92%, var(--text-primary));
  border-color: color-mix(in srgb, #16a34a 35%, var(--border-light));
  background: color-mix(in srgb, #16a34a 10%, var(--bg-primary));
}
.pcm-pill--off {
  color: var(--text-secondary);
}
.pcm-btn-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}
.pcm-model-btn {
  padding: 8px 12px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  max-width: 100%;
}
.pcm-model-btn:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 45%, var(--border-light));
}
.pcm-model-btn--active {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 14%, var(--bg-primary));
  font-weight: 650;
}
.pcm-model-btn:disabled {
  opacity: 0.55;
  cursor: wait;
}
.pcm-custom-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
}
.pcm-custom-input {
  flex: 1 1 160px;
  min-width: 0;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  box-sizing: border-box;
}
.pcm-custom-apply {
  padding: 7px 12px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.pcm-custom-apply:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.pcm-linkish {
  margin-top: 4px;
  padding: 0;
  font-size: 12px;
  border: none;
  background: none;
  color: var(--accent);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}
.pcm-sec--cloud :deep(.clqs) {
  margin-top: 4px;
}
</style>
