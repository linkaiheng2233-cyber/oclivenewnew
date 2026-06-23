import type { ScriptLine } from './theaterLogic'

/** Single-pass cast rewrite progress label key. */
export const CAST_REWRITE_PROGRESS_KEY = 'theater.think.rewrite.writing'

/** First beat line preview after rewrite. */
export function pickCastRewritePreviewLine(beats: ScriptLine[]): string | null {
  const first = beats.find(b => b.text.trim().length > 0)
  if (!first)
    return null
  const snippet = first.text.length > 40 ? `${first.text.slice(0, 40)}…` : first.text
  return `${first.name}：${snippet}`
}
