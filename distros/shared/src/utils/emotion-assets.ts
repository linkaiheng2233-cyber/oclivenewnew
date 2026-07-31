/**
 * Emotion display assets: aligned with backend `Emotion` lowercase tags and filenames under `roles/{roleId}/assets/images/`.
 * When adding backend emotion enums, sync this file and add role pack images (placeholders OK).
 * Supported: happy / sad / angry / shy / confused / disgust* / neutral / excited, etc.
 */

/** Lowercase key; unknown keys fall back to raw label or default icon in UI. */
export const emotionToEmoji: Record<string, string> = {
  happy: '😊',
  sad: '😢',
  angry: '😠',
  shy: '☺️',
  confused: '😕',
  disgust: '🙄',
  neutral: '😐',
  /** When backend adds e.g. `Emotion::Excited`, extend here with image. */
  excited: '🤩',
}

/** Filename (no path); default fallback `{emotion}.png`. */
export const emotionToImage: Record<string, string> = {
  happy: 'happy.png',
  sad: 'sad.png',
  angry: 'angry.png',
  shy: 'shy.png',
  confused: 'confused.png',
  disgust: 'disgust_light.png',
  neutral: 'normal.png',
  excited: 'excited.png',
  disgust_light: 'disgust_light.png',
  disgust_mid: 'disgust_mid.png',
  disgust_heavy: 'disgust_heavy.png',
}

export function emotionToAssetFilename(emotion: string): string {
  const e = emotion.trim().toLowerCase()
  return emotionToImage[e] ?? `${e}.png`
}

/**
 * Ordered legacy-file candidates for hosts that have not received a catalog
 * directive yet. Intensity-aware packs use the mild image as their stable
 * baseline; older packs simply fall through to the canonical filename.
 */
export function emotionAssetCandidates(emotion: string): string[] {
  const key = emotion.trim().toLowerCase() || 'neutral'
  const primary = emotionToAssetFilename(key)
  const out = new Set<string>()

  const pushExpanded = (file: string) => {
    const idx = file.lastIndexOf('.')
    const base = idx >= 0 ? file.slice(0, idx) : file
    for (const ext of ['png', 'jpg', 'jpeg', 'webp'])
      out.add(`${base}.${ext}`)
  }

  pushExpanded(`${key}_mild.png`)
  pushExpanded(primary)
  if (key === 'neutral')
    pushExpanded('neutral.png')
  if (key.startsWith('disgust')) {
    pushExpanded('disgust_light.png')
    pushExpanded('disgust_mid.png')
    pushExpanded('disgust_heavy.png')
  }
  pushExpanded('normal.png')
  pushExpanded('neutral.png')
  return Array.from(out)
}
