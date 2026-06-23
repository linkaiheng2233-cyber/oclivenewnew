import type { TheaterSkeleton } from './theaterLogic'

/** Official pair-relation preset ids (mode 1; custom relations deferred to mode 2+). */
export type TheaterPairRelationId = 'family' | 'friend' | 'stranger' | 'lover'

export const DEFAULT_PAIR_RELATION_ID: TheaterPairRelationId = 'family'

export interface TheaterPairRelationDef {
  displayName: string
  promptHint: string
}

export const THEATER_PAIR_RELATIONS: Record<TheaterPairRelationId, TheaterPairRelationDef> = {
  family: {
    displayName: '家人',
    promptHint: '同住或家人般的日常照应：一方略操心、一方嘴硬但不真疏远，可拌嘴也可温存。',
  },
  friend: {
    displayName: '朋友',
    promptHint: '同龄好友：说话随意、互怼互损，但会自然帮对方收拾东西、提醒出门。',
  },
  stranger: {
    displayName: '陌生人',
    promptHint: '刚认识或并不熟：礼貌克制、有距离感，避免过度亲密称呼与同居式唠叨。',
  },
  lover: {
    displayName: '恋人',
    promptHint: '恋人或暧昧对象：语气更软、更在意对方反应，可害羞、可斗嘴，但底色是彼此在意。',
  },
}

export const THEATER_PAIR_RELATION_IDS = Object.keys(THEATER_PAIR_RELATIONS) as TheaterPairRelationId[]

export function isTheaterPairRelationId(raw: unknown): raw is TheaterPairRelationId {
  return typeof raw === 'string' && raw in THEATER_PAIR_RELATIONS
}

export function normalizePairRelationId(raw: unknown): TheaterPairRelationId {
  return isTheaterPairRelationId(raw) ? raw : DEFAULT_PAIR_RELATION_ID
}

export function resolvePairRelationDef(
  id: TheaterPairRelationId,
  skeleton?: TheaterSkeleton | null,
): TheaterPairRelationDef {
  const fromSkeleton = skeleton?.pairRelations?.[id]
  if (fromSkeleton?.promptHint?.trim())
    return fromSkeleton
  return THEATER_PAIR_RELATIONS[id]
}

export function resolvePairRelationHint(
  id: TheaterPairRelationId,
  skeleton?: TheaterSkeleton | null,
): string {
  return resolvePairRelationDef(id, skeleton).promptHint.trim()
}

export function resolveDefaultPairRelationId(skeleton?: TheaterSkeleton | null): TheaterPairRelationId {
  return normalizePairRelationId(skeleton?.defaultPairRelation)
}

/** True when both slots are the official example roles (order-independent). */
export function isOfficialCastPair(
  config: { castA: { roleId: string }, castB: { roleId: string } },
  officialRoleA: string,
  officialRoleB: string,
): boolean {
  const ids = new Set([config.castA.roleId, config.castB.roleId])
  return ids.has(officialRoleA) && ids.has(officialRoleB)
}

/** True when official pregen skeleton applies (official pair + family relation). */
export function isBaselinePregenCast(
  config: { castA: { roleId: string }, castB: { roleId: string }, pairRelationId?: TheaterPairRelationId },
  defaultCastRoleA: string,
  defaultCastRoleB: string,
): boolean {
  return isOfficialCastPair(config, defaultCastRoleA, defaultCastRoleB)
    && normalizePairRelationId(config.pairRelationId) === DEFAULT_PAIR_RELATION_ID
}
