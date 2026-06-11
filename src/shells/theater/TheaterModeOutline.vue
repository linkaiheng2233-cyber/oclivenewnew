<script setup lang="ts">
import type { TheaterOutline, TheaterSkeleton } from '../../theater/types'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  addOutlineBeat,
  compileOutlineToSkeleton,
  createEmptyOutline,
  skeletonToOutline,
  validateOutline,
} from '../../theater/useTheaterOutlineCompiler'

const props = defineProps<{
  skeleton: TheaterSkeleton | null
}>()

const emit = defineEmits<{
  compiled: [skeleton: TheaterSkeleton]
}>()

const { t, locale } = useI18n()
const outline = ref<TheaterOutline | null>(null)
const compileErrors = ref<string[]>([])
const compileNote = ref<string | null>(null)

const loc = computed(() => (locale.value.startsWith('zh') ? 'zh' : 'en') as 'zh' | 'en')

watch(
  () => props.skeleton,
  (sk) => {
    if (sk && !outline.value) {
      outline.value = skeletonToOutline(sk)
    }
  },
  { immediate: true },
)

function onImportFromSkeleton() {
  if (props.skeleton) {
    outline.value = skeletonToOutline(props.skeleton)
    compileNote.value = null
    compileErrors.value = []
  }
}

function onNewOutline() {
  const sk = props.skeleton
  outline.value = createEmptyOutline(
    sk?.scene_id ?? 'custom',
    sk?.title ?? (loc.value === 'zh' ? '新场景' : 'New scene'),
    sk?.role_a ?? 'theater-breakfast-a',
    sk?.role_b ?? 'theater-breakfast-b',
  )
}

function onAddBeat() {
  if (outline.value) {
    outline.value = addOutlineBeat(outline.value)
  }
}

function onCompile() {
  if (!outline.value) {
    return
  }
  compileErrors.value = validateOutline(outline.value)
  if (compileErrors.value.length > 0) {
    compileNote.value = null
    return
  }
  const compiled = compileOutlineToSkeleton(outline.value)
  compileNote.value = t('theater.outlineCompiled')
  emit('compiled', compiled)
}

function onExportJson() {
  if (!outline.value) {
    return
  }
  const blob = new Blob([JSON.stringify(outline.value, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${outline.value.scene_id}-outline.json`
  a.click()
  URL.revokeObjectURL(url)
}

function speakerLabel(speaker: string): string {
  if (speaker === 'user') {
    return t('theater.speakerUser')
  }
  if (speaker === 'b') {
    return t('theater.roleB')
  }
  return t('theater.roleA')
}
</script>

<template>
  <section class="theater-outline" aria-label="theater outline mode">
    <header class="theater-outline__toolbar">
      <button type="button" class="theater-chip" @click="onImportFromSkeleton">
        {{ t('theater.outlineImport') }}
      </button>
      <button type="button" class="theater-chip theater-chip--ghost" @click="onNewOutline">
        {{ t('theater.outlineNew') }}
      </button>
      <button type="button" class="theater-chip" @click="onAddBeat">
        {{ t('theater.outlineAddBeat') }}
      </button>
      <button type="button" class="theater-chip" @click="onCompile">
        {{ t('theater.outlineCompile') }}
      </button>
      <button type="button" class="theater-chip theater-chip--ghost" @click="onExportJson">
        {{ t('theater.outlineExport') }}
      </button>
    </header>

    <div v-if="outline" class="theater-outline__editor">
      <label class="theater-outline__field">
        <span>{{ t('theater.outlineTitle') }}</span>
        <input v-model="outline.title" type="text" class="theater-outline__input">
      </label>

      <div
        v-for="(beat, index) in outline.beats"
        :key="beat.id"
        class="theater-outline__beat"
      >
        <div class="theater-outline__beat-head">
          <span class="theater-outline__beat-id">{{ beat.id }}</span>
          <select v-model="beat.speaker" class="theater-outline__select">
            <option value="a">
              {{ speakerLabel('a') }}
            </option>
            <option value="b">
              {{ speakerLabel('b') }}
            </option>
            <option value="user">
              {{ speakerLabel('user') }}
            </option>
          </select>
          <span class="theater-outline__index">#{{ index + 1 }}</span>
        </div>
        <textarea
          v-model="beat.summary"
          class="theater-outline__textarea"
          rows="2"
          :placeholder="t('theater.outlineBeatPlaceholder')"
        />
      </div>
    </div>

    <ul v-if="compileErrors.length" class="theater-outline__errors" role="alert">
      <li v-for="err in compileErrors" :key="err">
        {{ err }}
      </li>
    </ul>
    <p v-if="compileNote" class="theater-note">
      {{ compileNote }}
    </p>
  </section>
</template>

<style scoped>
.theater-outline {
  flex: 1;
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
  padding: 0 1rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.theater-outline__toolbar {
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

.theater-chip--ghost {
  border-style: dashed;
  opacity: 0.85;
}

.theater-outline__editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.theater-outline__field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  font-size: 0.85rem;
  opacity: 0.9;
}

.theater-outline__input {
  border-radius: 8px;
  border: 1px solid rgba(255, 220, 180, 0.25);
  background: rgba(0, 0, 0, 0.25);
  color: inherit;
  padding: 0.5rem 0.65rem;
}

.theater-outline__beat {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 0.65rem;
  background: rgba(0, 0, 0, 0.15);
}

.theater-outline__beat-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.4rem;
  font-size: 0.8rem;
  opacity: 0.75;
}

.theater-outline__select {
  border-radius: 6px;
  border: 1px solid rgba(255, 220, 180, 0.2);
  background: rgba(0, 0, 0, 0.3);
  color: inherit;
  padding: 0.2rem 0.4rem;
}

.theater-outline__textarea {
  width: 100%;
  border-radius: 8px;
  border: 1px solid rgba(255, 220, 180, 0.2);
  background: rgba(0, 0, 0, 0.2);
  color: inherit;
  padding: 0.5rem;
  resize: vertical;
  font-family: inherit;
  line-height: 1.5;
}

.theater-outline__errors {
  margin: 0;
  padding-left: 1.2rem;
  color: #faa;
  font-size: 0.85rem;
}

.theater-note {
  text-align: center;
  font-size: 0.8rem;
  opacity: 0.85;
}
</style>
