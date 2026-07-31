import type { TheaterForkTemplate, TheaterScriptLine } from '@oclive/shared/api/theater'
import type { TheaterCastConfig } from './theaterCastConfig'
import type { PokeChipId, ScriptLine, TheaterSkeleton } from './theaterLogic'
import {
  DEFAULT_THEATER_CAST_CONFIG,
  isDefaultCastConfig,
  syncBeatSpeakerNames,

} from './theaterCastConfig'
import {
  isBaselinePregenCast,
  normalizePairRelationId,
} from './theaterPairRelation'

export const THEATER_ADAPTED_STORAGE_KEY = 'oclive.theater.adapted.v2'
const LEGACY_ADAPTED_STORAGE_KEY = 'oclive.theater.adapted.v1'

function migrateLegacyAdaptedCache(): void {
  try {
    if (localStorage.getItem(THEATER_ADAPTED_STORAGE_KEY))
      return
    const legacy = localStorage.getItem(LEGACY_ADAPTED_STORAGE_KEY)
    if (!legacy)
      return
    localStorage.removeItem(LEGACY_ADAPTED_STORAGE_KEY)
  }
  catch {
    /* ignore */
  }
}

migrateLegacyAdaptedCache()

export interface AdaptedCacheEntry {
  skeletonHash: string
  beats: TheaterScriptLine[]
  forks: TheaterForkTemplate[]
  source: string
  ts: number
}

type AdaptedCacheStore = Record<string, AdaptedCacheEntry>

