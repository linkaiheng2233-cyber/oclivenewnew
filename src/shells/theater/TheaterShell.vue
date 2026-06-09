<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  patchTheaterBeats,
  probeOllamaAvailable,
  resolveImpactedBeatIds,
} from '../theater/useTheaterBeatPatch'
import { useTheaterPlayback, useTheaterVariables } from '../theater/useTheaterPlayback'
import type { TheaterPokeChipId, TheaterSkeleton } from '../theater/types'
import { NICKNAME_OPTIONS, THEATER_POKE_CHIP_IDS } from '../theater/types'
import { openPackEditorForRole } from '../utils/openPackEditor'

const { t, locale } = useI18n()

const skeleton = ref<TheaterSkeleton | null>(null)
const loadError = ref<string | null>(null)
const patching = ref(false)
const patchNote = ref<string | null>(null)
const ollamaUp = ref<boolean | null>(null)
const nicknamePickerOpen = ref(false)

const skeletonGetter = () => skeleton.value
const { variables } = useTheaterVariables(skeletonGetter)
const {
  displayedBeats,
  visibleBeats,
  playing,
  finished,
  resetPlayback,
  startPlayback,
} = useTheaterPlayback(skeletonGetter)

const roleAName = computed(() => t('theater.roleA'))
const roleBName = computed(() => t('theater.roleB'))

const pokeChips = computed(() =>
  THEATER_POKE_CHIP_IDS.map((id) => {
    const def = skeleton.value?.variables[id]
    const label = locale.value.startsWith('zh')
      ? def?.label_zh ?? id
      : def?.label_en ?? id
    return { id, label }
  }),
)

onMounted(async () => {
  ollamaUp.value = await probeOllamaAvailable()
  try {
    const res = await fetch('/theater/breakfast/skeleton.json')
    if (!res.ok) {
      throw new Error(`${res.status}`)
    }
    skeleton.value = await res.json() as TheaterSkeleton
  }
  catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
})

async function onPoke(chipId: TheaterPokeChipId) {
  if (!skeleton.value || patching.value) {
    return
  }
  if (chipId === 'nickname_change') {
    nicknamePickerOpen.value = true
    return
  }
  variables.value = { ...variables.value, [chipId]: true }
  await applyPatch(chipId)
}

async function onNicknamePick(nick: string) {
  nicknamePickerOpen.value = false
  variables.value = { ...variables.value, nickname_change: nick }
  await applyPatch('nickname_change')
}

async function applyPatch(varId: TheaterPokeChipId) {
  const sk = skeleton.value
  if (!sk) {
    return
  }
  patching.value = true
  patchNote.value = null
  const beatIds = resolveImpactedBeatIds(sk, varId)
  const loc = locale.value.startsWith('zh') ? 'zh' : 'en'
  const { beats, patched } = await patchTheaterBeats(
    sk,
    displayedBeats.value,
    beatIds,
    variables.value,
    loc,
  )
  resetPlayback(beats)
  if (!patched) {
    patchNote.value = t('theater.patchFallback')
  }
  else {
    patchNote.value = t('theater.patchOk')
  }
  patching.value = false
  startPlayback()
}

async function onEditPersonality() {
  const roleId = skeleton.value?.role_a ?? 'theater-breakfast-a'
  const result = await openPackEditorForRole(roleId)
  if (!result.ok && result.message) {
    patchNote.value = result.message
  }
}
</script>

