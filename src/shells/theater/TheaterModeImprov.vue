<script setup lang="ts">
import type { TheaterSkeleton } from '../../theater/types'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { probeOllamaAvailable } from '../../theater/useTheaterBeatPatch'
import { useTheaterDirector } from '../../theater/useTheaterDirector'
import { generateImprovLine } from '../../theater/useTheaterImprovLine'
import { sessionToOutline, sessionToSkeleton } from '../../theater/useTheaterOutlineCompiler'

const props = defineProps<{
  skeleton: TheaterSkeleton | null
}>()

const emit = defineEmits<{
  frozen: [skeleton: TheaterSkeleton]
}>()

const { t, locale } = useI18n()
const userInput = ref('')
const generating = ref(false)
const statusNote = ref<string | null>(null)
const ollamaUp = ref<boolean | null>(null)

function sceneMeta() {
  return {
    scene_id: props.skeleton?.scene_id ?? 'breakfast',
    title: props.skeleton?.title ?? '',
    role_a: props.skeleton?.role_a ?? 'theater-breakfast-a',
    role_b: props.skeleton?.role_b ?? 'theater-breakfast-b',
  }
}

const {
  turns,
  phase,
  roundCount,
  maxRounds,
  canUserSpeak,
  session,
  resetDirector,
  submitUserLine,
  appendOcLine,
  pendingSpeaker,
  endSession,
} = useTheaterDirector(sceneMeta)

const loc = computed(() => (locale.value.startsWith('zh') ? 'zh' : 'en') as 'zh' | 'en')
const roleAName = computed(() => t('theater.roleA'))
const roleBName = computed(() => t('theater.roleB'))

watch(
  () => props.skeleton?.scene_id,
  () => {
    resetDirector()
    userInput.value = ''
    statusNote.value = null
  },
)

async function initOllamaProbe() {
  ollamaUp.value = await probeOllamaAvailable()
}

onMounted(() => {
  void initOllamaProbe()
})

defineExpose({ initOllamaProbe })

function speakerLabel(speaker: string): string {
  if (speaker === 'user') {
    return t('theater.speakerUser')
  }
  if (speaker === 'b') {
    return roleBName.value
  }
  return roleAName.value
}

async function onSubmitUser() {
  if (!submitUserLine(userInput.value)) {
    return
  }
  userInput.value = ''
  await generatePendingOcLines()
}

async function generatePendingOcLines() {
  while (pendingSpeaker()) {
    generating.value = true
    const speaker = pendingSpeaker()!
    const roleId = speaker === 'a'
      ? props.skeleton?.role_a ?? 'theater-breakfast-a'
      : props.skeleton?.role_b ?? 'theater-breakfast-b'
    const roleLabel = speaker === 'a' ? roleAName.value : roleBName.value

    const { text, source } = await generateImprovLine({
      sceneTitle: props.skeleton?.title ?? '',
      roleId,
      speaker,
      roleLabel,
      priorTurns: turns.value,
      locale: loc.value,
    })

    appendOcLine(speaker, text)
    statusNote.value = source === 'fallback'
      ? t('theater.improvFallback')
      : t('theater.improvLineOk', { who: roleLabel })
    generating.value = false

    if (phase.value === 'ended' || phase.value === 'waiting_user') {
      break
    }
  }
}

