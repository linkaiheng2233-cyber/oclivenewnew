<script setup lang="ts">
import { readRoleAssetBytes, resolveRoleAssetPath } from '@oclive/shared/api'
import {
  emotionAssetCandidates,
  emotionToEmoji,
} from '@oclive/shared/utils/emotion-assets'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    roleId: string
    name: string
    emotion: string
    /** Catalog / directive relative asset path (e.g. assets/images/happy.webp) */
    portraitAssetRelPath?: string | null
    layout?: 'stack' | 'sidebar' | 'preview'
  }>(),
  { layout: 'stack' },
)

const { t, te } = useI18n()

const portraitSrc = ref<string | null>(null)
const portraitBlobUrl = ref<string | null>(null)
const portraitLoadFailed = ref(false)
let portraitGeneration = 0
let portraitRoleId = ''
let loadedCatalogRel = ''

function isTauri(): boolean {
  return (
    typeof window !== 'undefined'
    && Object.hasOwn(window, '__TAURI_INTERNALS__')
  )
}

function revokeBlob(): void {
  if (portraitBlobUrl.value) {
    URL.revokeObjectURL(portraitBlobUrl.value)
    portraitBlobUrl.value = null
  }
}

function showPortrait(src: string, isBlob = false): void {
  revokeBlob()
  portraitBlobUrl.value = isBlob ? src : null
  portraitSrc.value = src
  portraitLoadFailed.value = false
}

function emotionKey(): string {
  return props.emotion.trim().toLowerCase() || 'neutral'
}

const emotionDisplayLabel = computed(() => {
  const key = emotionKey()
  const path = `emotionUi.${key}`
  return te(path) ? t(path) : props.emotion.trim() || key
})

async function tryLoadRelativeAsset(rel: string, gen: number): Promise<boolean> {
  let path: string | null
  try {
    path = await resolveRoleAssetPath(props.roleId, rel)
  }
  catch (e) {
    if (gen !== portraitGeneration)
      return false
    console.warn('[CharacterInfo] catalog asset resolve failed', e)
    return false
  }
  if (!path)
    return false
  if (gen !== portraitGeneration)
    return false

  const filename = rel.split('/').pop() ?? rel
  if (isTauri()) {
    try {
      const bytes = await readRoleAssetBytes(props.roleId, rel)
      if (gen !== portraitGeneration)
        return false
      if (!bytes)
        return false
      const mime = filename.endsWith('.webp')
        ? 'image/webp'
        : filename.endsWith('.jpg') || filename.endsWith('.jpeg')
          ? 'image/jpeg'
          : filename.endsWith('.gif')
            ? 'image/gif'
            : 'image/png'
      const blob = new Blob([new Uint8Array(bytes)], { type: mime })
      const url = URL.createObjectURL(blob)
      showPortrait(url, true)
      return true
    }
    catch (e) {
      console.warn('[CharacterInfo] catalog readRoleAssetBytes failed', e)
    }
  }

  try {
    if (gen !== portraitGeneration)
      return false
    showPortrait(convertFileSrc(path))
    return true
  }
  catch (e) {
    console.warn('[CharacterInfo] catalog convertFileSrc failed', e)
    return false
  }
}

async function refreshPortrait(): Promise<void> {
  const catalogRel = props.portraitAssetRelPath?.trim()
  if (catalogRel && loadedCatalogRel === catalogRel && portraitRoleId === props.roleId && !portraitLoadFailed.value)
    return
  const gen = ++portraitGeneration
  loadedCatalogRel = ''
  portraitLoadFailed.value = false
  // Keep the current portrait while a same-role replacement loads. A role
  // change must clear it immediately, so another character is never shown.
  if (portraitRoleId !== props.roleId) {
    revokeBlob()
    portraitSrc.value = null
    portraitRoleId = props.roleId
  }

  if (catalogRel) {
    const loadedCatalog = await tryLoadRelativeAsset(catalogRel, gen)
    if (gen !== portraitGeneration)
      return
    if (loadedCatalog) {
      loadedCatalogRel = catalogRel
      return
    }
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
    if (gen !== portraitGeneration)
      return

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
        showPortrait(url, true)
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
      showPortrait(convertFileSrc(path))
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
  ++portraitGeneration
  revokeBlob()
})
</script>

<template>
  <!-- Match oclive-new .main-content: hero image + name + emotion (affinity in bottom .status-bar) -->
  <div class="hero" :class="{ 'hero--sidebar': props.layout === 'sidebar', 'hero--preview': props.layout === 'preview' }" :aria-hidden="props.layout === 'preview' ? 'true' : undefined">
    <div class="avatar-wrap">
      <img
        v-if="portraitSrc && !portraitLoadFailed"
        :key="portraitSrc"
        class="avatar"
        :src="portraitSrc"
        alt=""
        decoding="async"
        @error="onPortraitError"
      >
      <span v-else class="avatar-fallback">{{ props.layout === 'preview' ? props.name.trim().slice(0, 1) || '◇' : emotionToEmoji[emotionKey()] ?? "😐" }}</span>
    </div>
    <h2 v-if="props.layout !== 'preview'" class="title">
      {{ props.name }}
    </h2>
    <p v-if="props.layout !== 'preview'" class="emotion-line">
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
  max-height: min(50vh, min(620px, 62vw));
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
  max-height: min(72vh, 880px);
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

/* Library framing crops only the display. Runtime portraits keep their original layout. */
.hero--preview { width:100%;height:100%;padding:0;overflow:hidden; }
.hero--preview .avatar-wrap { width:100%;height:100%;max-width:none;margin:0;overflow:hidden; }
.hero--preview .avatar { width:100%;height:100%;max-width:none;max-height:none;object-fit:cover;object-position:50% 18%;animation:none; }
.hero--preview .avatar-fallback { width:100%;height:100%;min-height:0;font-size:48px;font-weight:650;color:var(--accent);background:color-mix(in srgb,var(--accent) 9%,var(--bg-primary)); }
@media(prefers-reduced-motion:reduce) { .avatar,.icon { animation:none; } }
</style>
