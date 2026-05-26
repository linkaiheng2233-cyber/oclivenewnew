<script setup lang="ts">
import type { ExpertRouteStep, ExpertRoutingDoc, ExpertRoute } from '../../api/role/expert'
import { EXPERT_FACILITY_ACTIONS } from '../../api/role/expert'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getExpertRouting,
  listBlueprintIncludes,
  saveExpertRouting,
} from '../../api/role/expert'
import { useAppToast } from '../../composables/useAppToast'
import { useRoleStore } from '../../stores/roleStore'

const EMOTION_OPTIONS = [
  'happy',
  'sad',
  'angry',
  'neutral',
  'excited',
  'confused',
  'shy',
] as const

const { t } = useI18n()
const roleStore = useRoleStore()
const { showToast } = useAppToast()

const includesFiles = ref<string[]>([])
const loading = ref(false)
const wizardOpen = ref(false)
const wizardStep = ref(1)

const selectedScenes = ref<string[]>([])
const keywordsText = ref('')
const selectedEmotions = ref<string[]>([])
const minLen = ref<number | ''>('')
const maxLen = ref<number | ''>('')
const timeAfter = ref('')
const timeBefore = ref('')
const selectedRelations = ref<string[]>([])
const routePriority = ref<number | ''>(10)

const selectedLlmKey = ref('')
const extraAnalyze = ref(false)
const facilityAction = ref<string>('')
const paramTrait = ref('warmth')
const paramDelta = ref(0.05)
const paramPromptText = ref('')
const paramMemoryContent = ref('')
const paramLoraPluginId = ref('')
const previewJson = ref('')

const sceneOptions = computed(() =>
  (roleStore.roleInfo.scene_labels ?? []).map(s => s.id),
)
const relationOptions = computed(() =>
  (roleStore.roleInfo.user_relations ?? []).map(r => r.id),
)

const llmSlotKeys = computed(() => {
  const pack = roleStore.roleInfo.slotRegistryPack
  if (!pack)
    return []
  return Object.entries(pack)
    .filter(([, e]) => e.type === 'llm')
    .map(([k, e]) => ({ key: k, label: e.label || k }))
})

watch(
  llmSlotKeys,
  (keys) => {
    if (keys.length && !keys.some(k => k.key === selectedLlmKey.value)) {
      selectedLlmKey.value = keys[0]?.key ?? ''
    }
  },
  { immediate: true },
)

