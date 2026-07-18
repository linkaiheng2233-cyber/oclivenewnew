import type { TheaterScenePresetId } from './theaterSceneCatalog'
import { getPokeChipsForPreset, getTheaterScenePreset, LEGACY_BREAKFAST_SKELETON_URL, THEATER_RUNTIME_SCENE_ID } from './theaterSceneCatalog'

export type TheaterCast = 'a' | 'b'

export type TheaterCastSide = 'left' | 'right'

export type TheaterStageState = 'playing' | 'idle' | 'patching'

export type TheaterSourceKind = 'pregen' | 'local' | 'cloud'

export type PokeChipId
  = | 'tea'
    | 'late'
    | 'biteTongue'
    | 'nickname'
    | 'personality'
    | 'buyMilk'
    | 'forgotMilk'
    | 'milkSoldOut'
    | 'milkLottery'
    | 'strayCat'
    | 'hitPole'
    | 'wrongWay'
    | 'sprainedAnkle'
    | 'drinkMilk'
    | 'insomnia'
    | 'midnightSnack'
    | 'thunderstorm'

export interface ScriptLine {
  id: string
  cast: TheaterCast
  name: string
  text: string
  stageHint?: string
  /** Portrait emotion tag (happy/shy/angry/…); inferred from hint when omitted. */
  emotion?: string
}

export interface SkeletonFork {
  chipId: PokeChipId
  insertAfterBeatId: string
  patchLines: ScriptLine[]
}

export interface CastEntry {
  roleId: string
  name: string
  side?: TheaterCastSide
}

export interface AppliedTweak {
  kind: 'chip' | 'custom'
  chipId?: PokeChipId
  dramaSeed: string
  insertAfterBeatId: string
  leadCast: TheaterCast
  anchorLines: ScriptLine[]
  patchLines: ScriptLine[]
}

export interface TheaterPairRelationDef {
  displayName: string
  promptHint: string
}

export interface TheaterSkeleton {
  scene: string
  sceneId?: string
  cast: {
    a: CastEntry
    b: CastEntry
  }
  beats: ScriptLine[]
  forks: Partial<Record<PokeChipId, SkeletonFork[]>>
  /** Default pair relation when user has not chosen (mode 1). */
  defaultPairRelation?: string
  /** Official pair-relation presets for cast_rewrite prompts. */
  pairRelations?: Partial<Record<string, TheaterPairRelationDef>>
}

export interface PokeChipDef {
  id: PokeChipId
  emoji: string
  labelKey: string
  weight: 'high' | 'neutral'
  /** Director's intent fed to the LLM so the rewrite has real dramatic tension. */
  dramaSeed: string
}

/** @deprecated Use `getPokeChipsForPreset` from `theaterSceneCatalog`. */
export const THEATER_POKE_CHIPS: PokeChipDef[] = getPokeChipsForPreset('breakfast')

export const SKELETON_URL = '/theater/scenes/breakfast.skeleton.json'

/** @deprecated Use `fetchSkeletonForPreset` with catalog preset id. */
export const LEGACY_SKELETON_URL = LEGACY_BREAKFAST_SKELETON_URL

/** Frontend race timeout for `generate_theater_scene` (kernel defaults to 25s). */
export const SCENE_GEN_TIMEOUT_MS = 30_000

export class SceneGenTimeoutError extends Error {
  constructor() {
    super('scene generation timeout')
    this.name = 'SceneGenTimeoutError'
  }
}

/** Rejects after `ms` with [`SceneGenTimeoutError`]. */
export function timeoutReject(ms: number): Promise<never> {
  return new Promise((_, reject) => {
    setTimeout(() => reject(new SceneGenTimeoutError()), ms)
  })
}

