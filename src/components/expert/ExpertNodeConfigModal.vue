<script setup lang="ts">
import type { ExpertRoutingDoc } from '../../api/role/expert'
import { computed, ref, toRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '../../composables/useModalFocusRestore'
import {
  applyLlmNodeConfig,
  applyLoraNodeConfig,
  applyMemoryNodeConfig,
  applyPersonalityNodeConfig,
  applyPromptNodeConfig,
  configKindForSlotType,
  loadLlmFormFromDoc,
  loadLoraFormFromDoc,
  loadMemoryFormFromDoc,
  loadPersonalityFormFromDoc,
  loadPromptFormFromDoc,
  type ExpertNodeConfigKind,
} from '../../lib/expertNodeRouting'
import { PERSONALITY_TRAIT_KEYS, vec7ToRecord } from '../../utils/personality-traits'
import { useRoleStore } from '../../stores/roleStore'

const props = defineProps<{
  open: boolean
  slotKey: string
  slotType: string
  routingDoc: ExpertRoutingDoc | null | undefined
  busy?: boolean
}>()

const emit = defineEmits<{
  close: []
  preview: [before: ExpertRoutingDoc | null, after: ExpertRoutingDoc]
}>()

const { t } = useI18n()
const roleStore = useRoleStore()
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'open'), dialogRef)

const configKind = computed<ExpertNodeConfigKind>(() =>
  configKindForSlotType(props.slotType),
)

const llmSlotKeys = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack
  if (!pack) {
    return []
  }
  return Object.entries(pack)
    .filter(([, e]) => e.type === 'llm')
    .map(([k, e]) => ({ key: k, label: e.label || k }))
})

const loraPluginOptions = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack
  if (!pack) {
    return []
  }
  return Object.entries(pack)
    .filter(([, e]) => e.type === 'lora' || e.backend === 'directory')
    .map(([k, e]) => ({ key: k, label: e.label || k }))
})

const llmForm = ref(loadLlmFormFromDoc(null, ''))
const personalityTraits = ref<Record<string, number>>({})
const personalityDelta = ref(0.05)
const primaryTrait = ref<string>(PERSONALITY_TRAIT_KEYS[0])
const promptText = ref('')
const memoryContent = ref('')
const loraPluginId = ref('')

function resetForms() {
  const key = props.slotKey
  llmForm.value = loadLlmFormFromDoc(props.routingDoc, key)
  const defaults = vec7ToRecord(roleStore.roleInfo.personality)
  const pf = loadPersonalityFormFromDoc(props.routingDoc, defaults)
  personalityTraits.value = { ...pf.traits }
  personalityDelta.value = pf.delta
  promptText.value = loadPromptFormFromDoc(props.routingDoc).text
  memoryContent.value = loadMemoryFormFromDoc(props.routingDoc).content
  loraPluginId.value = loadLoraFormFromDoc(props.routingDoc).pluginId
}

watch(
  () => [props.open, props.slotKey, props.routingDoc] as const,
  ([open]) => {
    if (open) {
      resetForms()
    }
  },
  { immediate: true },
)

function buildAfterDoc(): ExpertRoutingDoc {
  const key = props.slotKey
  const doc = props.routingDoc
  switch (configKind.value) {
    case 'llm':
      return applyLlmNodeConfig(doc, key, llmForm.value)
    case 'personality':
      return applyPersonalityNodeConfig(
        doc,
        key,
        props.slotType,
        { traits: personalityTraits.value, delta: personalityDelta.value },
        primaryTrait.value,
      )
    case 'prompt':
      return applyPromptNodeConfig(doc, key, props.slotType, { text: promptText.value })
    case 'memory':
      return applyMemoryNodeConfig(doc, key, props.slotType, { content: memoryContent.value })
    case 'lora':
      return applyLoraNodeConfig(doc, key, props.slotType, { pluginId: loraPluginId.value })
    default:
      return applyLlmNodeConfig(doc, key, llmForm.value)
  }
}

function onPreview() {
  emit('preview', props.routingDoc, buildAfterDoc())
}

