<script setup lang="ts">
import type { TheaterMode, TheaterSceneMeta, TheaterSkeleton } from '../../theater/types'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import StartupWarningsBanner from '../../components/StartupWarningsBanner.vue'
import { loadSceneIndex, loadTheaterSkeleton, resolveSceneTitle } from '../../theater/sceneRegistry'
import { THEATER_MODES } from '../../theater/types'
import TheaterModeImprov from './TheaterModeImprov.vue'
import TheaterModeOutline from './TheaterModeOutline.vue'
import TheaterModeTweak from './TheaterModeTweak.vue'

const { t, locale } = useI18n()

const scenes = ref<TheaterSceneMeta[]>([])
const activeSceneId = ref('breakfast')
const skeleton = ref<TheaterSkeleton | null>(null)
const compiledSkeleton = ref<TheaterSkeleton | null>(null)
const loadError = ref<string | null>(null)
const mode = ref<TheaterMode>('tweak')

const tweakRef = ref<InstanceType<typeof TheaterModeTweak> | null>(null)
const improvRef = ref<InstanceType<typeof TheaterModeImprov> | null>(null)

const loc = computed(() => (locale.value.startsWith('zh') ? 'zh' : 'en') as 'zh' | 'en')

const activeSkeleton = computed(() => compiledSkeleton.value ?? skeleton.value)

const modeTabs = computed(() =>
  THEATER_MODES.map(m => ({
    id: m,
    label: t(`theater.mode.${m}`),
    enabled: m === 'tweak' || m === 'outline' || m === 'improv',
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
    if (index.scenes.length > 0) {
      activeSceneId.value = index.scenes[0].scene_id
    }
  }
  catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
    return
  }
  await loadScene(activeSceneId.value)
  await tweakRef.value?.initOllamaProbe?.()
})

watch(activeSceneId, async (id) => {
  await loadScene(id)
})

watch(mode, async (m) => {
  if (m === 'tweak') {
    await tweakRef.value?.initOllamaProbe?.()
  }
  if (m === 'improv') {
    await improvRef.value?.initOllamaProbe?.()
  }
})

function onOutlineCompiled(next: TheaterSkeleton) {
  compiledSkeleton.value = next
  mode.value = 'tweak'
}

function onImprovFrozen(next: TheaterSkeleton) {
  compiledSkeleton.value = next
  mode.value = 'tweak'
}

function sceneLabel(scene: TheaterSceneMeta): string {
  return resolveSceneTitle(scene, loc.value)
}
</script>

<template>
  <div class="theater-root" data-shell="theater">
    <StartupWarningsBanner />
    <header class="theater-header">
      <h1 class="theater-title">
        {{ t('theater.title') }}
      </h1>
      <p class="theater-sub">
        {{ t('theater.subtitle') }}
      </p>

      <nav class="theater-mode-tabs" role="tablist" :aria-label="t('theater.modeTabs')">
        <button
          v-for="tab in modeTabs"
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
    </header>

    <div v-if="loadError" class="theater-error" role="alert">
      {{ t('theater.loadError', { err: loadError }) }}
    </div>

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
  padding: 1.25rem 1.5rem 0.75rem;
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

.theater-mode-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1rem;
}

.theater-mode-tab {
  border: 1px solid rgba(255, 220, 180, 0.3);
  background: rgba(255, 220, 180, 0.06);
  color: inherit;
  border-radius: 999px;
  padding: 0.4rem 0.85rem;
  font-size: 0.82rem;
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
  font-size: 0.65rem;
  opacity: 0.7;
}

.theater-scene-picker {
  margin-top: 0.75rem;
  font-size: 0.85rem;
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

.theater-error {
  margin: 2rem;
  padding: 1rem;
  border: 1px solid #c44;
  border-radius: 8px;
  color: #faa;
}
</style>
