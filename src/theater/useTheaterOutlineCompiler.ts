import type {
  TheaterBeat,
  TheaterOutline,
  TheaterOutlineBeat,
  TheaterSkeleton,
  TheaterSpeaker,
  TheaterTurnSpeaker,
} from './types'

const DEFAULT_DELAY_MS = 2200

export function createEmptyOutline(
  sceneId: string,
  title: string,
  roleA: string,
  roleB: string,
): TheaterOutline {
  return {
    schema_version: 1,
    scene_id: sceneId,
    title,
    role_a: roleA,
    role_b: roleB,
    beats: [
      { id: 'beat_1', speaker: 'a', summary: '' },
      { id: 'beat_2', speaker: 'b', summary: '' },
    ],
  }
}

/** Compile outline beats into a playable skeleton (Mode 2 → Mode 1). */
export function compileOutlineToSkeleton(outline: TheaterOutline): TheaterSkeleton {
  const beats: TheaterBeat[] = outline.beats.map((ob, index) => ({
    id: ob.id || `beat_${index + 1}`,
    speaker: normalizeOcSpeaker(ob.speaker),
    text: ob.summary.trim() || placeholderLine(ob.speaker, index),
    delay_ms: index === 0 ? 0 : DEFAULT_DELAY_MS,
  }))

  return {
    schema_version: 1,
    scene_id: outline.scene_id,
    title: outline.title,
    role_a: outline.role_a,
    role_b: outline.role_b,
    variables: {},
    impact_map: {},
    beats,
  }
}

/** Reverse: skeleton beats → editable outline draft. */
export function skeletonToOutline(skeleton: TheaterSkeleton): TheaterOutline {
  return {
    schema_version: 1,
    scene_id: skeleton.scene_id,
    title: skeleton.title,
    role_a: skeleton.role_a,
    role_b: skeleton.role_b,
    beats: skeleton.beats.map(b => ({
      id: b.id,
      speaker: b.speaker,
      summary: b.text,
    })),
  }
}

/** Mode 3 session transcript → outline for export. */
export function sessionToOutline(
  session: { scene_id: string, title: string, role_a: string, role_b: string, turns: Array<{ id: string, speaker: TheaterTurnSpeaker, text: string }> },
): TheaterOutline {
  return {
    schema_version: 1,
    scene_id: session.scene_id,
    title: session.title,
    role_a: session.role_a,
    role_b: session.role_b,
    beats: session.turns.map(t => ({
      id: t.id,
      speaker: t.speaker,
      summary: t.text,
    })),
  }
}

/** Freeze improv segment into skeleton (user lines become narrator-style A lines). */
export function sessionToSkeleton(
  session: { scene_id: string, title: string, role_a: string, role_b: string, turns: Array<{ id: string, speaker: TheaterTurnSpeaker, text: string }> },
): TheaterSkeleton {
  const outline = sessionToOutline(session)
  const normalized: TheaterOutline = {
    ...outline,
    beats: outline.beats.map(b => ({
      ...b,
      speaker: b.speaker === 'user' ? 'a' : b.speaker,
      summary: b.speaker === 'user' ? `（用户）${b.summary}` : b.summary,
    })),
  }
  return compileOutlineToSkeleton(normalized)
}

export function validateOutline(outline: TheaterOutline): string[] {
  const errors: string[] = []
  if (!outline.scene_id.trim()) {
    errors.push('scene_id required')
  }
  if (!outline.title.trim()) {
    errors.push('title required')
  }
  if (outline.beats.length === 0) {
    errors.push('at least one beat required')
  }
  const ids = new Set<string>()
  for (const beat of outline.beats) {
    if (!beat.id.trim()) {
      errors.push('beat id required')
    }
    if (ids.has(beat.id)) {
      errors.push(`duplicate beat id: ${beat.id}`)
    }
    ids.add(beat.id)
    if (!beat.summary.trim()) {
      errors.push(`beat ${beat.id}: summary required`)
    }
  }
  return errors
}

export function addOutlineBeat(outline: TheaterOutline): TheaterOutline {
  const nextIndex = outline.beats.length + 1
  const lastSpeaker = outline.beats.at(-1)?.speaker ?? 'b'
  const nextSpeaker: TheaterOutlineBeat['speaker']
    = lastSpeaker === 'a' ? 'b' : lastSpeaker === 'b' ? 'a' : 'a'
  return {
    ...outline,
    beats: [
      ...outline.beats,
      { id: `beat_${nextIndex}`, speaker: nextSpeaker, summary: '' },
    ],
  }
}

function normalizeOcSpeaker(speaker: TheaterTurnSpeaker): TheaterSpeaker {
  return speaker === 'b' ? 'b' : 'a'
}

function placeholderLine(speaker: TheaterTurnSpeaker, index: number): string {
  if (speaker === 'user') {
    return `（用户插话 ${index + 1}）`
  }
  return speaker === 'b' ? '……' : '……'
}