function onExportOutline() {
  const blob = new Blob([JSON.stringify(sessionToOutline(session.value), null, 2)], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${session.value.scene_id}-session-outline.json`
  a.click()
  URL.revokeObjectURL(url)
}

function onFreezeSkeleton() {
  const frozen = sessionToSkeleton(session.value)
  emit('frozen', frozen)
  statusNote.value = t('theater.improvFrozen')
  endSession()
}
</script>

<template>
  <section class="theater-improv" aria-label="theater improv mode">
    <p class="theater-improv__hint">
      {{ t('theater.improvHint', { max: maxRounds, round: roundCount }) }}
    </p>

    <div class="theater-dialogue">
      <article
        v-for="turn in turns"
        :key="turn.id"
        class="theater-line"
        :class="[
          turn.speaker === 'user' ? 'theater-line--user' : `theater-line--${turn.speaker}`,
        ]"
      >
        <span class="theater-line__who">
          {{ speakerLabel(turn.speaker) }}
        </span>
        <p class="theater-line__text">
          {{ turn.text }}
        </p>
      </article>
      <p v-if="generating" class="theater-typing" aria-live="polite">
        {{ t('theater.improvGenerating') }}
      </p>
    </div>

    <form class="theater-improv__input-row" @submit.prevent="onSubmitUser">
      <input
        v-model="userInput"
        type="text"
        class="theater-improv__input"
        :placeholder="t('theater.improvInputPlaceholder')"
        :disabled="!canUserSpeak || generating"
      >
      <button
        type="submit"
        class="theater-chip"
        :disabled="!canUserSpeak || generating || !userInput.trim()"
      >
        {{ t('theater.improvSend') }}
      </button>
    </form>

    <div class="theater-improv__actions">
      <button type="button" class="theater-chip theater-chip--ghost" @click="resetDirector">
        {{ t('theater.improvReset') }}
      </button>
      <button
        type="button"
        class="theater-chip"
        :disabled="turns.length === 0"
        @click="onExportOutline"
      >
        {{ t('theater.improvExportOutline') }}
      </button>
      <button
        type="button"
        class="theater-chip"
        :disabled="turns.length === 0"
        @click="onFreezeSkeleton"
      >
        {{ t('theater.improvFreeze') }}
      </button>
    </div>

    <p v-if="statusNote" class="theater-note">
      {{ statusNote }}
    </p>
    <p v-if="ollamaUp === false" class="theater-note theater-note--muted">
      {{ t('theater.ollamaOff') }}
    </p>
    <p v-if="phase === 'ended'" class="theater-note">
      {{ t('theater.improvEnded') }}
    </p>
  </section>
</template>

<style scoped>
.theater-improv {
  flex: 1;
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
  padding: 0 1rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.theater-improv__hint {
  text-align: center;
  font-size: 0.85rem;
  opacity: 0.75;
  margin: 0;
}

.theater-dialogue {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 200px;
}

.theater-line {
  padding: 0.65rem 0.85rem;
  border-radius: 10px;
  max-width: 92%;
}

.theater-line--a {
  align-self: flex-start;
  background: rgba(255, 180, 120, 0.12);
  border: 1px solid rgba(255, 180, 120, 0.2);
}

.theater-line--b {
  align-self: flex-end;
  background: rgba(140, 180, 255, 0.1);
  border: 1px solid rgba(140, 180, 255, 0.18);
}

.theater-line--user {
  align-self: center;
  background: rgba(180, 255, 180, 0.08);
  border: 1px solid rgba(180, 255, 180, 0.15);
  max-width: 96%;
}

.theater-line__who {
  display: block;
  font-size: 0.72rem;
  opacity: 0.65;
  margin-bottom: 0.25rem;
}

.theater-line__text {
  margin: 0;
  line-height: 1.55;
  font-size: 0.95rem;
}

.theater-typing {
  opacity: 0.5;
  margin: 0;
  text-align: center;
}

.theater-improv__input-row {
  display: flex;
  gap: 0.5rem;
  margin-top: auto;
}

.theater-improv__input {
  flex: 1;
  border-radius: 999px;
  border: 1px solid rgba(255, 220, 180, 0.25);
  background: rgba(0, 0, 0, 0.25);
  color: inherit;
  padding: 0.5rem 0.85rem;
}

.theater-improv__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
}

.theater-chip {
  border: 1px solid rgba(255, 220, 180, 0.35);
  background: rgba(255, 220, 180, 0.08);
  color: inherit;
  border-radius: 999px;
  padding: 0.45rem 0.9rem;
  font-size: 0.85rem;
  cursor: pointer;
}

.theater-chip:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.theater-chip--ghost {
  border-style: dashed;
}

.theater-note {
  text-align: center;
  font-size: 0.8rem;
  opacity: 0.85;
}

.theater-note--muted {
  opacity: 0.55;
}
</style>
