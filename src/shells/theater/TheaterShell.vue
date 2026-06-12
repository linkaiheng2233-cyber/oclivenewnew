<script setup lang="ts">
import type { TheaterMode, TheaterSceneMeta, TheaterSkeleton } from '../../theater/types'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { loadSceneIndex, loadTheaterSkeleton, prefetchTheaterBootstrap, resolveSceneTitle } from '../../theater/sceneRegistry'
import { THEATER_MODES } from '../../theater/types'
import { openPackEditorForRole } from '../../utils/openPackEditor'
import TheaterModeImprov from './TheaterModeImprov.vue'
import TheaterModeOutline from './TheaterModeOutline.vue'
import TheaterModeTweak from './TheaterModeTweak.vue'
import TheaterStagePanel from './TheaterStagePanel.vue'

const { t, locale } = useI18n()

prefetchTheaterBootstrap()

const scenes = ref<TheaterSceneMeta[]>([])
const activeSceneId = ref('breakfast')
const skeleton = ref<TheaterSkeleton | null>(null)
const compiledSkeleton = ref<TheaterSkeleton | null>(null)
const loadError = ref<string | null>(null)
const mode = ref<TheaterMode>('tweak')
const advancedOpen = ref(false)
const personalityNote = ref<string | null>(null)

const tweakRef = ref<InstanceType<typeof TheaterModeTweak> | null>(null)
const improvRef = ref<InstanceType<typeof TheaterModeImprov> | null>(null)

const loc = computed(() => (locale.value.startsWith('zh') ? 'zh' : 'en') as 'zh' | 'en')

const activeSkeleton = computed(() => compiledSkeleton.value ?? skeleton.value)

const advancedModeTabs = computed(() =>
  THEATER_MODES.filter(m => m !== 'tweak').map(m => ({
    id: m,
    label: t(`theater.mode.${m}`),
    enabled: m === 'outline' || m === 'improv',
  })),
)

async function loadScene(sceneId: string) {
  loadError.value = null
  compiledSkeleton.value = null
  try {
    skeleton.value = await loadTheaterSkeleton(sceneId)
  }
  catch (e) {
    skeleton.value = null
    loadError.value = e instanceof Error ? e.message : String(e)
  }
}

onMounted(async () => {
  try {
    const index = await loadSceneIndex()
    scenes.value = index.scenes
    const sceneId = index.scenes[0]?.scene_id ?? activeSceneId.value
    activeSceneId.value = sceneId
    skeleton.value = await loadTheaterSkeleton(sceneId)
  }
  catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
})

watch(activeSceneId, async (id) => {
  if (skeleton.value?.scene_id === id) {
    return
  }
  await loadScene(id)
})

watch(mode, async (m) => {
  if (m === 'improv') {
    await improvRef.value?.initOllamaProbe?.()
  }
})

function onOutlineCompiled(next: TheaterSkeleton) {
  compiledSkeleton.value = next
  mode.value = 'tweak'
  advancedOpen.value = false
}

function onImprovFrozen(next: TheaterSkeleton) {
  compiledSkeleton.value = next
  mode.value = 'tweak'
  advancedOpen.value = false
}

function sceneLabel(scene: TheaterSceneMeta): string {
  return resolveSceneTitle(scene, loc.value)
}

async function onEditPersonality() {
  const roleId = activeSkeleton.value?.role_a ?? 'theater-breakfast-a'
  const result = await openPackEditorForRole(roleId)
  if (!result.ok && result.message) {
    personalityNote.value = result.message
  }
}
</script>

