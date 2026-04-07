/** 与后端 `RelationState::as_str` 一致（好感度关系阶段） */
const RELATION_ORDER = [
  "Stranger",
  "Acquaintance",
  "Friend",
  "CloseFriend",
  "Partner",
] as const;

/** 升级时展示的文案（按新阶段） */
const RELATION_UPGRADE_TEXT: Record<string, string> = {
  Acquaintance: "关系更近了一步：你们不再陌生。",
  Friend: "✨ 你们成为了朋友！",
  CloseFriend: "🎉 你们已经是好朋友了！",
  Partner: "💖 关系阶段：伴侣",
};

function rankOf(state: string): number {
  const i = RELATION_ORDER.indexOf(state as (typeof RELATION_ORDER)[number]);
  return i >= 0 ? i : -1;
}

/** 仅当新阶段高于旧阶段时返回提示文案，否则 `null`（不提示降级）。 */
export function getRelationUpgradeMessage(
  newState: string,
  oldState: string,
): string | null {
  if (!newState || !oldState || newState === oldState) return null;
  const newIndex = rankOf(newState);
  const oldIndex = rankOf(oldState);
  if (newIndex < 0 || oldIndex < 0) {
    console.warn(
      `[Relation] 未知阶段：new=${newState}, old=${oldState}`,
    );
    return null;
  }
  if (newIndex <= oldIndex) return null;
  return RELATION_UPGRADE_TEXT[newState] ?? `关系阶段更新为：${newState}`;
}
