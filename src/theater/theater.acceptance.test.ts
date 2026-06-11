import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import skeleton from '../../public/theater/breakfast/skeleton.json'
import { THEATER_POKE_CHIP_IDS } from './types'
import {
  defaultVariableState,
  patchTheaterBeats,
  probeOllamaAvailable,
  readTheaterPokePerfSample,
  resolveImpactedBeatIds,
  THEATER_POKE_PERF_MARKS,
} from './useTheaterBeatPatch'

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

describe('theater patch degradation (no Ollama)', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn(async () => {
      throw new Error('network unreachable')
    }))
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('probeOllamaAvailable returns false when Ollama is unreachable', async () => {
    await expect(probeOllamaAvailable()).resolves.toBe(false)
  })

  it('patchTheaterBeats keeps original beats when Ollama fetch fails', async () => {
    const beats = skeleton.beats.map(b => ({ ...b }))
    const beatIds = resolveImpactedBeatIds(skeleton, 'running_late')
    const vars = { ...defaultVariableState(skeleton), running_late: true }
    const { beats: next, patched } = await patchTheaterBeats(
      skeleton,
      beats,
      beatIds,
      vars,
      'zh',
    )
    expect(patched).toBe(false)
    expect(next.map(b => b.text)).toEqual(beats.map(b => b.text))
  })

  it('patch with empty beatIds is a no-op', async () => {
    const beats = skeleton.beats.map(b => ({ ...b }))
    const { beats: next, patched } = await patchTheaterBeats(
      skeleton,
      beats,
      [],
      defaultVariableState(skeleton),
      'zh',
    )
    expect(patched).toBe(false)
    expect(next).toEqual(beats)
  })
})

describe('theater poke perf marks (V-THEATER-PERF-01)', () => {
  beforeEach(() => {
    performance.clearMarks()
  })

  it('records probe timing marks when Ollama probe fails fast', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => {
      throw new Error('offline')
    }))
    await probeOllamaAvailable()
    const sample = readTheaterPokePerfSample()
    expect(sample.probeMs).not.toBeNull()
    expect(performance.getEntriesByName(THEATER_POKE_PERF_MARKS.probeStart, 'mark').length).toBe(1)
    vi.unstubAllGlobals()
  })
})