function onBackdrop(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('encm-backdrop')) {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="open"
    class="encm-backdrop"
    role="dialog"
    aria-modal="true"
    :aria-label="t('expertConfig.nodeConfig.title', { slot: slotKey })"
    @click="onBackdrop"
  >
    <form ref="dialogRef" class="encm-panel" tabindex="-1" @submit.prevent="onPreview">
      <h3 class="encm-title">
        {{ t("expertConfig.nodeConfig.title", { slot: slotKey }) }}
      </h3>
      <p class="encm-kind">
        {{ t(`expertConfig.nodeConfig.kind.${configKind}`) }}
      </p>

      <template v-if="configKind === 'llm'">
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.llmModel") }}</span>
          <select v-model="llmForm.llmSlotKey" class="encm-input" :disabled="busy">
            <option v-for="opt in llmSlotKeys" :key="opt.key" :value="opt.key">
              {{ opt.label }}
            </option>
          </select>
        </label>
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.temperature") }} ({{ llmForm.temperature.toFixed(2) }})</span>
          <input
            v-model.number="llmForm.temperature"
            class="encm-range"
            type="range"
            min="0"
            max="2"
            step="0.05"
            :disabled="busy"
          >
        </label>
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.maxTokens") }}</span>
          <input
            v-model.number="llmForm.maxTokens"
            class="encm-input"
            type="number"
            min="256"
            max="128000"
            step="256"
            :disabled="busy"
          >
        </label>
      </template>

      <template v-else-if="configKind === 'personality'">
        <label
          v-for="trait in PERSONALITY_TRAIT_KEYS"
          :key="trait"
          class="encm-field"
        >
          <span>
            {{ t(`editor.personalityTrait.${trait}`) }}
            ({{ personalityTraits[trait]?.toFixed(2) }})
          </span>
          <input
            v-model.number="personalityTraits[trait]"
            class="encm-range"
            type="range"
            min="0"
            max="1"
            step="0.01"
            :disabled="busy"
            @change="primaryTrait = trait"
          >
        </label>
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.adjustDelta") }} ({{ personalityDelta.toFixed(2) }})</span>
          <input
            v-model.number="personalityDelta"
            class="encm-range"
            type="range"
            min="-0.5"
            max="0.5"
            step="0.01"
            :disabled="busy"
          >
        </label>
      </template>

      <template v-else-if="configKind === 'prompt'">
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.promptText") }}</span>
          <textarea
            v-model="promptText"
            class="encm-textarea"
            rows="5"
            :disabled="busy"
          />
        </label>
      </template>

      <template v-else-if="configKind === 'memory'">
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.memoryContent") }}</span>
          <textarea
            v-model="memoryContent"
            class="encm-textarea"
            rows="5"
            :disabled="busy"
          />
        </label>
      </template>

      <template v-else-if="configKind === 'lora'">
        <label class="encm-field">
          <span>{{ t("expertConfig.nodeConfig.loraPlugin") }}</span>
          <select v-model="loraPluginId" class="encm-input" :disabled="busy">
            <option value="">
              {{ t("expertConfig.nodeConfig.loraNone") }}
            </option>
            <option v-for="opt in loraPluginOptions" :key="opt.key" :value="opt.key">
              {{ opt.label }}
            </option>
          </select>
        </label>
      </template>

      <div class="encm-actions">
        <button type="button" class="encm-btn" :disabled="busy" @click="emit('close')">
          {{ t("expertConfig.cancel") }}
        </button>
        <button type="submit" class="encm-btn encm-btn--primary" :disabled="busy">
          {{ t("expertConfig.nodeConfig.previewBlueprint") }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.encm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
  padding: 12px;
}
.encm-panel {
  width: min(480px, 94vw);
  max-height: 88vh;
  overflow: auto;
  padding: 16px 18px;
  border-radius: 10px;
  background: var(--bg-elevated, #1e1e24);
  border: 1px solid var(--border-light, #444);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
}
.encm-title {
  margin: 0 0 4px;
  font-size: 15px;
}
.encm-kind {
  margin: 0 0 14px;
  font-size: 11px;
  color: var(--text-secondary);
}
.encm-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 12px;
}
.encm-input,
.encm-textarea {
  font-size: 12px;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light, #444);
  background: #121218;
  color: var(--text-primary);
}
.encm-range {
  width: 100%;
}
.encm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
.encm-btn {
  font-size: 12px;
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid var(--border-light, #444);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
}
.encm-btn--primary {
  background: #e8a317;
  border-color: transparent;
  color: #1a1a1a;
  font-weight: 600;
}
.encm-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
