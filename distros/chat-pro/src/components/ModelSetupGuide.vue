<script setup lang="ts">
import { getLlmUserSettings } from '@oclive/shared/api/llmSettings'
import ErrorNotice from '@oclive/shared/components/ErrorNotice.vue'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import MumuFeatureIcon from './MumuFeatureIcon.vue'

const props = defineProps<{ roleId: string, ready: boolean }>()
const emit = defineEmits<{ configure: [provider: 'local' | 'cloud'] }>()
const { t } = useI18n()
const ui = useUiStore()
const pending = ref<'local' | 'cloud' | null>(null)
const checking = ref(false)
const error = ref<unknown>(null)
const existing = ref(false)
let generation = 0

async function checkExisting(): Promise<void> {
  const attempt = ++generation
  const roleId = props.roleId
  error.value = null
  existing.value = false
  checking.value = false
  if (!props.ready || !roleId || ui.modelSetupDismissed || pending.value)
    return
  checking.value = true
  try {
    const settings = await getLlmUserSettings(roleId)
    if (attempt !== generation || pending.value)
      return
    // A pack's default model name does not mean weights are installed.
    existing.value = settings.provider === 'cloud'
      ? Boolean(settings.remoteUrl.trim() && settings.remoteModel.trim() && (settings.remoteTokenConfigured || settings.remoteTokenEnvActive))
      : Boolean(settings.localModelPath.trim() || settings.sessionOllamaModel?.trim() || settings.performanceReady)
  }
  catch (failure) {
    if (attempt === generation)
      error.value = failure
  }
  finally {
    if (attempt === generation)
      checking.value = false
  }
}

function choose(provider: 'local' | 'cloud'): void {
  if (ui.modelSetupDismissed) {
    pending.value = null
    return
  }
  generation += 1
  checking.value = false
  error.value = null
  if (props.ready && props.roleId) {
    pending.value = null
    emit('configure', provider)
  }
  else {
    pending.value = provider
  }
}

watch(() => [props.roleId, props.ready] as const, () => {
  if (pending.value && props.ready && props.roleId)
    choose(pending.value)
  else
    void checkExisting()
}, { immediate: true })
watch(() => ui.modelSetupDismissed, () => {
  generation += 1
  pending.value = null
  checking.value = false
})
onBeforeUnmount(() => {
  generation += 1
})
</script>

<template>
  <section v-if="!ui.modelSetupDismissed && !existing" class="model-setup" aria-labelledby="model-setup-title">
    <div class="setup-heading">
      <MumuFeatureIcon name="model" />
      <h2 id="model-setup-title">
        {{ t('workspace.setupTitle') }}
      </h2>
      <button type="button" class="setup-later" @click="ui.modelSetupDismissed = true">
        {{ t('workspace.setupLater') }}
      </button>
    </div>
    <p>{{ t('workspace.setupLead') }}</p>
    <div class="setup-options">
      <button type="button" :aria-pressed="pending === 'local'" @click="choose('local')">
        <strong>{{ t('workspace.setupLocal') }}</strong><span>{{ t('workspace.setupLocalHint') }}</span>
      </button>
      <button type="button" :aria-pressed="pending === 'cloud'" @click="choose('cloud')">
        <strong>{{ t('workspace.setupCloud') }}</strong><span>{{ t('workspace.setupCloudHint') }}</span>
      </button>
    </div>
    <p v-if="pending" role="status">
      {{ t('workspace.setupSelectRole') }}
    </p>
    <p v-else-if="checking" role="status">
      {{ t('workspace.setupChecking') }}
    </p>
    <ErrorNotice v-if="error" :error="error" :summary="t('workspace.setupCheckFailed')">
      <button type="button" @click="checkExisting">
        {{ t('workspace.retry') }}
      </button>
    </ErrorNotice>
  </section>
</template>

<style scoped>
.model-setup { flex-shrink:0; margin:12px 22px; padding:16px; border:1px solid var(--border-light); border-radius:12px; color:var(--text-primary); background:var(--bg-elevated,var(--bg-primary)); }
.setup-heading { display:flex; align-items:center; justify-content:space-between; gap:12px; }
.setup-heading h2 { flex:1;min-width:0;line-height:1.5; }
h2 { margin:0; font-size:16px; }
p { margin:8px 0 0; color:var(--text-secondary); font-size:12px; line-height:1.7; }
button { font:inherit; cursor:pointer; }
.setup-later { flex-shrink:0; border:0; background:transparent; color:var(--text-secondary); font-size:12px; text-decoration:underline; }
.setup-options { display:flex; gap:10px; margin-top:12px; }
.setup-options button { flex:1; min-width:0; display:flex; flex-direction:column; gap:6px; align-items:flex-start; padding:12px; text-align:left; border:1px solid var(--border-light); border-radius:8px; background:var(--bg-primary); color:var(--text-primary); }
.setup-options button:hover,.setup-options button[aria-pressed=true] { border-color:var(--accent); }
.setup-options strong { font-size:13px; }
.setup-options span { color:var(--text-secondary); font-size:11px; line-height:1.6; }
button:focus-visible { outline:2px solid var(--accent); outline-offset:3px; }
@media(max-width:600px) { .model-setup { margin:8px 12px;padding:12px }.setup-options { gap:8px }.setup-options button { padding:9px } }
</style>
