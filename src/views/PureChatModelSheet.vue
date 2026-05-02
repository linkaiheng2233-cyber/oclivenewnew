<script setup lang="ts">
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import CloudLlmQuickSetup from "../components/CloudLlmQuickSetup.vue";
import HostModelPickRow from "../components/HostModelPickRow.vue";
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

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void pick.bootstrap();
      void ollamaModelsHealth().then(() => {
        void pick.loadOllama();
      });
    }
  },
);

function onCloudSaved(): void {
  void pick.loadCloudPublic().then(() => {
    pick.syncSelectFromModel();
  });
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
            <section class="pcm-sec">
              <h3 class="pcm-h3">{{ t("pureChatModelSheet.sectionChatModel") }}</h3>
              <p class="pcm-muted">{{ t("pureChatModelSheet.sectionChatModelHint") }}</p>
              <HostModelPickRow
                select-id="pure-chat-model-select"
                :show-gear="false"
                @open-settings="emit('openSettings')"
              />
              <button type="button" class="pcm-linkish" @click="emit('openSettings')">
                {{ t("pureChatModelSheet.openFullSettings") }}
              </button>
            </section>

            <section class="pcm-sec">
              <h3 class="pcm-h3">{{ t("pureChatModelSheet.sectionLocal") }}</h3>
              <p class="pcm-muted">{{ t("pureChatModelSheet.sectionLocalHint") }}</p>
              <div class="pcm-pill" :class="ollamaOnline ? 'pcm-pill--ok' : 'pcm-pill--off'">
                {{
                  ollamaOnline
                    ? t("pureChatModelSheet.ollamaOnline")
                    : t("pureChatModelSheet.ollamaOffline")
                }}
              </div>
              <ul v-if="ollamaNames.length" class="pcm-ul">
                <li v-for="n in ollamaNames" :key="n" class="pcm-li">{{ n }}</li>
              </ul>
              <p v-else class="pcm-muted pcm-tiny">{{ t("pureChatModelSheet.noLocalModels") }}</p>
            </section>

            <section class="pcm-sec pcm-sec--cloud">
              <h3 class="pcm-h3">{{ t("pureChatModelSheet.sectionCloud") }}</h3>
              <p class="pcm-muted">{{ t("pureChatModelSheet.sectionCloudHint") }}</p>
              <CloudLlmQuickSetup @saved="onCloudSaved" />
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
  width: min(520px, 100%);
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
  margin-bottom: 16px;
}
.pcm-sec:last-child {
  margin-bottom: 0;
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
.pcm-ul {
  margin: 0;
  padding: 0 0 0 16px;
  max-height: 140px;
  overflow-y: auto;
  font-size: 12px;
  color: var(--text-primary);
}
.pcm-li {
  margin: 2px 0;
}
.pcm-linkish {
  margin-top: 8px;
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