<template>
  <div class="theater-root" data-shell="theater">
    <header class="theater-header">
      <h1 class="theater-title">
        {{ t('theater.title') }}
      </h1>
      <p class="theater-action-hint">
        {{ t('theater.actionHint') }}
      </p>

      <button
        type="button"
        class="theater-advanced-toggle"
        :aria-expanded="advancedOpen"
        @click="advancedOpen = !advancedOpen"
      >
        {{ advancedOpen ? t('theater.advancedCollapse') : t('theater.advancedModes') }}
      </button>

      <div v-if="advancedOpen" class="theater-advanced-panel">
        <nav class="theater-mode-tabs" role="tablist" :aria-label="t('theater.modeTabs')">
          <button
            type="button"
            role="tab"
            class="theater-mode-tab theater-mode-tab--active"
            :aria-selected="mode === 'tweak'"
            @click="mode = 'tweak'"
          >
            {{ t('theater.mode.tweak') }}
          </button>
          <button
            v-for="tab in advancedModeTabs"
            :key="tab.id"
            type="button"
            role="tab"
            class="theater-mode-tab"
            :class="{ 'theater-mode-tab--active': mode === tab.id }"
            :aria-selected="mode === tab.id"
            :disabled="!tab.enabled"
            @click="tab.enabled && (mode = tab.id)"
          >
            {{ tab.label }}
            <span v-if="!tab.enabled" class="theater-mode-tab__soon">{{ t('theater.modeComingSoon') }}</span>
          </button>
        </nav>

        <div class="theater-advanced-actions">
          <button type="button" class="theater-chip theater-chip--ghost" @click="onEditPersonality">
            {{ t('theater.editPersonality') }}
          </button>
        </div>

        <div v-if="scenes.length > 1" class="theater-scene-picker">
          <label>
            <span class="theater-scene-picker__label">{{ t('theater.scenePicker') }}</span>
            <select v-model="activeSceneId" class="theater-scene-picker__select">
              <option v-for="scene in scenes" :key="scene.scene_id" :value="scene.scene_id">
                {{ sceneLabel(scene) }}
              </option>
            </select>
          </label>
        </div>

        <p v-if="personalityNote" class="theater-note">
          {{ personalityNote }}
        </p>
      </div>
    </header>

    <div v-if="loadError" class="theater-error" role="alert">
      {{ t('theater.loadError', { err: loadError }) }}
    </div>

    <TheaterStagePanel
      v-if="!loadError && activeSkeleton"
      :role-id="activeSkeleton.role_a"
      :role-name="t('theater.roleA')"
    />

    <TheaterModeTweak
      v-else-if="mode === 'tweak'"
      ref="tweakRef"
      :skeleton="activeSkeleton"
    />
    <TheaterModeOutline
      v-else-if="mode === 'outline'"
      :skeleton="activeSkeleton"
      @compiled="onOutlineCompiled"
    />
    <TheaterModeImprov
      v-else-if="mode === 'improv'"
      ref="improvRef"
      :skeleton="activeSkeleton"
      @frozen="onImprovFrozen"
    />
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
  padding: 0.75rem 1.25rem 0.5rem;
  text-align: center;
}

.theater-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.theater-action-hint {
  margin: 0.25rem 0 0;
  opacity: 0.7;
  font-size: 0.82rem;
}

.theater-advanced-toggle {
  margin-top: 0.5rem;
  border: none;
  background: transparent;
  color: inherit;
  opacity: 0.55;
  font-size: 0.78rem;
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.theater-advanced-toggle:hover {
  opacity: 0.85;
}

.theater-advanced-panel {
  margin-top: 0.65rem;
}

.theater-mode-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
}

.theater-mode-tab {
  border: 1px solid rgba(255, 220, 180, 0.3);
  background: rgba(255, 220, 180, 0.06);
  color: inherit;
  border-radius: 999px;
  padding: 0.35rem 0.75rem;
  font-size: 0.78rem;
  cursor: pointer;
}

.theater-mode-tab--active {
  background: rgba(255, 220, 180, 0.18);
  border-color: rgba(255, 220, 180, 0.5);
}

.theater-mode-tab:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.theater-mode-tab__soon {
  display: block;
  font-size: 0.6rem;
  opacity: 0.7;
}

.theater-advanced-actions {
  display: flex;
  justify-content: center;
  margin-top: 0.5rem;
}

.theater-chip {
  border: 1px solid rgba(255, 220, 180, 0.35);
  background: rgba(255, 220, 180, 0.08);
  color: inherit;
  border-radius: 999px;
  padding: 0.4rem 0.85rem;
  font-size: 0.8rem;
  cursor: pointer;
}

.theater-chip--ghost {
  border-style: dashed;
  opacity: 0.85;
}

.theater-scene-picker {
  margin-top: 0.65rem;
  font-size: 0.82rem;
}

.theater-scene-picker__label {
  margin-right: 0.5rem;
  opacity: 0.75;
}

.theater-scene-picker__select {
  border-radius: 8px;
  border: 1px solid rgba(255, 220, 180, 0.25);
  background: rgba(0, 0, 0, 0.25);
  color: inherit;
  padding: 0.35rem 0.5rem;
}

.theater-note {
  text-align: center;
  font-size: 0.78rem;
  margin: 0.5rem 0 0;
  opacity: 0.75;
}

.theater-error {
  margin: 2rem;
  padding: 1rem;
  border: 1px solid #c44;
  border-radius: 8px;
  color: #faa;
}
</style>
