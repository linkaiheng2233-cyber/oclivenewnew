/**
 * Narrative scene id resolution: align with DB `user_presence_scene` (overrides locally persisted top-bar id).
 * If DB holds an id no longer in manifest, fall back: current_scene → local persistence → first list item.
 */
export function resolveUserNarrativeSceneId(
  userPresence: string | null,
  backendCurrent: string | null,
  scenes: string[],
  persistedId: string,
): string {
  const list = scenes.length > 0 ? scenes : ['default']
  const pick = (id: string | null | undefined) =>
    id && list.includes(id) ? id : null

  const ups = pick(userPresence)
  if (ups)
    return ups

  if (userPresence && userPresence.trim() !== '') {
    if (pick(backendCurrent))
      return pick(backendCurrent)!
    if (pick(persistedId))
      return pick(persistedId)!
    return list[0] ?? 'default'
  }

  if (pick(persistedId))
    return pick(persistedId)!
  if (pick(backendCurrent))
    return pick(backendCurrent)!
  return list[0] ?? 'default'
}
