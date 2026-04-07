<script setup lang="ts">
import { readBinaryFile } from "@tauri-apps/api/fs";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { onBeforeUnmount, ref, watch } from "vue";
import {
  emotionToAssetFilename,
  emotionToEmoji,
  emotionToLabelZh,
} from "../utils/emotion-assets";
import { resolveRoleAssetPath } from "../utils/tauri-api";

const props = withDefaults(
  defineProps<{
    roleId: string;
    name: string;
    emotion: string;
    /** stack：纵向主布局；sidebar：左侧窄栏时略压缩立绘与留白 */
    layout?: "stack" | "sidebar";
  }>(),
  { layout: "stack" },
);

const portraitSrc = ref<string | null>(null);
const portraitBlobUrl = ref<string | null>(null);
const portraitLoadFailed = ref(false);

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

function revokeBlob(): void {
  if (portraitBlobUrl.value) {
    URL.revokeObjectURL(portraitBlobUrl.value);
    portraitBlobUrl.value = null;
  }
}

function emotionKey(): string {
  return props.emotion.trim().toLowerCase() || "neutral";
}

function emotionAssetCandidates(key: string): string[] {
  const primary = emotionToAssetFilename(key);
  const out = new Set<string>();

  const pushExpanded = (file: string) => {
    const idx = file.lastIndexOf(".");
    const base = idx >= 0 ? file.slice(0, idx) : file;
    for (const ext of ["png", "jpg", "jpeg", "webp"]) {
      out.add(`${base}.${ext}`);
    }
  };

  pushExpanded(primary);
  // 常见兼容：如果角色包使用了不同命名（例如 neutral.png）
  if (key === "neutral") {
    pushExpanded("neutral.png");
  }
  if (key.startsWith("disgust")) {
    pushExpanded("disgust_light.png");
    pushExpanded("disgust_mid.png");
    pushExpanded("disgust_heavy.png");
  }
  // 最终兜底
  pushExpanded("normal.png");
  pushExpanded("neutral.png");
  return Array.from(out);
}

async function refreshPortrait(): Promise<void> {
  portraitLoadFailed.value = false;
  revokeBlob();
  portraitSrc.value = null;

  const key = emotionKey();
  let loaded = false;
  for (const filename of emotionAssetCandidates(key)) {
    const rel = `assets/images/${filename}`;
    let path: string | null;
    try {
      path = await resolveRoleAssetPath(props.roleId, rel);
    } catch (e) {
      console.warn("[CharacterInfo] resolve_role_asset_path failed", e);
      portraitLoadFailed.value = true;
      return;
    }
    if (!path) continue;

    /* 优先 readBinaryFile + Blob：不依赖 asset 自定义协议，避免 net::ERR_CONNECTION_REFUSED */
    if (isTauri()) {
      try {
        const bytes = await readBinaryFile(path);
        const mime = filename.endsWith(".webp")
          ? "image/webp"
          : filename.endsWith(".jpg") || filename.endsWith(".jpeg")
            ? "image/jpeg"
            : filename.endsWith(".gif")
              ? "image/gif"
              : "image/png";
        const blob = new Blob([bytes], { type: mime });
        const url = URL.createObjectURL(blob);
        portraitBlobUrl.value = url;
        portraitSrc.value = url;
        loaded = true;
        break;
      } catch (e) {
        console.warn(
          "[CharacterInfo] readBinaryFile failed, fallback convertFileSrc",
          e,
        );
      }
    }

    try {
      portraitSrc.value = convertFileSrc(path);
      loaded = true;
      break;
    } catch (e) {
      console.warn("[CharacterInfo] convertFileSrc failed", e);
    }
  }

  if (!loaded) {
    portraitLoadFailed.value = true;
  }
}

function onPortraitError(): void {
  portraitLoadFailed.value = true;
}

watch(
  () => [props.roleId, props.emotion] as const,
  () => {
    void refreshPortrait();
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  revokeBlob();
});
</script>

<template>
  <!-- 对齐 oclive-new .main-content：大图 + 名称 + 情绪（好感度在底部 .status-bar） -->
  <div class="hero" :class="{ 'hero--sidebar': props.layout === 'sidebar' }">
    <div class="avatar-wrap">
      <img
        v-if="portraitSrc && !portraitLoadFailed"
        :key="portraitSrc"
        class="avatar"
        :src="portraitSrc"
        alt=""
        @error="onPortraitError"
      />
      <span v-else class="avatar-fallback">{{ emotionToEmoji[emotionKey()] ?? "😐" }}</span>
    </div>
    <h2 class="title">{{ props.name }}</h2>
    <p class="emotion-line">
      <span :key="emotionKey()" class="icon">{{ emotionToEmoji[emotionKey()] ?? "😐" }}</span>
      <span>{{ emotionToLabelZh[emotionKey()] ?? props.emotion }}</span>
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
  /* 完整展示立绘：无圆形/固定方框裁切 */
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