async function refresh() {
  const roleId = roleStore.currentRoleId
  if (!roleId)
    return
  loading.value = true
  try {
    includesFiles.value = await listBlueprintIncludes(roleId)
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
  finally {
    loading.value = false
  }
}

function openWizard() {
  wizardStep.value = 1
  selectedScenes.value = []
  keywordsText.value = ''
  selectedEmotions.value = []
  minLen.value = ''
  maxLen.value = ''
  timeAfter.value = ''
  timeBefore.value = ''
  selectedRelations.value = []
  routePriority.value = 10
  extraAnalyze.value = false
  facilityAction.value = ''
  previewJson.value = ''
  wizardOpen.value = true
}

function buildSteps(): ExpertRoute['steps'] {
  const steps: ExpertRouteStep[] = []
  if (extraAnalyze.value) {
    const emotionKey = Object.keys(roleStore.roleInfo.slotRegistryPack ?? {}).find(
      k => roleStore.roleInfo.slotRegistryPack?.[k]?.type === 'emotion',
    )
    if (emotionKey) {
      steps.push({ action: `slot.${emotionKey}.analyze`, depends_on: [] })
    }
  }
  const llm = selectedLlmKey.value.trim()
  if (llm) {
    const dep = steps.length ? [steps[steps.length - 1]!.action] : []
    steps.push({ action: `slot.${llm}.generate`, depends_on: dep })
  }
  if (facilityAction.value) {
    const dep = steps.length ? [steps[steps.length - 1]!.action] : []
    const step: ExpertRouteStep = {
      action: facilityAction.value,
      depends_on: dep,
    }
    if (facilityAction.value === 'slot.personality.adjust') {
      step.params = { trait: paramTrait.value, delta: paramDelta.value }
    }
    else if (facilityAction.value === 'slot.prompt_enhance.apply') {
      step.params = { text: paramPromptText.value }
    }
    else if (facilityAction.value === 'slot.memory.inject') {
      step.params = { content: paramMemoryContent.value, importance: 0.85 }
    }
    else if (facilityAction.value === 'slot.lora.apply') {
      step.params = { plugin_id: paramLoraPluginId.value }
    }
    steps.push(step)
  }
  return steps
}

function buildDoc(): ExpertRoutingDoc {
  const keywords = keywordsText.value
    .split(/[,，\s]+/)
    .map(s => s.trim())
    .filter(Boolean)
  const route: ExpertRoute = {
    id: 'wizard-route',
    enabled: true,
    priority: routePriority.value === '' ? undefined : Number(routePriority.value),
    trigger: {
      scenes: selectedScenes.value.length ? [...selectedScenes.value] : undefined,
      keywords: keywords.length ? keywords : undefined,
      user_emotion: selectedEmotions.value.length ? [...selectedEmotions.value] : undefined,
      message_length:
        minLen.value !== '' || maxLen.value !== ''
          ? {
              min: minLen.value === '' ? undefined : Number(minLen.value),
              max: maxLen.value === '' ? undefined : Number(maxLen.value),
            }
          : undefined,
      time_of_day:
        timeAfter.value || timeBefore.value
          ? {
              after: timeAfter.value || undefined,
              before: timeBefore.value || undefined,
            }
          : undefined,
      user_relation: selectedRelations.value.length ? [...selectedRelations.value] : undefined,
    },
    steps: buildSteps(),
  }
  return {
    fallback: 'skip',
    routes: [route],
  }
}

function advancePreview() {
  previewJson.value = JSON.stringify(buildDoc(), null, 2)
  wizardStep.value = 5
}

async function confirmSave() {
  const roleId = roleStore.currentRoleId
  if (!roleId)
    return
  try {
    const doc = buildDoc()
    await saveExpertRouting(roleId, doc)
    showToast('success', t('expertConfig.toast.saved'))
    wizardOpen.value = false
    await refresh()
  }
  catch (e) {
    showToast('error', e instanceof Error ? e.message : String(e))
  }
}

function toggleInList(list: string[], id: string) {
  const i = list.indexOf(id)
  if (i >= 0)
    list.splice(i, 1)
  else
    list.push(id)
}

onMounted(() => {
  void refresh()
})

watch(
  () => roleStore.currentRoleId,
  () => void refresh(),
)
</script>

<template>
  <section class="expert-panel">
    <header class="expert-head">
      <h3 class="expert-title">
        {{ t('expertConfig.title') }}
      </h3>
      <p class="expert-hint">
        {{ t('expertConfig.hint') }}
      </p>
    </header>

    <p v-if="loading" class="expert-muted">
      {{ t('expertConfig.loading') }}
    </p>
    <ul v-else-if="includesFiles.length" class="expert-file-list">
      <li v-for="f in includesFiles" :key="f">
        <code>blueprint/includes/{{ f }}</code>
      </li>
    </ul>
    <p v-else class="expert-muted">
      {{ t('expertConfig.noIncludes') }}
    </p>

    <div class="expert-actions">
      <button type="button" class="expert-btn primary" @click="openWizard">
        {{ t('expertConfig.newWizard') }}
      </button>
    </div>

    <div v-if="wizardOpen" class="expert-wizard" role="dialog" :aria-label="t('expertConfig.wizardAria')">
      <p class="expert-step-label">
        {{ t('expertConfig.step', { n: wizardStep }) }}
      </p>

      <div v-if="wizardStep === 1" class="expert-step">
        <label class="expert-label">{{ t('expertConfig.trigger.scenes') }}</label>
        <div v-if="sceneOptions.length" class="expert-chips">
          <label v-for="sid in sceneOptions" :key="sid" class="expert-chip">
            <input
              type="checkbox"
              :checked="selectedScenes.includes(sid)"
              @change="toggleInList(selectedScenes, sid)"
            >
            {{ sid }}
          </label>
        </div>
        <p v-else class="expert-muted">
          {{ t('expertConfig.trigger.noScenes') }}
        </p>
        <label class="expert-label">{{ t('expertConfig.trigger.keywords') }}</label>
        <input v-model="keywordsText" type="text" class="expert-input" :placeholder="t('expertConfig.trigger.keywordsPh')">
        <label class="expert-label">{{ t('expertConfig.trigger.emotions') }}</label>
        <div class="expert-chips">
          <label v-for="emo in EMOTION_OPTIONS" :key="emo" class="expert-chip">
            <input
              type="checkbox"
              :checked="selectedEmotions.includes(emo)"
              @change="toggleInList(selectedEmotions, emo)"
            >
            {{ emo }}
          </label>
        </div>
        <button type="button" class="expert-btn" @click="wizardStep = 2">
          {{ t('expertConfig.next') }}
        </button>
      </div>

      <div v-else-if="wizardStep === 2" class="expert-step">
        <label class="expert-label">{{ t('expertConfig.trigger.minLen') }}</label>
        <input v-model.number="minLen" type="number" min="0" class="expert-input">
        <label class="expert-label">{{ t('expertConfig.trigger.maxLen') }}</label>
        <input v-model.number="maxLen" type="number" min="0" class="expert-input">
        <label class="expert-label">{{ t('expertConfig.trigger.timeAfter') }}</label>
        <input v-model="timeAfter" type="time" class="expert-input">
        <label class="expert-label">{{ t('expertConfig.trigger.timeBefore') }}</label>
        <input v-model="timeBefore" type="time" class="expert-input">
        <label class="expert-label">{{ t('expertConfig.trigger.relations') }}</label>
        <div v-if="relationOptions.length" class="expert-chips">
          <label v-for="rid in relationOptions" :key="rid" class="expert-chip">
            <input
              type="checkbox"
              :checked="selectedRelations.includes(rid)"
              @change="toggleInList(selectedRelations, rid)"
            >
            {{ rid }}
          </label>
        </div>
        <label class="expert-label">{{ t('expertConfig.trigger.priority') }}</label>
        <input v-model.number="routePriority" type="number" class="expert-input">
        <button type="button" class="expert-btn" @click="wizardStep = 3">
          {{ t('expertConfig.next') }}
        </button>
      </div>

      <div v-else-if="wizardStep === 3" class="expert-step">
        <label class="expert-label">{{ t('expertConfig.modelPick') }}</label>
        <select v-model="selectedLlmKey" class="expert-input">
          <option v-for="opt in llmSlotKeys" :key="opt.key" :value="opt.key">
            {{ opt.label }} ({{ opt.key }})
          </option>
        </select>
        <label class="expert-check">
          <input v-model="extraAnalyze" type="checkbox">
          {{ t('expertConfig.extraAnalyze') }}
        </label>
        <button type="button" class="expert-btn" @click="wizardStep = 4">
          {{ t('expertConfig.next') }}
        </button>
      </div>

      <div v-else-if="wizardStep === 4" class="expert-step">
        <label class="expert-label">{{ t('expertConfig.facilityStep') }}</label>
        <select v-model="facilityAction" class="expert-input">
          <option value="">
            {{ t('expertConfig.facilityNone') }}
          </option>
          <option v-for="a in EXPERT_FACILITY_ACTIONS" :key="a" :value="a">
            {{ a }}
          </option>
        </select>
        <template v-if="facilityAction === 'slot.personality.adjust'">
          <input v-model="paramTrait" type="text" class="expert-input" placeholder="trait">
          <input v-model.number="paramDelta" type="number" step="0.01" class="expert-input">
        </template>
        <template v-else-if="facilityAction === 'slot.prompt_enhance.apply'">
          <textarea v-model="paramPromptText" class="expert-input" rows="2" />
        </template>
        <template v-else-if="facilityAction === 'slot.memory.inject'">
          <textarea v-model="paramMemoryContent" class="expert-input" rows="2" />
        </template>
        <template v-else-if="facilityAction === 'slot.lora.apply'">
          <input v-model="paramLoraPluginId" type="text" class="expert-input" placeholder="plugin_id">
        </template>
        <button type="button" class="expert-btn" @click="advancePreview">
          {{ t('expertConfig.preview') }}
        </button>
      </div>

      <div v-else class="expert-step">
        <pre class="expert-preview">{{ previewJson }}</pre>
        <button type="button" class="expert-btn primary" @click="confirmSave">
          {{ t('expertConfig.confirmSave') }}
        </button>
      </div>

      <button type="button" class="expert-btn link" @click="wizardOpen = false">
        {{ t('expertConfig.cancel') }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.expert-panel {
  margin-bottom: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
}
.expert-title {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
}
.expert-hint,
.expert-muted {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.expert-file-list {
  margin: 0 0 10px;
  padding-left: 18px;
  font-size: 12px;
}
.expert-actions {
  display: flex;
  gap: 8px;
}
.expert-btn {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-base);
  cursor: pointer;
  font-size: 12px;
}
.expert-btn.primary {
  background: var(--accent, #5b7cfa);
  color: #fff;
  border-color: transparent;
}
.expert-btn.link {
  margin-top: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
}
.expert-wizard {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px dashed var(--border-light);
}
.expert-step {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.expert-label {
  font-size: 12px;
  font-weight: 500;
}
.expert-input {
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  font-size: 12px;
}
.expert-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.expert-chip {
  font-size: 11px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  border: 1px solid var(--border-light);
  border-radius: 4px;
}
.expert-preview {
  max-height: 200px;
  overflow: auto;
  padding: 8px;
  font-size: 11px;
  background: var(--bg-base);
  border-radius: 6px;
}
.expert-check {
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}
</style>
