import { rt } from '../i18n/runtimeT'

/** Matches backend `RelationState::as_str` (affection relation stage). */
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

/** Return upgrade message only when new stage is higher than old; otherwise `null` (no downgrade toast). */
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
