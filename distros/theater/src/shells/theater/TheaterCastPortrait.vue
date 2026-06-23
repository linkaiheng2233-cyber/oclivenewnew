<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { readRoleAssetBytes, resolveRoleAssetPath } from '@oclive/shared/api'
import {
  emotionToAssetFilename,
  emotionToEmoji,
} from '@oclive/shared/utils/emotion-assets'

const props = defineProps<{
  roleId: string
  name: string
  emotion: string
  side: 'left' | 'right'
  active?: boolean
  /** Event preview / patching: primary subject of the poke chip. */
  eventAffected?: boolean
}>()

const portraitSrc = ref<string | null>(null)
const portraitBlobUrl = ref<string | null>(null)
const portraitLoadFailed = ref(false)
let portraitGeneration = 0

const { t } = useI18n()

const emotionKey = computed(() => props.emotion.trim().toLowerCase() || 'neutral')
const fallbackEmoji = computed(() => emotionToEmoji[emotionKey.value] ?? '😐')

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

function revokeBlob(): void {
  if (portraitBlobUrl.value) {
    URL.revokeObjectURL(portraitBlobUrl.value)
    portraitBlobUrl.value = null
  }
}

function emotionAssetCandidates(key: string): string[] {
  const primary = emotionToAssetFilename(key)
  const out = new Set<string>()
  const pushExpanded = (file: string) => {
    const idx = file.lastIndexOf('.')
    const base = idx >= 0 ? file.slice(0, idx) : file
    for (const ext of ['png', 'jpg', 'jpeg', 'webp'])
      out.add(`${base}.${ext}`)
  }
  pushExpanded(primary)
  if (key === 'neutral') {
    pushExpanded('neutral.png')
  }
  pushExpanded('normal.png')
  pushExpanded('neutral.png')
  return Array.from(out)
}

async function tryLoadBytes(roleId: string, rel: string, filename: string, gen: number): Promise<boolean> {
  let path: string | null
  try {
    path = await resolveRoleAssetPath(roleId, rel)
  }
  catch {
    return false
  }
  if (!path)
    return false

  if (isTauri()) {
    try {
      const bytes = await readRoleAssetBytes(roleId, rel)
      if (gen !== portraitGeneration || !bytes)
        return false
      const mime = filename.endsWith('.webp')
        ? 'image/webp'
        : filename.endsWith('.jpg') || filename.endsWith('.jpeg')
          ? 'image/jpeg'
          : 'image/png'
      const blob = new Blob([new Uint8Array(bytes)], { type: mime })
      const url = URL.createObjectURL(blob)
      portraitBlobUrl.value = url
      portraitSrc.value = url
      return true
    }
    catch {
      // fall through
    }
  }

  try {
    if (gen !== portraitGeneration)
      return false
    portraitSrc.value = convertFileSrc(path)
    return true
  }
  catch {
    return false
  }
}

async function refreshPortrait(): Promise<void> {
  const gen = ++portraitGeneration
  portraitLoadFailed.value = false
  revokeBlob()
  portraitSrc.value = null

  for (const filename of emotionAssetCandidates(emotionKey.value)) {
    if (gen !== portraitGeneration)
      return
    const rel = `assets/images/${filename}`
    if (await tryLoadBytes(props.roleId, rel, filename, gen))
      return
  }

  if (gen === portraitGeneration)
    portraitLoadFailed.value = true
}

watch(
  () => [props.roleId, props.emotion] as const,
  () => { void refreshPortrait() },
  { immediate: true },
)

onBeforeUnmount(() => {
  revokeBlob()
})
</script>

