import type { PokeChipId, ScriptLine, SkeletonFork, TheaterSkeleton } from './theaterLogic'

export interface TheaterCastSlot {
  roleId: string
  displayName: string
}

export interface TheaterCastConfig {
  castA: TheaterCastSlot
  castB: TheaterCastSlot
  /** Canonical name in the official skeleton (cast A). */
  canonicalA: string
  /** Canonical name in the official skeleton (cast B). */
  canonicalB: string
}

export const THEATER_CAST_STORAGE_KEY = 'oclive.theater.cast.v1'

export const DEFAULT_THEATER_CAST_CONFIG: TheaterCastConfig = {
  castA: { roleId: 'mumu', displayName: '木木' },
  castB: { roleId: '枫侵月', displayName: '枫侵月' },
  canonicalA: '木木',
  canonicalB: '枫侵月',
}

/** Alias for plan/docs; same object as {@link DEFAULT_THEATER_CAST_CONFIG}. */
export const DEFAULT_CAST_CONFIG = DEFAULT_THEATER_CAST_CONFIG

export type CastTier = 'default' | 'applied'

export function isDefaultCastConfig(config: TheaterCastConfig): boolean {
  return config.castA.roleId === DEFAULT_THEATER_CAST_CONFIG.castA.roleId
    && config.castB.roleId === DEFAULT_THEATER_CAST_CONFIG.castB.roleId
}

export function resolveCastTier(config: TheaterCastConfig): CastTier {
  return isDefaultCastConfig(config) ? 'default' : 'applied'
}

/** One official example role and one custom role (mixed cast). */
export function isHybridCast(config: TheaterCastConfig): boolean {
  const aOfficial = config.castA.roleId === DEFAULT_THEATER_CAST_CONFIG.castA.roleId
  const bOfficial = config.castB.roleId === DEFAULT_THEATER_CAST_CONFIG.castB.roleId
  return (aOfficial && !bOfficial) || (!aOfficial && bOfficial)
}

function cloneSkeleton(sk: TheaterSkeleton): TheaterSkeleton {
  return {
    ...sk,
    cast: {
      a: { ...sk.cast.a },
      b: { ...sk.cast.b },
    },
    beats: sk.beats.map(b => ({ ...b })),
    forks: Object.fromEntries(
      Object.entries(sk.forks).map(([key, entries]) => [
        key,
        entries?.map(f => ({
          ...f,
          patchLines: f.patchLines.map(p => ({ ...p })),
        })),
      ]),
    ) as TheaterSkeleton['forks'],
  }
}

/** Ordered name pairs (longer source first) for global replace in dialogue text. */
function nameReplacePairs(
  canonicalA: string,
  canonicalB: string,
  newA: string,
  newB: string,
): Array<[string, string]> {
  const pairs: Array<[string, string]> = [
    [canonicalA, newA],
    [canonicalB, newB],
  ]
  return pairs.sort((a, b) => b[0].length - a[0].length)
}

function replaceNamesInText(text: string, pairs: Array<[string, string]>): string {
  let out = text
  for (const [from, to] of pairs) {
    if (from && from !== to)
      out = out.split(from).join(to)
  }
  return out
}

function swapLineNames(
  line: ScriptLine,
  pairs: Array<[string, string]>,
  canonicalA: string,
  canonicalB: string,
  newA: string,
  newB: string,
): ScriptLine {
  let name = line.name
  if (name === canonicalA)
    name = newA
  else if (name === canonicalB)
    name = newB
  else
    name = replaceNamesInText(name, pairs)

  return {
    ...line,
    name,
    text: replaceNamesInText(line.text, pairs),
    stageHint: line.stageHint
      ? replaceNamesInText(line.stageHint, pairs)
      : undefined,
  }
}

/** Replace official canonical names in beat lines (name / text / stageHint). */
export function swapCanonicalNamesInBeats(
  beats: ScriptLine[],
  canonicalA: string,
  canonicalB: string,
  newA: string,
  newB: string,
): ScriptLine[] {
  const pairs = nameReplacePairs(canonicalA, canonicalB, newA, newB)
  return beats.map(line => swapLineNames(line, pairs, canonicalA, canonicalB, newA, newB))
}

