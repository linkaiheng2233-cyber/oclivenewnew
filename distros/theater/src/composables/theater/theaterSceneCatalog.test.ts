import type { TheaterSkeleton } from './theaterLogic'
import { describe, expect, it } from 'vitest'
import {
  getPokeChipsForPreset,
  listTheaterScenePresets,
  resolveActivePokeChips,
} from './theaterSceneCatalog'

describe('theaterSceneCatalog', () => {
  it('lists four scene presets with poke enabled', () => {
    const presets = listTheaterScenePresets()
    expect(presets).toHaveLength(4)
    expect(presets.every(p => p.pokeEnabled)).toBe(true)
  })

  it('each preset declares four poke chips', () => {
    for (const preset of listTheaterScenePresets()) {
      expect(getPokeChipsForPreset(preset.id)).toHaveLength(4)
    }
  })

  it('resolveActivePokeChips intersects catalog chips with skeleton forks', () => {
    const preset = listTheaterScenePresets().find(p => p.id === 'supermarket')!
    const skeleton: TheaterSkeleton = {
      scene: 'supermarket',
      cast: {
        a: { roleId: 'mumu', name: '木木' },
        b: { roleId: '枫侵月', name: '枫侵月' },
      },
      beats: [],
      forks: {
        buyMilk: [{
          chipId: 'buyMilk',
          insertAfterBeatId: 'b5',
          patchLines: [],
        }],
      },
    }
    const active = resolveActivePokeChips(preset, skeleton)
    expect(active).toHaveLength(1)
    expect(active[0]?.id).toBe('buyMilk')
  })

  it('resolveActivePokeChips returns empty when skeleton has no forks', () => {
    const preset = listTheaterScenePresets().find(p => p.id === 'breakfast')!
    const active = resolveActivePokeChips(preset, {
      scene: 'breakfast',
      cast: {
        a: { roleId: 'mumu', name: '木木' },
        b: { roleId: '枫侵月', name: '枫侵月' },
      },
      beats: [],
      forks: {},
    })
    expect(active).toEqual([])
  })
})