<template>
  <aside
    class="cast-portrait"
    :class="[
      side === 'left' ? 'cast-portrait--a' : 'cast-portrait--b',
      { 'cast-portrait--active': active },
      { 'cast-portrait--event-affected': eventAffected },
    ]"
    :aria-label="eventAffected ? t('theater.stage.eventAffected', { name }) : name"
  >
    <div class="cast-portrait__frame">
      <img
        v-if="portraitSrc && !portraitLoadFailed"
        :key="`${roleId}-${emotionKey}-${portraitSrc}`"
        class="cast-portrait__img"
        :src="portraitSrc"
        alt=""
      >
      <span v-else class="cast-portrait__emoji" aria-hidden="true">{{ fallbackEmoji }}</span>
    </div>
    <p class="cast-portrait__name">{{ name }}</p>
  </aside>
</template>

<style scoped>
.cast-portrait {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  width: var(--theater-portrait-w, 96px);
  padding: var(--tool-space-2, 8px) var(--tool-space-1, 4px);
  opacity: 0.72;
  transition: opacity 0.28s ease, transform 0.28s ease;
}

.cast-portrait--active {
  opacity: 1;
  transform: translateY(-2px);
}

.cast-portrait--event-affected {
  opacity: 1;
}

.cast-portrait--event-affected .cast-portrait__frame {
  outline: 2px dashed color-mix(in srgb, var(--tool-accent) 72%, transparent);
  outline-offset: 3px;
}

.cast-portrait--a.cast-portrait--event-affected .cast-portrait__frame {
  box-shadow:
    0 0 0 3px color-mix(in srgb, var(--theater-cast-a) 55%, transparent),
    0 0 0 6px color-mix(in srgb, var(--theater-cast-a) 18%, transparent),
    0 8px 24px color-mix(in srgb, var(--theater-cast-a) 32%, transparent);
}

.cast-portrait--b.cast-portrait--event-affected .cast-portrait__frame {
  box-shadow:
    0 0 0 3px color-mix(in srgb, var(--theater-cast-b) 55%, transparent),
    0 0 0 6px color-mix(in srgb, var(--theater-cast-b) 18%, transparent),
    0 8px 24px color-mix(in srgb, var(--theater-cast-b) 32%, transparent);
}

.cast-portrait--a.cast-portrait--active .cast-portrait__frame {
  box-shadow: 0 0 0 2px var(--theater-cast-a), 0 6px 20px color-mix(in srgb, var(--theater-cast-a) 28%, transparent);
}

.cast-portrait--b.cast-portrait--active .cast-portrait__frame {
  box-shadow: 0 0 0 2px var(--theater-cast-b), 0 6px 20px color-mix(in srgb, var(--theater-cast-b) 28%, transparent);
}

.cast-portrait__frame {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  width: 100%;
  min-height: calc(var(--theater-portrait-max-h, 200px) * 0.44);
  max-height: var(--theater-portrait-max-h, 200px);
  border-radius: var(--tool-radius-lg, 12px);
  background: color-mix(in srgb, var(--tool-elevated) 92%, var(--tool-accent) 8%);
  overflow: hidden;
  transition: box-shadow 0.28s ease;
}

.cast-portrait__img {
  display: block;
  max-width: 100%;
  max-height: var(--theater-portrait-max-h, 200px);
  width: auto;
  height: auto;
  object-fit: contain;
  object-position: center bottom;
  animation: castPortraitIn 0.32s ease both;
}

.cast-portrait__emoji {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: calc(var(--theater-portrait-max-h, 200px) * 0.36);
  font-size: clamp(1.6rem, calc(var(--theater-portrait-w, 96px) * 0.28), 3rem);
  line-height: 1;
  animation: castPortraitIn 0.32s ease both;
}

.cast-portrait__name {
  margin: var(--tool-space-1, 4px) 0 0;
  max-width: 100%;
  font-size: var(--tool-fs-sm, 12px);
  font-weight: 600;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cast-portrait--a .cast-portrait__name {
  color: var(--theater-cast-a);
}

.cast-portrait--b .cast-portrait__name {
  color: var(--theater-cast-b);
}

@keyframes castPortraitIn {
  from {
    opacity: 0;
    transform: scale(0.94);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .cast-portrait,
  .cast-portrait__img,
  .cast-portrait__emoji {
    animation: none;
    transition: none;
  }
}
</style>