/** Replace canonical names inside fork patch lines. */
export function swapCanonicalNamesInForks(
  forks: TheaterSkeleton['forks'],
  canonicalA: string,
  canonicalB: string,
  newA: string,
  newB: string,
): TheaterSkeleton['forks'] {
  const pairs = nameReplacePairs(canonicalA, canonicalB, newA, newB)
  const out: TheaterSkeleton['forks'] = {}
  for (const [chipId, entries] of Object.entries(forks)) {
    if (!entries?.length)
      continue
    out[chipId as PokeChipId] = entries.map((fork: SkeletonFork) => ({
      ...fork,
      patchLines: fork.patchLines.map(line =>
        swapLineNames(line, pairs, canonicalA, canonicalB, newA, newB),
      ),
    }))
  }
  return out
}

/** Bind user cast config onto a canonical skeleton (does not mutate input). */
export function bindCastToSkeleton(
  sk: TheaterSkeleton,
  config: TheaterCastConfig,
): TheaterSkeleton {
  const result = cloneSkeleton(sk)
  result.cast.a.roleId = config.castA.roleId
  result.cast.a.name = config.castA.displayName
  result.cast.b.roleId = config.castB.roleId
  result.cast.b.name = config.castB.displayName
  result.beats = swapCanonicalNamesInBeats(
    result.beats,
    config.canonicalA,
    config.canonicalB,
    config.castA.displayName,
    config.castB.displayName,
  )
  result.forks = swapCanonicalNamesInForks(
    result.forks,
    config.canonicalA,
    config.canonicalB,
    config.castA.displayName,
    config.castB.displayName,
  )
  return result
}

function isValidSlot(raw: unknown): raw is TheaterCastSlot {
  if (!raw || typeof raw !== 'object')
    return false
  const s = raw as TheaterCastSlot
  return typeof s.roleId === 'string' && s.roleId.trim() !== ''
    && typeof s.displayName === 'string' && s.displayName.trim() !== ''
}

export function normalizeTheaterCastConfig(raw: unknown): TheaterCastConfig {
  if (!raw || typeof raw !== 'object')
    return { ...DEFAULT_THEATER_CAST_CONFIG }
  const o = raw as Partial<TheaterCastConfig>
  if (!isValidSlot(o.castA) || !isValidSlot(o.castB))
    return { ...DEFAULT_THEATER_CAST_CONFIG }
  return {
    castA: {
      roleId: o.castA.roleId.trim(),
      displayName: o.castA.displayName.trim(),
    },
    castB: {
      roleId: o.castB.roleId.trim(),
      displayName: o.castB.displayName.trim(),
    },
    canonicalA: typeof o.canonicalA === 'string' && o.canonicalA.trim()
      ? o.canonicalA.trim()
      : DEFAULT_THEATER_CAST_CONFIG.canonicalA,
    canonicalB: typeof o.canonicalB === 'string' && o.canonicalB.trim()
      ? o.canonicalB.trim()
      : DEFAULT_THEATER_CAST_CONFIG.canonicalB,
  }
}

export function getTheaterCastConfig(): TheaterCastConfig {
  try {
    const raw = localStorage.getItem(THEATER_CAST_STORAGE_KEY)
    if (raw == null || raw === '')
      return { ...DEFAULT_THEATER_CAST_CONFIG }
    return normalizeTheaterCastConfig(JSON.parse(raw))
  }
  catch {
    return { ...DEFAULT_THEATER_CAST_CONFIG }
  }
}

export function setTheaterCastConfig(config: TheaterCastConfig): void {
  try {
    localStorage.setItem(THEATER_CAST_STORAGE_KEY, JSON.stringify(config))
  }
  catch {
    /* ignore quota / private mode */
  }
}

/** Resolve display names from role list when saved config lacks them. */
export function enrichCastConfigFromRoles(
  config: TheaterCastConfig,
  roles: Array<{ id: string, name: string }>,
): TheaterCastConfig {
  const findName = (roleId: string, fallback: string) =>
    roles.find(r => r.id === roleId)?.name ?? fallback
  return {
    ...config,
    castA: {
      roleId: config.castA.roleId,
      displayName: findName(config.castA.roleId, config.castA.displayName),
    },
    castB: {
      roleId: config.castB.roleId,
      displayName: findName(config.castB.roleId, config.castB.displayName),
    },
  }
}
