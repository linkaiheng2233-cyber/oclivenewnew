<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { useRoleStore } from '@oclive/shared/stores/roleStore'

const props = defineProps<{ loading: boolean }>()

const emit = defineEmits<{
  send: [payload: { content: string }]
}>()
const { t } = useI18n()
const roleStore = useRoleStore()

const text = ref('')
const textAreaEl = ref<HTMLTextAreaElement | null>(null)

const placeholder = computed(() =>
  t('common.chatPlaceholder', {
    name: roleStore.roleInfo.name?.trim() || t('app.defaultRoleName'),
  }),
)

function onSetDraftInput(payload: unknown): void {
  const raw = (payload as { text?: string } | null)?.text
  const next = typeof raw === 'string' ? raw.trim() : ''
  if (!next)
    return
  text.value = next
  void nextTick(() => {
    textAreaEl.value?.focus()
    textAreaEl.value?.setSelectionRange(next.length, next.length)
  })
}

function focusInput(): void {
  void nextTick(() => {
    textAreaEl.value?.focus()
  })
}

function submit() {
  const value = text.value.trim()
  if (!value || props.loading)
    return
  emit('send', { content: value })
  text.value = ''
  focusInput()
}

defineExpose({ focusInput })

function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Enter')
    return
  /* Same as oclive-new Enter to send; Shift+Enter keeps newline */
  if (e.shiftKey)
    return
  e.preventDefault()
  submit()
}

onMounted(() => {
  hostEventBus.on('chat:set_input_draft', onSetDraftInput)
})

onBeforeUnmount(() => {
  hostEventBus.off('chat:set_input_draft', onSetDraftInput)
})
</script>

<template>
  <section class="input-row">
    <div class="input-col">
      <label class="sr-only" for="chat-user-message">{{ t("common.chatInputLabel") }}</label>
      <textarea
        id="chat-user-message"
        ref="textAreaEl"
        v-model="text"
        class="input"
        name="user_message"
        rows="2"
        autocomplete="off"
        :placeholder="placeholder"
        :disabled="loading"
        @keydown="onKeydown"
      />
    </div>
    <button
      type="button"
      class="send"
      :disabled="loading || !text.trim()"
      @click="submit"
    >
      {{ t("common.send") }}
    </button>
  </section>
</template>

<style scoped>
/* Match oclive-new #userInput + #sendBtn */
.input-row {
  display: flex;
  gap: 10px;
  padding: 16px 18px;
  background: var(--bg-primary);
}
.input-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.input {
  width: 100%;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 10px 16px;
  color: var(--text-primary);
  background: var(--bg-elevated);
  resize: none;
  outline: none;
  transition: border-color var(--ease), box-shadow var(--ease);
  font-size: 14px;
  box-sizing: border-box;
}
.input::placeholder {
  color: var(--text-light);
}
.input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 22%, transparent);
}
.input:focus-visible {
  border-color: var(--accent);
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 28%, transparent),
    0 0 0 4px color-mix(in srgb, var(--focus-ring-color) 12%, transparent);
}
/* Screen-reader only; satisfies associated-label a11y checks */
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
.send {
  min-width: 88px;
  align-self: flex-start;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color var(--ease), background var(--ease), transform var(--ease),
    box-shadow var(--ease);
}
.send:hover {
  background: linear-gradient(
    135deg,
    var(--btn-primary-hover-a),
    var(--btn-primary-hover-b)
  );
  border-color: var(--accent);
  transform: translateY(-1px);
  box-shadow: var(--shadow-btn-hover);
}
.send:focus {
  outline: none;
}
.send:focus-visible {
  border-color: var(--accent);
  box-shadow:
    var(--shadow-btn-hover),
    0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 45%, transparent);
}
.send:active {
  transform: translateY(0);
}
.send:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}
</style>