/** Embedded opening beats when skeleton JSON is unavailable (dev / offline). */
export const FALLBACK_OPENING_BEATS: ScriptLine[] = [
  { id: 'b1', cast: 'b', name: '枫侵月', text: '木木，粥还要不要温一下？今天外面有点凉。', stageHint: '把碗推过去' },
  { id: 'b2', cast: 'a', name: '木木', text: '……谁要你温了。我自己会吃。', stageHint: '别过脸' },
  { id: 'b3', cast: 'b', name: '枫侵月', text: '天气预报说下午可能要下雨，伞在玄关。' },
  { id: 'b4', cast: 'a', name: '木木', text: '知道了知道了，啰嗦。', stageHint: '扒粥' },
  { id: 'b5', cast: 'b', name: '枫侵月', text: '书包我放你椅子上了，作业本在上层。' },
  { id: 'b6', cast: 'a', name: '木木', text: '……谢、谢谢。', stageHint: '声音很小' },
  { id: 'b7', cast: 'b', name: '枫侵月', text: '别谢了，再不吃真要迟到了。', stageHint: '笑' },
  { id: 'b8', cast: 'a', name: '木木', text: '才、才没有要迟到！我很快的！', stageHint: '猛塞一口' },
  { id: 'b9', cast: 'b', name: '枫侵月', text: '慢点，别噎着。' },
  { id: 'b10', cast: 'b', name: '枫侵月', text: '对了，冰箱里有昨天切的水果，上学路上记得拿。' },
]

export const FALLBACK_SKELETON: TheaterSkeleton = {
  scene: 'breakfast',
  sceneId: THEATER_RUNTIME_SCENE_ID,
  cast: {
    a: { roleId: 'mumu', name: '木木' },
    b: { roleId: '枫侵月', name: '枫侵月' },
  },
  beats: FALLBACK_OPENING_BEATS,
  forks: {},
}

const FALLBACK_SUPERMARKET_BEATS: ScriptLine[] = [
  { id: 'b1', cast: 'b', name: '枫侵月', text: '购物车推这边，特价鸡蛋在冷藏柜那头。' },
  { id: 'b2', cast: 'a', name: '木木', text: '我不买鸡蛋。又不是我做饭。' },
  { id: 'b3', cast: 'b', name: '枫侵月', text: '你前天不是说周末想做蛋糕？' },
  { id: 'b4', cast: 'a', name: '木木', text: '……那是随口一说。' },
  { id: 'b5', cast: 'b', name: '枫侵月', text: '试吃区有新酸奶，就一小杯。' },
  { id: 'b6', cast: 'a', name: '木木', text: '不要。会胖。' },
  { id: 'b7', cast: 'b', name: '枫侵月', text: '一小杯不会胖。尝一口。' },
  { id: 'b8', cast: 'a', name: '木木', text: '……就一口。' },
  { id: 'b9', cast: 'b', name: '枫侵月', text: '结账队伍好长。你带钱包了吗？' },
  { id: 'b10', cast: 'a', name: '木木', text: '啊？！我、我没带……' },
]

const FALLBACK_WAY_HOME_BEATS: ScriptLine[] = [
  { id: 'b1', cast: 'b', name: '枫侵月', text: '路灯亮了，走内侧。' },
  { id: 'b2', cast: 'a', name: '木木', text: '用不着你管，路又不是你的。' },
  { id: 'b3', cast: 'b', name: '枫侵月', text: '两袋东西，你拎一个？' },
  { id: 'b4', cast: 'a', name: '木木', text: '不行，太重。' },
  { id: 'b5', cast: 'b', name: '枫侵月', text: '你手里那个是零食。' },
  { id: 'b6', cast: 'a', name: '木木', text: '零食也很重！' },
  { id: 'b7', cast: 'b', name: '枫侵月', text: '……行，我拿重的。' },
  { id: 'b8', cast: 'a', name: '木木', text: '我才不会。' },
  { id: 'b9', cast: 'b', name: '枫侵月', text: '公交还有两站。围巾系好。' },
  { id: 'b10', cast: 'a', name: '木木', text: '……知道了。多谢。' },
]

const FALLBACK_BEDTIME_BEATS: ScriptLine[] = [
  { id: 'b1', cast: 'b', name: '枫侵月', text: '你先洗脸，我去放水。' },
  { id: 'b2', cast: 'a', name: '木木', text: '凭什么我先？' },
  { id: 'b3', cast: 'b', name: '枫侵月', text: '因为你每次都吹头发，吹到浴室全是雾气。' },
  { id: 'b4', cast: 'a', name: '木木', text: '那是吹风机的问题！' },
  { id: 'b5', cast: 'b', name: '枫侵月', text: '好吧，你先洗头发。别跟我抢毛巾。' },
  { id: 'b6', cast: 'a', name: '木木', text: '谁要抢你的……' },
  { id: 'b7', cast: 'b', name: '枫侵月', text: '牙刷挤好了，在杯子里。' },
  { id: 'b8', cast: 'a', name: '木木', text: '……多管闲事。' },
  { id: 'b9', cast: 'b', name: '枫侵月', text: '晚安之前还有一句——今天辛苦了。' },
  { id: 'b10', cast: 'a', name: '木木', text: '……你也是。快睡啦，啰嗦。' },
]

