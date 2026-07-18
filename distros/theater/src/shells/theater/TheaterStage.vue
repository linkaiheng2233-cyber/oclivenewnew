<script setup lang="ts">
import type { ScriptLine, TheaterCast, TheaterCastSide, TheaterStageState } from '../../composables/theater/theaterLogic'
import UiResizeHandle from '@oclive/shared/components/ui/UiResizeHandle.vue'
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { buildCastRoster, resolveCastPortraitState, rosterBySide } from '../../composables/theater/theaterPortrait'
import { setTheaterPortraitWidth, THEATER_PORTRAIT_DEFAULTS } from '../../composables/useTheaterPortraitLayout'
import TheaterCastColumn from './TheaterCastColumn.vue'
import TheaterScriptLine from './TheaterScriptLine.vue'

export interface TheaterCastInfo {
  roleId: string
  name: string
  side?: TheaterCastSide
}

const props = defineProps<{
  lines: ScriptLine[]
  state: TheaterStageState
  loadError: string | null
  cast: {
    a: TheaterCastInfo
    b: TheaterCastInfo
  } | null
  /** Poke preview / patching: which cast slot is primarily affected by the event. */
  eventHighlightCast?: TheaterCast | null
}>()

const { t } = useI18n()
const scrollRef = ref<HTMLElement | null>(null)
let portraitWidth = 0

const columns = computed(() => {
  if (!props.cast)
    return { left: [], right: [] }
  const roster = buildCastRoster(props.cast)
  const portraitMap = resolveCastPortraitState(props.lines)
  const grouped = rosterBySide(roster, portraitMap)
  const highlight = props.eventHighlightCast
  const mark = (entries: typeof grouped.left) =>
    entries.map(entry => ({
      ...entry,
      eventAffected: highlight != null && entry.castId === highlight,
    }))
  return { left: mark(grouped.left), right: mark(grouped.right) }
})

watch(
  () => props.lines.length,
  async () => {
    await nextTick()
    const el = scrollRef.value
    if (el)
      el.scrollTop = el.scrollHeight
  },
)

function onPortraitResizeStart() {
  portraitWidth = Number.parseInt(
    getComputedStyle(document.documentElement).getPropertyValue('--theater-portrait-w'),
    10,
  ) || THEATER_PORTRAIT_DEFAULTS.width
}

function onPortraitResizeLeft(deltaX: number) {
  portraitWidth = setTheaterPortraitWidth(portraitWidth + deltaX)
}

function onPortraitResizeRight(deltaX: number) {
  portraitWidth = setTheaterPortraitWidth(portraitWidth - deltaX)
}
</script>

<template>
  <section
    class="theater-stage"
    aria-live="polite"
    :aria-busy="state === 'playing' || state === 'patching'"
  >
    <TheaterCastColumn
      v-if="cast"
      side="left"
      :casts="columns.left"
    />

    <UiResizeHandle
      v-if="cast"
      edge="left"
      :aria-label="t('theater.stage.resizePortrait')"
      @resize-start="onPortraitResizeStart"
      @resize="onPortraitResizeLeft"
    />

    <div ref="scrollRef" class="theater-stage__scroll">
      <div class="theater-stage__inner">
        <p v-if="loadError" class="theater-stage__error" role="alert">
          {{ loadError }}
        </p>
        <TheaterScriptLine
          v-for="line in lines"
          :key="line.id"
          :line="line"
        />
        <slot name="variant-backdrop" />
      </div>
    </div>

    <UiResizeHandle
      v-if="cast"
      edge="right"
      :aria-label="t('theater.stage.resizePortrait')"
      @resize-start="onPortraitResizeStart"
      @resize="onPortraitResizeRight"
    />

    <TheaterCastColumn
      v-if="cast"
      side="right"
      :casts="columns.right"
    />
  </section>
</template>

<style scoped>
.theater-stage {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
  padding: 0 var(--tool-space-1, 4px);
}

.theater-stage__scroll {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.theater-stage__inner {
  position: relative;
  max-width: var(--theater-stage-max-w, 720px);
  margin: 0 auto;
  padding: var(--tool-space-4, 16px) var(--tool-space-2, 8px);
}

.theater-stage__error {
  margin: 0 0 var(--tool-space-4, 16px);
  color: #c0392b;
  font-size: var(--tool-fs-md, 13px);
}
</style>
