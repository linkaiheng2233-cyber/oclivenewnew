import type { CastEntry, ScriptLine, TheaterCast, TheaterCastSide } from './theaterLogic'

export interface CastPortraitSide {
  emotion: string
  active: boolean
}

export type CastPortraitMap = Record<TheaterCast, CastPortraitSide>

export interface CastRosterEntry {
  castId: TheaterCast
  roleId: string
  name: string
  side: TheaterCastSide
}

export interface CastColumnEntry extends CastRosterEntry {
  emotion: string
  active: boolean
  /** Poke / tweak preview: this cast is the primary subject of the hovered or running event. */
  eventAffected?: boolean
}

/** Build roster from skeleton cast entries (defaults a=left, b=right). */
export function buildCastRoster(cast: { a: CastEntry, b: CastEntry }): CastRosterEntry[] {
  return (['a', 'b'] as TheaterCast[]).map((castId) => {
    const entry = cast[castId]
    return {
      castId,
      roleId: entry.roleId,
      name: entry.name,
      side: entry.side ?? (castId === 'a' ? 'left' : 'right'),
    }
  })
}

/** Group roster entries by stage side with portrait state attached. */
export function rosterBySide(
  roster: CastRosterEntry[],
  portraitMap: CastPortraitMap,
): { left: CastColumnEntry[], right: CastColumnEntry[] } {
  const left: CastColumnEntry[] = []
  const right: CastColumnEntry[] = []
  for (const entry of roster) {
    const side = portraitMap[entry.castId]
    const col: CastColumnEntry = {
      ...entry,
      emotion: side.emotion,
      active: side.active,
    }
    if (entry.side === 'left')
      left.push(col)
    else
      right.push(col)
  }
  return { left, right }
}

/** Infer portrait emotion tag from optional line field + stage hint + dialogue. */
export function inferLineEmotion(line: ScriptLine): string {
  const explicit = line.emotion?.trim().toLowerCase()
  if (explicit)
    return explicit

  const hint = line.stageHint ?? ''
  const text = line.text
  const blob = `${hint}${text}`

  if (/[笑哈稳]|呵呵/.test(blob))
    return 'happy'
  if (/脸红|别过脸|耳朵|很小|[谢埋]|轻声|轻/.test(blob))
    return 'shy'
  if (/皱|灾难|惩罚|哼|闭嘴|啰嗦|捂嘴|舌头/.test(blob))
    return 'angry'
  if (/[弹猛快跑拽]/.test(blob))
    return 'excited'
  if (/[愣？?糟]|什么/.test(blob))
    return 'confused'
  if (/……|…/.test(text))
    return 'shy'
  return 'neutral'
}

/** Latest visible line per cast drives portrait emotion; last line is "active". */
export function resolveCastPortraitState(lines: ScriptLine[]): CastPortraitMap {
  const state: CastPortraitMap = {
    a: { emotion: 'neutral', active: false },
    b: { emotion: 'neutral', active: false },
  }
  let activeCast: TheaterCast | null = null

  for (const line of lines) {
    const emotion = inferLineEmotion(line)
    state[line.cast] = { ...state[line.cast], emotion }
    activeCast = line.cast
  }

  if (activeCast) {
    state.a.active = activeCast === 'a'
    state.b.active = activeCast === 'b'
  }

  return state
}
