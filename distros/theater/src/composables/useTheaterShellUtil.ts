import type { TheaterScriptLine, TheaterTweak } from '@oclive/shared/api/theater'
import type {
  AppliedTweak,
  PokeChipId,
  ScriptLine,
  TheaterCast,
  TheaterSkeleton,
  TheaterSourceKind,
} from './theater/theaterLogic'
import type { TheaterScenePresetId } from './theater/theaterSceneCatalog'
import { beatsAfterInsert } from './theater/theaterLogic'
import { getPokeChipsForPreset } from './theater/theaterSceneCatalog'

export const LINE_REVEAL_MS = 720
export const THINK_STEP_MS = 650
export const CAST_ADAPT_DONE_VISIBLE_MS = 1000
/** Two kernel attempts × cast_rewrite timeout (default 45s) + buffer. */
export const CAST_REWRITE_TIMEOUT_MS = 100_000

export interface PokeVariant {
  id: 'a' | 'b'
  patchLines: ScriptLine[]
  fullBeats: ScriptLine[]
  source: TheaterSourceKind
}

export interface PendingVariantContext {
  key: string
  tweaks: AppliedTweak[]
  insertAfterBeatId: string
}

export function extractPatchSegment(
  beats: ScriptLine[],
  insertAfterBeatId: string,
  baseBeats: ScriptLine[],
): ScriptLine[] {
  const tailIds = new Set(beatsAfterInsert(baseBeats, insertAfterBeatId).map(b => b.id))
  const anchorIdx = beats.findIndex(b => b.id === insertAfterBeatId)
  if (anchorIdx < 0)
    return []
  const tailIdx = beats.findIndex((b, i) => i > anchorIdx && tailIds.has(b.id))
  const end = tailIdx >= 0 ? tailIdx : beats.length
  return beats.slice(anchorIdx + 1, end)
}

export function pokeVariantKey(
  presetId: string,
  chipId: PokeChipId | 'custom',
  tweakIndex: number,
): string {
  return `${presetId}:${chipId}:${tweakIndex}`
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export function castName(sk: TheaterSkeleton, cast: TheaterCast): string {
  return cast === 'a' ? sk.cast.a.name : sk.cast.b.name
}

export function toScriptLineDto(line: ScriptLine): TheaterScriptLine {
  return {
    id: line.id,
    cast: line.cast,
    name: line.name,
    text: line.text,
    stage_hint: line.stageHint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

export function fromScriptLineDto(line: TheaterScriptLine): ScriptLine {
  return {
    id: line.id,
    cast: line.cast as TheaterCast,
    name: line.name,
    text: line.text,
    stageHint: line.stage_hint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

export function tweakToDto(
  tweak: AppliedTweak,
  translate: (key: string) => string,
  presetId: TheaterScenePresetId,
): TheaterTweak {
  let chipLabel: string | undefined
  if (tweak.kind === 'chip' && tweak.chipId) {
    const chip = getPokeChipsForPreset(presetId).find(c => c.id === tweak.chipId)
    chipLabel = chip ? translate(chip.labelKey) : tweak.chipId
  }
  else if (tweak.kind === 'custom') {
    chipLabel = translate('theater.poke.customLabel')
  }
  return {
    kind: tweak.kind,
    chip_label: chipLabel,
    drama_seed: tweak.dramaSeed,
    insert_after_beat_id: tweak.insertAfterBeatId,
    lead_cast: tweak.leadCast,
  }
}

export function mapFooterSource(source: string): TheaterSourceKind {
  if (source === 'cloud')
    return 'cloud'
  if (source === 'local')
    return 'local'
  return 'pregen'
}

export function beatsEqual(a: ScriptLine[], b: ScriptLine[]): boolean {
  if (a.length !== b.length)
    return false
  return a.every((line, i) => {
    const other = b[i]
    return other != null && line.id === other.id && line.text === other.text
  })
}