const FALLBACK_SKELETON_BY_PRESET: Record<TheaterScenePresetId, TheaterSkeleton> = {
  breakfast: FALLBACK_SKELETON,
  supermarket: {
    scene: 'supermarket',
    sceneId: THEATER_RUNTIME_SCENE_ID,
    cast: FALLBACK_SKELETON.cast,
    beats: FALLBACK_SUPERMARKET_BEATS,
    forks: {},
  },
  way_home: {
    scene: 'way_home',
    sceneId: THEATER_RUNTIME_SCENE_ID,
    cast: FALLBACK_SKELETON.cast,
    beats: FALLBACK_WAY_HOME_BEATS,
    forks: {},
  },
  bedtime: {
    scene: 'bedtime',
    sceneId: THEATER_RUNTIME_SCENE_ID,
    cast: FALLBACK_SKELETON.cast,
    beats: FALLBACK_BEDTIME_BEATS,
    forks: {},
  },
}

export function fallbackSkeletonForPreset(presetId: TheaterScenePresetId): TheaterSkeleton {
  const sk = FALLBACK_SKELETON_BY_PRESET[presetId]
  return {
    ...sk,
    cast: {
      a: { ...sk.cast.a },
      b: { ...sk.cast.b },
    },
    beats: cloneScriptLines(sk.beats),
    forks: { ...sk.forks },
  }
}

export async function fetchSkeletonForPreset(presetId: TheaterScenePresetId): Promise<TheaterSkeleton> {
  const preset = getTheaterScenePreset(presetId)
  const urls = presetId === 'breakfast'
    ? [preset.skeletonPath, LEGACY_BREAKFAST_SKELETON_URL]
    : [preset.skeletonPath]
  for (const url of urls) {
    try {
      const res = await fetch(url)
      if (!res.ok)
        continue
      return validateSkeleton(await res.json())
    }
    catch {
      /* try next url */
    }
  }
  console.warn(`[theater] skeleton fetch failed for ${presetId}, using embedded fallback`)
  return fallbackSkeletonForPreset(presetId)
}

export function cloneScriptLines(lines: ScriptLine[]): ScriptLine[] {
  return lines.map(line => ({ ...line }))
}

export function insertForkLines(
  lines: ScriptLine[],
  insertAfterBeatId: string,
  patchLines: ScriptLine[],
): ScriptLine[] {
  const idx = lines.findIndex(line => line.id === insertAfterBeatId)
  if (idx < 0)
    return [...lines, ...patchLines]
  const next = [...lines]
  next.splice(idx + 1, 0, ...patchLines)
  return next
}

/** Apply tweaks in order onto base opening beats. */
export function buildWorkingScript(
  baseBeats: ScriptLine[],
  applied: AppliedTweak[],
): ScriptLine[] {
  let working = cloneScriptLines(baseBeats)
  for (const tweak of applied)
    working = insertForkLines(working, tweak.insertAfterBeatId, tweak.patchLines)
  return working
}

/** Beats in the base script that follow the mid insert anchor (ripple zone). */
export function beatsAfterInsert(
  baseBeats: ScriptLine[],
  insertAfterBeatId: string,
): ScriptLine[] {
  const idx = baseBeats.findIndex(b => b.id === insertAfterBeatId)
  if (idx < 0)
    return []
  return baseBeats.slice(idx + 1)
}

/** Default mid-anchor for tweaks; falls back to skeleton midpoint. */
export function defaultInsertAnchor(skeleton: TheaterSkeleton): string {
  const forks = Object.values(skeleton.forks).flat()
  const fromFork = forks[0]?.insertAfterBeatId
  if (fromFork)
    return fromFork
  const mid = Math.max(0, Math.floor(skeleton.beats.length / 2) - 1)
  return skeleton.beats[mid]?.id ?? skeleton.beats[skeleton.beats.length - 1]!.id
}

export function pickCanFork(
  skeleton: TheaterSkeleton,
  chipId: PokeChipId,
): SkeletonFork | null {
  const entries = skeleton.forks[chipId]
  if (!entries?.length)
    return null
  return entries[0] ?? null
}

