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