function scriptLineToDto(line: ScriptLine): TheaterScriptLine {
  return {
    id: line.id,
    cast: line.cast,
    name: line.name,
    text: line.text,
    stage_hint: line.stageHint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

function scriptLineFromDto(line: TheaterScriptLine): ScriptLine {
  return {
    id: line.id,
    cast: line.cast as ScriptLine['cast'],
    name: line.name,
    text: line.text,
    stageHint: line.stage_hint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

export function isDefaultCast(config: TheaterCastConfig): boolean {
  return isDefaultCastConfig(config)
}

export function needsCastAdaptation(config: TheaterCastConfig): boolean {
  return !isBaselinePregenCast(
    config,
    DEFAULT_THEATER_CAST_CONFIG.castA.roleId,
    DEFAULT_THEATER_CAST_CONFIG.castB.roleId,
  )
}

export type CastAdaptStatus = 'default' | 'cached' | 'renameOnly'

export type CastAdaptIssueKind = 'failure' | 'degraded'

export interface CastAdaptIssue {
  kind: CastAdaptIssueKind
  code: string
}

/** Whether applied cast has AI-adapted cache or is rename-only baseline. */
export function resolveCastAdaptStatus(
  config: TheaterCastConfig,
  presetId: string,
  skeletonHash: string,
): CastAdaptStatus {
  if (isBaselinePregenCast(
    config,
    DEFAULT_THEATER_CAST_CONFIG.castA.roleId,
    DEFAULT_THEATER_CAST_CONFIG.castB.roleId,
  )) {
    return 'default'
  }
  const cached = getAdaptedCache(config, presetId, skeletonHash)
  return cached ? 'cached' : 'renameOnly'
}

const DEFAULT_ADAPTED_CACHE_MAX = 8

/** Drop oldest adapted entries when store exceeds `maxEntries` (LRU by `ts`). */
export function pruneAdaptedCache(maxEntries = DEFAULT_ADAPTED_CACHE_MAX): void {
  const store = readCacheStore()
  const keys = Object.keys(store)
  if (keys.length <= maxEntries)
    return
  const sorted = [...keys].sort(
    (a, b) => (store[a]?.ts ?? 0) - (store[b]?.ts ?? 0),
  )
  for (const key of sorted.slice(0, keys.length - maxEntries))
    delete store[key]
  writeCacheStore(store)
}

export function clearAdaptedCacheForCast(
  config: TheaterCastConfig,
  presetId = 'breakfast',
): void {
  const key = cacheKeyForCast(config, presetId)
  const store = readCacheStore()
  if (!(key in store))
    return
  delete store[key]
  writeCacheStore(store)
}

/** Number of cached `(presetId, castA, castB)` adaptation entries. */
export function countAdaptedCacheEntries(): number {
  return Object.keys(readCacheStore()).length
}

/** Remove all cast adaptation cache entries; returns how many were cleared. */
export function clearAllAdaptedCache(): number {
  const store = readCacheStore()
  const count = Object.keys(store).length
  if (count === 0)
    return 0
  writeCacheStore({})
  return count
}

export function computeSkeletonHash(canonical: TheaterSkeleton): string {
  const payload = {
    scene: canonical.scene,
    sceneId: canonical.sceneId,
    beats: canonical.beats.map(b => ({ id: b.id, cast: b.cast })),
    forks: Object.fromEntries(
      Object.entries(canonical.forks).map(([k, v]) => [
        k,
        v?.map(f => ({
          chipId: f.chipId,
          insertAfterBeatId: f.insertAfterBeatId,
          patchIds: f.patchLines.map(p => p.id),
        })),
      ]),
    ),
  }
  const str = JSON.stringify(payload)
  let h = 0
  for (let i = 0; i < str.length; i++)
    h = ((h << 5) - h + str.charCodeAt(i)) | 0
  return (h >>> 0).toString(36)
}

export function cacheKeyForCast(config: TheaterCastConfig, presetId: string): string {
  const rel = normalizePairRelationId(config.pairRelationId)
  return `${presetId}:${config.castA.roleId}:${config.castB.roleId}:${rel}`
}

function readCacheStore(): AdaptedCacheStore {
  try {
    const raw = localStorage.getItem(THEATER_ADAPTED_STORAGE_KEY)
    if (!raw)
      return {}
    const parsed = JSON.parse(raw) as AdaptedCacheStore
    return parsed && typeof parsed === 'object' ? parsed : {}
  }
  catch {
    return {}
  }
}

function writeCacheStore(store: AdaptedCacheStore): void {
  try {
    localStorage.setItem(THEATER_ADAPTED_STORAGE_KEY, JSON.stringify(store))
  }
  catch {
    /* ignore quota / private mode */
  }
}

export function getAdaptedCache(
  config: TheaterCastConfig,
  presetId: string,
  skeletonHash: string,
): AdaptedCacheEntry | null {
  const key = cacheKeyForCast(config, presetId)
  const entry = readCacheStore()[key]
  if (!entry || entry.skeletonHash !== skeletonHash)
    return null
  return entry
}

export function setAdaptedCache(
  config: TheaterCastConfig,
  presetId: string,
  entry: AdaptedCacheEntry,
): void {
  const key = cacheKeyForCast(config, presetId)
  const store = readCacheStore()
  store[key] = { ...entry, ts: entry.ts || Date.now() }
  writeCacheStore(store)
  pruneAdaptedCache()
}

export function skeletonToForkTemplates(sk: TheaterSkeleton): TheaterForkTemplate[] {
  const out: TheaterForkTemplate[] = []
  for (const entries of Object.values(sk.forks)) {
    if (!entries?.length)
      continue
    for (const fork of entries) {
      out.push({
        chip_id: fork.chipId,
        insert_after_beat_id: fork.insertAfterBeatId,
        patch_lines: fork.patchLines.map(scriptLineToDto),
      })
    }
  }
  return out
}

/** Write adapted beats + fork patch lines onto runtime skeleton; structure ids unchanged. */
export function applyAdaptedToSkeleton(
  sk: TheaterSkeleton,
  beats: TheaterScriptLine[],
  forks: TheaterForkTemplate[],
): TheaterSkeleton {
  const beatById = new Map(beats.map(b => [b.id, scriptLineFromDto(b)]))
  const nextBeats = sk.beats.map((line) => {
    const adapted = beatById.get(line.id)
    return adapted ?? { ...line }
  })

  const forkByChip = new Map(forks.map(f => [f.chip_id, f]))
  const nextForks: TheaterSkeleton['forks'] = {}
  for (const [chipId, entries] of Object.entries(sk.forks)) {
    if (!entries?.length)
      continue
    const adapted = forkByChip.get(chipId)
    nextForks[chipId as PokeChipId] = entries.map((fork) => {
      if (!adapted || adapted.chip_id !== fork.chipId)
        return { ...fork, patchLines: fork.patchLines.map(p => ({ ...p })) }
      const lineById = new Map(adapted.patch_lines.map(l => [l.id, l]))
      return {
        chipId: fork.chipId,
        insertAfterBeatId: fork.insertAfterBeatId,
        patchLines: fork.patchLines.map((line) => {
          const dto = lineById.get(line.id)
          return dto ? scriptLineFromDto(dto) : { ...line }
        }),
      }
    })
  }

  return {
    ...sk,
    cast: {
      a: { ...sk.cast.a },
      b: { ...sk.cast.b },
    },
    beats: nextBeats,
    forks: nextForks,
  }
}

/** Apply a full cast_rewrite result (new beats + forks) onto a runtime baseline. */
export function buildRuntimeFromRewrite(
  baseline: TheaterSkeleton,
  beats: TheaterScriptLine[],
  forks: TheaterForkTemplate[],
): TheaterSkeleton {
  const nextForks: TheaterSkeleton['forks'] = {}
  for (const tmpl of forks) {
    const chipId = tmpl.chip_id as PokeChipId
    nextForks[chipId] = [{
      chipId,
      insertAfterBeatId: tmpl.insert_after_beat_id,
      patchLines: tmpl.patch_lines.map(scriptLineFromDto),
    }]
  }
  return syncBeatSpeakerNames({
    ...baseline,
    cast: {
      a: { ...baseline.cast.a },
      b: { ...baseline.cast.b },
    },
    beats: beats.map(scriptLineFromDto),
    forks: nextForks,
  })
}
