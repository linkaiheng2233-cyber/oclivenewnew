<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { readRoleAssetBytes, resolveRoleAssetPath } from '@oclive/shared/api'
import {
  emotionToAssetFilename,
  emotionToEmoji,
} from '@oclive/shared/utils/emotion-assets'

const props = withDefaults(
  defineProps<{
    roleId: string
    name: string
    emotion: string
    /** Catalog / directive relative asset path (e.g. assets/images/happy.webp) */
    portraitAssetRelPath?: string | null
    layout?: 'stack' | 'sidebar'
  }>(),
  { layout: 'stack' },
)

const { t, te } = useI18n()

const portraitSrc = ref<string | null>(null)
const portraitBlobUrl = ref<string | null>(null)
const portraitLoadFailed = ref(false)
let portraitGeneration = 0

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

function revokeBlob(): void {
  if (portraitBlobUrl.value) {
    URL.revokeObjectURL(portraitBlobUrl.value)
    portraitBlobUrl.value = null
  }
}

function emotionKey(): string {
  return props.emotion.trim().toLowerCase() || 'neutral'
}

const emotionDisplayLabel = computed(() => {
  const key = emotionKey()
  const path = `emotionUi.${key}`
  return te(path) ? t(path) : props.emotion.trim() || key
})

function emotionAssetCandidates(key: string): string[] {
  const primary = emotionToAssetFilename(key)
  const out = new Set<string>()

  const pushExpanded = (file: string) => {
    const idx = file.lastIndexOf('.')
    const base = idx >= 0 ? file.slice(0, idx) : file
    for (const ext of ['png', 'jpg', 'jpeg', 'webp']) {
      out.add(`${base}.${ext}`)
    }
  }

  pushExpanded(primary)
  // Common compatibility: role packs may use alternate filenames (e.g. neutral.png)
  if (key === 'neutral') {
    pushExpanded('neutral.png')
  }
  if (key.startsWith('disgust')) {
    pushExpanded('disgust_light.png')
    pushExpanded('disgust_mid.png')
    pushExpanded('disgust_heavy.png')
  }
  // Final fallbacks
  pushExpanded('normal.png')
  pushExpanded('neutral.png')
  return Array.from(out)
}

async function tryLoadRelativeAsset(rel: string, gen: number): Promise<void> {
  let path: string | null
  try {
    path = await resolveRoleAssetPath(props.roleId, rel)
  }
  catch (e) {
    if (gen !== portraitGeneration)
      return
    console.warn('[CharacterInfo] catalog asset resolve failed', e)
    return
  }
  if (!path)
    return

  const filename = rel.split('/').pop() ?? rel
  if (isTauri()) {
    try {
      const bytes = await readRoleAssetBytes(props.roleId, rel)
      if (gen !== portraitGeneration)
        return
      if (!bytes)
        return
      const mime = filename.endsWith('.webp')
        ? 'image/webp'
        : filename.endsWith('.jpg') || filename.endsWith('.jpeg')
          ? 'image/jpeg'
          : filename.endsWith('.gif')
            ? 'image/gif'
            : 'image/png'
      const blob = new Blob([new Uint8Array(bytes)], { type: mime })
      const url = URL.createObjectURL(blob)
      portraitBlobUrl.value = url
      portraitSrc.value = url
      return
    }
    catch (e) {
      console.warn('[CharacterInfo] catalog readRoleAssetBytes failed', e)
    }
  }

  try {
    if (gen !== portraitGeneration)
      return
    portraitSrc.value = convertFileSrc(path)
  }
  catch (e) {
    console.warn('[CharacterInfo] catalog convertFileSrc failed', e)
  }
}

