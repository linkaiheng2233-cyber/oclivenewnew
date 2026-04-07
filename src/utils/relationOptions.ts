import { OCLIVE_DEFAULT_RELATION_SENTINEL } from "./tauri-api";
import type { UserRelationDto } from "./tauri-api";

export type RelationOptionRow = { id: string; name: string };

/**
 * 身份下拉选项：首项为「默认身份」，与后端 `set_user_relation` 哨兵一致。
 * 供顶栏、运行时面板等共用，避免多处复制粘贴。
 */
export function buildRelationDropdownOptions(
  userRelations: UserRelationDto[],
  defaultRelation: string,
): RelationOptionRow[] {
  const rows = userRelations.map((r) => ({
    id: r.id,
    name: r.name,
  }));
  const defId = defaultRelation || "friend";
  const defLabel = rows.find((r) => r.id === defId)?.name ?? defId;
  return [
    {
      id: OCLIVE_DEFAULT_RELATION_SENTINEL,
      name: `默认身份（${defLabel}）`,
    },
    ...rows,
  ];
}