/** Primary cast affected by a poke chip (first fork patch speaker). */
export function resolveChipLeadCast(
  skeleton: TheaterSkeleton,
  chipId: PokeChipId,
): TheaterCast | null {
  const fork = pickCanFork(skeleton, chipId)
  const lead = fork?.patchLines[0]
  return lead?.cast ?? null
}

export function playbackDone(visibleCount: number, total: number): boolean {
  return visibleCount >= total && total > 0
}

export function nextVisibleCount(visibleCount: number, total: number): number {
  return Math.min(visibleCount + 1, total)
}

export interface PatchPromptInput {
  /** Human-readable chip label (localized), e.g. "早饭咬到舌头". */
  chipLabel: string
  /** Director's dramatic intent for this beat. */
  dramaSeed: string
  /** The character who speaks this patch. */
  speakerName: string
  /** The scene partner reacting to the speaker. */
  partnerName: string
  contextLines: ScriptLine[]
  anchorLines: ScriptLine[]
  maxLines?: number
}

export function buildPatchPrompt(input: PatchPromptInput): string {
  const {
    chipLabel,
    dramaSeed,
    speakerName,
    partnerName,
    contextLines,
    anchorLines,
    maxLines = 3,
  } = input
  const context = contextLines
    .map(line => `${line.name}：${line.text}`)
    .join('\n')
  const anchor = anchorLines
    .map(line => `${line.name}：${line.text}`)
    .join('\n')
  return [
    '【剧场即兴 · 导演指令】',
    `这是一幕双人日常戏。现在轮到「${speakerName}」开口，对手戏是「${partnerName}」。`,
    `观众刚刚按下了剧情转折「${chipLabel}」。`,
    `本场戏剧目标：${dramaSeed}`,
    '',
    '【演出要求】',
    `· 写出「${speakerName}」接下来的 1–${maxLines} 句台词，每句一行，格式：角色名：台词`,
    '· 至少一句带上动作或神态，单独成行用括号包住，例：(耳朵红了)',
    '· 要有"戏"：制造冲突、反差或情绪起伏，可顺带描写对方的反应，别平淡复述',
    '· 紧接上文语气，口语化、贴合人设；不要旁白、解说、JSON 或引号',
    '· 总字数 100 字以内',
    '',
    '【刚刚发生的对白】',
    context || '（无）',
    '',
    '【可参考的情绪走向（仅作灵感，请改写出新意，禁止照抄）】',
    anchor || '（自由发挥）',
  ].join('\n')
}

export function parsePatchReply(
  reply: string,
  fallbackCast: TheaterCast,
  fallbackName: string,
  chipId: PokeChipId | 'custom',
  lineIdPrefix: string,
  maxLines = 3,
): ScriptLine[] {
  const trimmed = reply.trim()
  if (!trimmed)
    return []

  const lines: ScriptLine[] = []
  const rawLines = trimmed.split(/\n+/).map(s => s.trim()).filter(Boolean)

  for (let i = 0; i < rawLines.length && lines.length < maxLines; i++) {
    const row = rawLines[i]!
    if (row.startsWith('(') && row.endsWith(')')) {
      const last = lines[lines.length - 1]
      if (last)
        last.stageHint = row.slice(1, -1)
      continue
    }
    const m = row.match(/^([^：:]+)[：:](.+)$/)
    if (m) {
      lines.push({
        id: `${lineIdPrefix}-${chipId}-${lines.length}`,
        cast: fallbackCast,
        name: m[1]!.trim(),
        text: m[2]!.trim(),
      })
    }
    else {
      lines.push({
        id: `${lineIdPrefix}-${chipId}-${lines.length}`,
        cast: fallbackCast,
        name: fallbackName,
        text: row,
      })
    }
  }
  return lines
}

export function validateSkeleton(data: unknown): TheaterSkeleton {
  if (!data || typeof data !== 'object')
    throw new Error('skeleton: not an object')
  const sk = data as TheaterSkeleton
  if (!sk.scene || !Array.isArray(sk.beats) || sk.beats.length < 1)
    throw new Error('skeleton: missing scene or beats')
  if (!sk.cast?.a?.roleId || !sk.cast?.b?.roleId)
    throw new Error('skeleton: missing cast')
  return sk
}
