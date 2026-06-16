export type TheaterCast = 'a' | 'b'

export type TheaterCastSide = 'left' | 'right'

export type TheaterStageState = 'playing' | 'idle' | 'patching'

export type TheaterSourceKind = 'pregen' | 'local' | 'cloud'

export type PokeChipId = 'tea' | 'late' | 'biteTongue' | 'nickname' | 'personality'

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

export interface TheaterSkeleton {
  scene: string
  sceneId?: string
  cast: {
    a: CastEntry
    b: CastEntry
  }
  beats: ScriptLine[]
  forks: Partial<Record<PokeChipId, SkeletonFork[]>>
}

export interface PokeChipDef {
  id: PokeChipId
  emoji: string
  labelKey: string
  weight: 'high' | 'neutral'
  /** Director's intent fed to the LLM so the rewrite has real dramatic tension. */
  dramaSeed: string
}

export const THEATER_POKE_CHIPS: PokeChipDef[] = [
  {
    id: 'tea',
    emoji: '🍵',
    labelKey: 'theater.poke.tea',
    weight: 'high',
    dramaSeed: '把"喝下一碗苦中药"这件苦差事，变成两人拌嘴的笑料：一方嫌弃抗拒，另一方半哄半逼，制造嫌弃与无奈的反差。',
  },
  {
    id: 'late',
    emoji: '⏰',
    labelKey: 'theater.poke.late',
    weight: 'high',
    dramaSeed: '突然发现快要迟到，时间压力骤升，让两人手忙脚乱、语速变快、互相催促，节奏陡然紧张起来。',
  },
  {
    id: 'biteTongue',
    emoji: '👅',
    labelKey: 'theater.poke.biteTongue',
    weight: 'high',
    dramaSeed: '吃早饭时冷不丁咬到舌头，一个突发小意外打破平静：先是吃痛慌乱，再被对方关心查看，最后窘迫害羞，情绪起伏明显。',
  },
  {
    id: 'nickname',
    emoji: '😼',
    labelKey: 'theater.poke.nickname',
    weight: 'high',
    dramaSeed: '冷不防甩出一个出其不意的新称呼撩拨关系，让对方先愣神、再追问，引出害羞与微妙的暧昧拉扯。',
  },
]

export const SKELETON_URL = '/theater/breakfast.skeleton.json'

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
  sceneId: 'home',
  cast: {
    a: { roleId: 'mumu', name: '木木' },
    b: { roleId: '枫侵月', name: '枫侵月' },
  },
  beats: FALLBACK_OPENING_BEATS,
  forks: {},
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
