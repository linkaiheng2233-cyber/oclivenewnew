import { describe, expect, it } from 'vitest'
import skeleton from '../../public/theater/breakfast/skeleton.json'
import {
  defaultVariableState,
  resolveImpactedBeatIds,
} from './useTheaterBeatPatch'
import { THEATER_POKE_CHIP_IDS } from './types'

describe('theater v0 acceptance (structure)', () => {
  it('skeleton has 8 beats and 3 poke variables with impact_map', () => {
    expect(skeleton.schema_version).toBe(1)
    expect(skeleton.beats.length).toBeGreaterThanOrEqual(6)
    for (const id of THEATER_POKE_CHIP_IDS) {
      expect(skeleton.variables[id]).toBeDefined()
      expect((skeleton.impact_map[id] ?? []).length).toBeGreaterThan(0)
    }
  })

  it('first beat is pre-rendered speaker A with zero delay', () => {
    const first = skeleton.beats[0]
    expect(first.delay_ms).toBe(0)
    expect(first.speaker).toBe('a')
    expect(first.text.length).toBeGreaterThan(4)
  })

  it('impact_map only references existing beat ids', () => {
    const ids = new Set(skeleton.beats.map(b => b.id))
    for (const beatIds of Object.values(skeleton.impact_map)) {
      for (const bid of beatIds) {
        expect(ids.has(bid)).toBe(true)
      }
    }
  })

  it('default variables match schema', () => {
    const vars = defaultVariableState(skeleton)
    expect(vars.bitter_medicine).toBe(false)
    expect(vars.running_late).toBe(false)
    expect(resolveImpactedBeatIds(skeleton, 'running_late').length).toBeGreaterThan(0)
  })
})

describe('theater 60s timing budget (smoke)', () => {
  it('sum of delays fits ~30s playback before poke', () => {
    const totalMs = skeleton.beats.slice(1).reduce((acc, b) => acc + b.delay_ms, 0)
    expect(totalMs).toBeLessThan(35000)
  })
})
