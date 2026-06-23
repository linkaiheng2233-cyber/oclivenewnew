import { describe, expect, it } from 'vitest'
import {
  DEFAULT_PAIR_RELATION_ID,
  normalizePairRelationId,
  resolvePairRelationHint,
  THEATER_PAIR_RELATIONS,
} from './theaterPairRelation'
import type { TheaterSkeleton } from './theaterLogic'

describe('theaterPairRelation', () => {
  it('defaults unknown ids to family', () => {
    expect(normalizePairRelationId(undefined)).toBe(DEFAULT_PAIR_RELATION_ID)
    expect(normalizePairRelationId('invalid')).toBe(DEFAULT_PAIR_RELATION_ID)
  })

  it('prefers skeleton pairRelations prompt when present', () => {
    const sk: TheaterSkeleton = {
      scene: 'breakfast',
      cast: { a: { roleId: 'a', name: 'A' }, b: { roleId: 'b', name: 'B' } },
      beats: [{ id: 'b1', cast: 'a', name: 'A', text: 'x' }],
      forks: {},
      pairRelations: {
        lover: { displayName: '恋人', promptHint: '来自 skeleton 的恋人提示' },
      },
    }
    expect(resolvePairRelationHint('lover', sk)).toBe('来自 skeleton 的恋人提示')
    expect(resolvePairRelationHint('friend', sk)).toBe(THEATER_PAIR_RELATIONS.friend.promptHint)
  })
})
