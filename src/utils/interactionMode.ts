/** Matches backend `InteractionMode` / DTO. */
export type InteractionMode = 'immersive' | 'pure_chat'

/** Immersive-only UX: virtual time, narrative scenes, travel bars, life schedule hints, etc. */
export function isImmersiveMode(mode: InteractionMode): boolean {
  return mode === 'immersive'
}

/** Normalize API string to union type (unknown → pure_chat). */
export function normalizeInteractionMode(
  raw: string | undefined | null,
): InteractionMode {
  if (raw === 'immersive')
    return 'immersive'
  return 'pure_chat'
}

/** `interaction_mode_pack_default`: keep only valid values. */
export function packDefaultFromApi(
  raw: string | null | undefined,
): InteractionMode | null {
  if (raw === 'pure_chat' || raw === 'immersive')
    return raw
  return null
}
