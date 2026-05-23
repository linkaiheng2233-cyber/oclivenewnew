/** 与后端 `personality_vector` 七维顺序及 oclive-new 开发面板一致 */
export const PERSONALITY_TRAIT_KEYS = [
  'stubbornness',
  'clinginess',
  'sensitivity',
  'assertiveness',
  'forgiveness',
  'talkativeness',
  'warmth',
] as const

export function vec7ToRecord(
  v: number[] | undefined,
): Record<(typeof PERSONALITY_TRAIT_KEYS)[number], number> {
  const out = {} as Record<
    (typeof PERSONALITY_TRAIT_KEYS)[number],
    number
  >
  for (let i = 0; i < PERSONALITY_TRAIT_KEYS.length; i++) {
    const key = PERSONALITY_TRAIT_KEYS[i]
    const raw = v?.[i]
    out[key]
      = typeof raw === 'number' && Number.isFinite(raw) ? raw : 0
  }
  return out
}
