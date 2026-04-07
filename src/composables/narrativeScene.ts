/**
 * 叙事场景 id 解析：与 DB `user_presence_scene` 对齐（覆盖本地持久化顶栏 id）。
 * 若 DB 存了已不在 manifest 中的 id，则回退为 current_scene → 本地持久化 → 列表首项。
 */
export function resolveUserNarrativeSceneId(
  userPresence: string | null,
  backendCurrent: string | null,
  scenes: string[],
  persistedId: string,
): string {
  const list = scenes.length > 0 ? scenes : ["default"];
  const pick = (id: string | null | undefined) =>
    id && list.includes(id) ? id : null;

  const ups = pick(userPresence);
  if (ups) return ups;

  if (userPresence && userPresence.trim() !== "") {
    if (pick(backendCurrent)) return pick(backendCurrent)!;
    if (pick(persistedId)) return pick(persistedId)!;
    return list[0] ?? "default";
  }

  if (pick(persistedId)) return pick(persistedId)!;
  if (pick(backendCurrent)) return pick(backendCurrent)!;
  return list[0] ?? "default";
}
