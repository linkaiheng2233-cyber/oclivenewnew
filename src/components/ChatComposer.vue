<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { hostEventBus } from "../lib/hostEventBus";
import HostModelPickRow from "./HostModelPickRow.vue";

const props = defineProps<{ loading: boolean }>();

const emit = defineEmits<{
  send: [payload: { content: string }];
  openSettings: [];
  "clear-stuck-loading": [];
}>();

const { t } = useI18n();

const text = ref("");
const textAreaEl = ref<HTMLTextAreaElement | null>(null);

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

onMounted(() => {
  hostEventBus.on("chat:set_input_draft", onSetDraftInput);
});

onBeforeUnmount(() => {
  hostEventBus.off("chat:set_input_draft", onSetDraftInput);
});
</script>

<template>
  <section class="composer" role="region" :aria-label="String(t('chatComposer.aria'))">
    <div class="composer-toolbar">
      <HostModelPickRow
        select-id="oclive-composer-model-select"
        :disabled="loading"
        @open-settings="emit('openSettings')"
      />
    </div>
    <p class="composer-hint">{{ t("chatComposer.hint") }}</p>

    <div v-if="loading" class="composer-wait-bar" role="status">
      <span>{{ t("chatComposer.generatingHint") }}</span>
      <button type="button" class="composer-wait-btn" @click="emit('clear-stuck-loading')">
        {{ t("chatComposer.endWaiting") }}
      </button>
    </div>

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
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 85%, transparent);
  background: color-mix(in srgb, var(--bg-primary) 55%, var(--bg-elevated));
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
.composer-wait-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--accent) 8%, var(--bg-primary));
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
}
.composer-wait-btn {
  border-radius: 8px;
  border: 1px solid var(--border-light);
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.composer-wait-btn:hover {
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border-light));
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
