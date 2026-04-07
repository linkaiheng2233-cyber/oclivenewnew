/** 与后端 `InteractionMode` / DTO 一致 */
export type InteractionMode = "immersive" | "pure_chat";

/** 将 API 字符串规范为联合类型（未知则沉浸） */
export function normalizeInteractionMode(
  raw: string | undefined | null,
): InteractionMode {
  return raw === "pure_chat" ? "pure_chat" : "immersive";
}

/** `interaction_mode_pack_default`：仅合法值保留 */
export function packDefaultFromApi(
  raw: string | null | undefined,
): InteractionMode | null {
  if (raw === "pure_chat" || raw === "immersive") return raw;
  return null;
}
