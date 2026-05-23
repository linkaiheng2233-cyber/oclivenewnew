import { rt } from '../i18n/runtimeT'

/** 与后端 `RelationState::as_str` 一致（好感度关系阶段） */
const RELATION_ORDER = [
  'Stranger',
  'Acquaintance',
  'Friend',
  'CloseFriend',
  'Partner',
] as const

const RELATION_UPGRADE_I18N_KEY: Record<string, string> = {
  Acquaintance: 'relation.upgradeAcquaintance',
  Friend: 'relation.upgradeFriend',
  CloseFriend: 'relation.upgradeCloseFriend',
  Partner: 'relation.upgradePartner',
}

function rankOf(state: string): number {
  const i = RELATION_ORDER.indexOf(state as (typeof RELATION_ORDER)[number])
  return i >= 0 ? i : -1
}

/** 仅当新阶段高于旧阶段时返回提示文案，否则 `null`（不提示降级）。 */
export function getRelationUpgradeMessage(
  newState: string,
  oldState: string,
): string | null {
  if (!newState || !oldState || newState === oldState)
    return null
  const newIndex = rankOf(newState)
  const oldIndex = rankOf(oldState)
  if (newIndex < 0 || oldIndex < 0) {
    console.warn(
      `[Relation] Unknown states: new=${newState}, old=${oldState}`,
    )
    return null
  }
  if (newIndex <= oldIndex)
    return null
  const key = RELATION_UPGRADE_I18N_KEY[newState]
  return key ? rt(key) : rt('relation.upgradeUnknown', { state: newState })
}