<template>
  <div class="theater-root" data-shell="theater">
    <header class="theater-header">
      <h1 class="theater-title">
        {{ t('theater.title') }}
      </h1>
      <p class="theater-sub">
        {{ t('theater.subtitle') }}
      </p>
    </header>

    <div v-if="loadError" class="theater-error" role="alert">
      {{ t('theater.loadError', { err: loadError }) }}
    </div>

    <section v-else class="theater-stage" aria-label="breakfast scene">
      <div class="theater-scene-bg">
        <span class="theater-scene-label">{{ skeleton?.title ?? '…' }}</span>
      </div>

      <div class="theater-dialogue">
        <article
          v-for="beat in visibleBeats"
          :key="beat.id"
          class="theater-line"
          :class="[`theater-line--${beat.speaker}`]"
        >
          <span class="theater-line__who">
            {{ beat.speaker === 'a' ? roleAName : roleBName }}
          </span>
          <p class="theater-line__text">
            {{ beat.text }}
          </p>
        </article>
        <p v-if="playing && !patching" class="theater-typing" aria-live="polite">
          …
        </p>
      </div>
    </section>

    <footer class="theater-footer">
      <div class="theater-chips" role="toolbar" :aria-label="t('theater.pokeLabel')">
        <button
          v-for="chip in pokeChips"
          :key="chip.id"
          type="button"
          class="theater-chip"
          :disabled="patching || !skeleton"
          @click="onPoke(chip.id)"
        >
          {{ chip.label }}
        </button>
        <button
          type="button"
          class="theater-chip theater-chip--ghost"
          @click="onEditPersonality"
        >
          {{ t('theater.editPersonality') }}
        </button>
      </div>
      <p v-if="patchNote" class="theater-note">
        {{ patchNote }}
      </p>
      <p v-if="ollamaUp === false" class="theater-note theater-note--muted">
        {{ t('theater.ollamaOff') }}
      </p>
    </footer>

    <div v-if="patching" class="theater-patch-overlay" role="status" aria-live="polite">
      <div class="theater-patch-overlay__card">
        {{ t('theater.patching') }}
      </div>
    </div>

    <div
      v-if="nicknamePickerOpen"
      class="theater-patch-overlay"
      role="dialog"
      aria-modal="true"
      :aria-label="t('theater.nicknameTitle')"
    >
      <div class="theater-patch-overlay__card theater-nick-picker">
        <p>{{ t('theater.nicknameTitle') }}</p>
        <button
          v-for="nick in NICKNAME_OPTIONS"
          :key="nick"
          type="button"
          class="theater-chip"
          @click="onNicknamePick(nick)"
        >
          {{ nick === 'default' ? t('theater.nicknameDefault') : nick }}
        </button>
        <button type="button" class="theater-chip theater-chip--ghost" @click="nicknamePickerOpen = false">
          {{ t('theater.cancel') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.theater-root {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(165deg, #1a1410 0%, #2d2218 45%, #1f1812 100%);
  color: #f5ebe0;
  font-family: system-ui, sans-serif;
}

.theater-header {
  padding: 1.25rem 1.5rem 0.5rem;
  text-align: center;
}

.theater-title {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.theater-sub {
  margin: 0.35rem 0 0;
  opacity: 0.75;
  font-size: 0.9rem;
}

.theater-stage {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 0 1rem 1rem;
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
}

.theater-scene-bg {
  border-radius: 12px;
  min-height: 120px;
  background:
    radial-gradient(ellipse at 30% 20%, rgba(255, 200, 120, 0.25), transparent 55%),
    linear-gradient(180deg, #3d2e22, #2a1f16);
  display: flex;
  align-items: flex-end;
  padding: 1rem;
  margin-bottom: 1rem;
}

.theater-scene-label {
  font-size: 0.85rem;
  opacity: 0.8;
}

.theater-dialogue {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
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
  padding-left: 0.5rem;
}

.theater-footer {
  padding: 1rem 1.5rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.theater-chips {
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
  opacity: 0.85;
}

.theater-note {
  text-align: center;
  font-size: 0.8rem;
  margin: 0.65rem 0 0;
  opacity: 0.85;
}

.theater-note--muted {
  opacity: 0.55;
}

.theater-error {
  margin: 2rem;
  padding: 1rem;
  border: 1px solid #c44;
  border-radius: 8px;
  color: #faa;
}

.theater-patch-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.theater-patch-overlay__card {
  background: #2a2118;
  border: 1px solid rgba(255, 220, 180, 0.2);
  border-radius: 12px;
  padding: 1.25rem 1.5rem;
  min-width: 200px;
  text-align: center;
}

.theater-nick-picker {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
</style>