async function refreshPortrait(): Promise<void> {
  const gen = ++portraitGeneration
  portraitLoadFailed.value = false
  revokeBlob()
  portraitSrc.value = null

  const catalogRel = props.portraitAssetRelPath?.trim()
  if (catalogRel) {
    await tryLoadRelativeAsset(catalogRel, gen)
    if (gen !== portraitGeneration)
      return
    if (portraitSrc.value)
      return
  }

  const key = emotionKey()
  let loaded = false
  for (const filename of emotionAssetCandidates(key)) {
    if (gen !== portraitGeneration)
      return
    const rel = `assets/images/${filename}`
    let path: string | null
    try {
      path = await resolveRoleAssetPath(props.roleId, rel)
    }
    catch (e) {
      if (gen !== portraitGeneration)
        return
      console.warn('[CharacterInfo] find_role_asset_path failed', e)
      portraitLoadFailed.value = true
      return
    }
    if (!path)
      continue

    /* Prefer Tauri command + Blob: avoids custom asset protocol and net::ERR_CONNECTION_REFUSED */
    if (isTauri()) {
      try {
        const bytes = await readRoleAssetBytes(props.roleId, rel)
        if (gen !== portraitGeneration)
          return
        if (!bytes)
          continue
        const mime = filename.endsWith('.webp')
          ? 'image/webp'
          : filename.endsWith('.jpg') || filename.endsWith('.jpeg')
            ? 'image/jpeg'
            : filename.endsWith('.gif')
              ? 'image/gif'
              : 'image/png'
        const blob = new Blob([new Uint8Array(bytes)], { type: mime })
        const url = URL.createObjectURL(blob)
        portraitBlobUrl.value = url
        portraitSrc.value = url
        loaded = true
        break
      }
      catch (e) {
        console.warn(
          '[CharacterInfo] readRoleAssetBytes failed, fallback convertFileSrc',
          e,
        )
      }
    }

    try {
      if (gen !== portraitGeneration)
        return
      portraitSrc.value = convertFileSrc(path)
      loaded = true
      break
    }
    catch (e) {
      console.warn('[CharacterInfo] convertFileSrc failed', e)
    }
  }

  if (gen !== portraitGeneration)
    return
  if (!loaded) {
    portraitLoadFailed.value = true
  }
}

function onPortraitError(): void {
  portraitLoadFailed.value = true
}

watch(
  () => [props.roleId, props.emotion, props.portraitAssetRelPath] as const,
  () => {
    void refreshPortrait()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  revokeBlob()
})
</script>

<template>
  <!-- Match oclive-new .main-content: hero image + name + emotion (affinity in bottom .status-bar) -->
  <div class="hero" :class="{ 'hero--sidebar': props.layout === 'sidebar' }">
    <div class="avatar-wrap">
      <img
        v-if="portraitSrc && !portraitLoadFailed"
        :key="portraitSrc"
        class="avatar"
        :src="portraitSrc"
        alt=""
        @error="onPortraitError"
      >
      <span v-else class="avatar-fallback">{{ emotionToEmoji[emotionKey()] ?? "😐" }}</span>
    </div>
    <h2 class="title">
      {{ props.name }}
    </h2>
    <p class="emotion-line">
      <span :key="emotionKey()" class="icon">{{ emotionToEmoji[emotionKey()] ?? "😐" }}</span>
      <span>{{ emotionDisplayLabel }}</span>
    </p>
  </div>
</template>

<style scoped>
.hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 20px 12px;
  background: var(--bg-primary);
}
.avatar-wrap {
  width: 100%;
  max-width: min(100%, 560px);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-bottom: 12px;
  /* Full portrait: no circular/fixed box crop */
  border-radius: 0;
  overflow: visible;
  background: transparent;
  border: none;
  box-shadow: none;
  padding: 0;
}
.avatar {
  display: block;
  max-width: 100%;
  max-height: min(38vh, min(400px, 55vw));
  width: auto;
  height: auto;
  object-fit: contain;
  object-position: center bottom;
  animation: avatarFadeIn 180ms ease-out;
}
.avatar-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  font-size: 72px;
  line-height: 1;
}
.title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}
.emotion-line {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-secondary);
}
.icon {
  font-size: 22px;
  line-height: 1;
  animation: pop 220ms cubic-bezier(0.2, 0.9, 0.4, 1.1);
}
@keyframes pop {
  0% {
    transform: scale(1);
  }
  40% {
    transform: scale(1.12);
  }
  100% {
    transform: scale(1);
  }
}

@keyframes avatarFadeIn {
  from {
    opacity: 0;
    transform: scale(0.985);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.hero--sidebar {
  padding: 12px 10px 8px;
}
.hero--sidebar .avatar-wrap {
  max-width: 100%;
  margin-bottom: 8px;
}
.hero--sidebar .avatar {
  max-height: min(48vh, 340px);
  max-width: 100%;
}
.hero--sidebar .avatar-fallback {
  min-height: 80px;
  font-size: 56px;
}
.hero--sidebar .title {
  font-size: 16px;
  margin-bottom: 4px;
}
.hero--sidebar .emotion-line {
  font-size: 13px;
}
.hero--sidebar .icon {
  font-size: 18px;
}
</style>
