/** Matches backend `InteractionMode` / DTO. */
export type InteractionMode = 'immersive' | 'pure_chat'

/** Normalize API string to union type (unknown → immersive). */
export function normalizeInteractionMode(
  raw: string | undefined | null,
): InteractionMode {
  return raw === 'pure_chat' ? 'pure_chat' : 'immersive'
}

/** `interaction_mode_pack_default`: keep only valid values. */
export function packDefaultFromApi(
  raw: string | null | undefined,
): InteractionMode | null {
  if (raw === 'pure_chat' || raw === 'immersive')
    return raw
  return null
}
