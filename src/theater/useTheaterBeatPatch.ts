import type {
  TheaterBeat,
  TheaterSkeleton,
  TheaterVariableState,
} from './types'

const OLLAMA_URL = import.meta.env.VITE_OCLIVE_OLLAMA_URL?.trim() || 'http://127.0.0.1:11434'
const PATCH_MODEL = import.meta.env.VITE_OCLIVE_THEATER_PATCH_MODEL?.trim() || 'qwen2.5:7b'

/** Dev baseline marks for V-THEATER-PERF-01 poke budget (probe → patch → first new line). */
export const THEATER_POKE_PERF_MARKS = {
  probeStart: 'theater-poke-probe-start',
  probeEnd: 'theater-poke-probe-end',
  patchStart: 'theater-poke-patch-start',
  patchEnd: 'theater-poke-patch-end',
  firstLine: 'theater-poke-first-line',
} as const

export interface TheaterPokePerfSample {
  probeMs: number | null
  patchMs: number | null
  firstLineMs: number | null
}

export function readTheaterPokePerfSample(): TheaterPokePerfSample {
  if (typeof performance === 'undefined' || typeof performance.getEntriesByName !== 'function') {
    return { probeMs: null, patchMs: null, firstLineMs: null }
  }
  const delta = (start: string, end: string) => {
    const entries = performance.getEntriesByName(end, 'mark')
    const startEntries = performance.getEntriesByName(start, 'mark')
    if (entries.length === 0 || startEntries.length === 0) {
      return null
    }
    return Math.round(entries[entries.length - 1].startTime - startEntries[startEntries.length - 1].startTime)
  }
  return {
    probeMs: delta(THEATER_POKE_PERF_MARKS.probeStart, THEATER_POKE_PERF_MARKS.probeEnd),
    patchMs: delta(THEATER_POKE_PERF_MARKS.patchStart, THEATER_POKE_PERF_MARKS.patchEnd),
    firstLineMs: delta(THEATER_POKE_PERF_MARKS.patchEnd, THEATER_POKE_PERF_MARKS.firstLine),
  }
}

export function markTheaterPokeFirstLine(): void {
  if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
    performance.mark(THEATER_POKE_PERF_MARKS.firstLine)
  }
}

function cloneBeats(beats: TheaterBeat[]): TheaterBeat[] {
  return beats.map(b => ({ ...b }))
}

export function defaultVariableState(skeleton: TheaterSkeleton): TheaterVariableState {
  const out: TheaterVariableState = {}
  for (const [key, def] of Object.entries(skeleton.variables)) {
    out[key] = def.default
  }
  return out
}

export function resolveImpactedBeatIds(
  skeleton: TheaterSkeleton,
  varId: string,
): string[] {
  return skeleton.impact_map[varId] ?? []
}

/**
 * Local Ollama patch for impacted beats only. On failure returns original texts (graceful degrade).
 */
export async function patchTheaterBeats(
  skeleton: TheaterSkeleton,
  beats: TheaterBeat[],
  beatIds: string[],
  variables: TheaterVariableState,
  locale: 'zh' | 'en',
): Promise<{ beats: TheaterBeat[], patched: boolean }> {
  if (beatIds.length === 0) {
    return { beats, patched: false }
  }

  const idSet = new Set(beatIds)
  const targets = beats.filter(b => idSet.has(b.id))
  if (targets.length === 0) {
    return { beats, patched: false }
  }

  const hints = Object.entries(variables)
    .map(([k, v]) => `${k}=${String(v)}`)
    .join('; ')
  const patchRules = beatIds
    .map(id => skeleton.patch_hints?.[Object.keys(skeleton.impact_map).find(k => skeleton.impact_map[k]?.includes(id)) ?? ''] ?? '')
    .filter(Boolean)
    .join('\n')

  const system = locale === 'zh'
    ? '你是早饭场景剧本局部改写器。只改写用户给出的台词，保持角色口吻与场景，不要加 markdown，不要解释。'
    : 'You rewrite breakfast-scene dialogue lines only. Keep character voice; no markdown or explanation.'

  const userBlock = [
    `场景：${skeleton.title}`,
    `变量：${hints}`,
    patchRules ? `改写提示：${patchRules}` : '',
    '待改写（JSON 数组，保留 id，只改 text）：',
    JSON.stringify(targets.map(t => ({ id: t.id, speaker: t.speaker, text: t.text }))),
  ].filter(Boolean).join('\n')

  if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
    performance.mark(THEATER_POKE_PERF_MARKS.patchStart)
  }

  try {
    const res = await fetch(`${OLLAMA_URL.replace(/\/+$/, '')}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: AbortSignal.timeout(12000),
      body: JSON.stringify({
        model: PATCH_MODEL,
        stream: false,
        messages: [
          { role: 'system', content: system },
          { role: 'user', content: userBlock },
        ],
      }),
    })
    if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
      performance.mark(THEATER_POKE_PERF_MARKS.patchEnd)
    }
    if (!res.ok) {
      return { beats, patched: false }
    }
    const data = await res.json() as { message?: { content?: string } }
    const content = data?.message?.content?.trim()
    if (!content) {
      return { beats, patched: false }
    }
    const parsed = extractPatchedLines(content, targets)
    if (!parsed) {
      return { beats, patched: false }
    }
    const next = cloneBeats(beats)
    for (const row of parsed) {
      const idx = next.findIndex(b => b.id === row.id)
      if (idx >= 0 && row.text.trim()) {
        next[idx] = { ...next[idx], text: row.text.trim() }
      }
    }
    return { beats: next, patched: true }
  }
  catch {
    if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
      performance.mark(THEATER_POKE_PERF_MARKS.patchEnd)
    }
    return { beats, patched: false }
  }
}

function extractPatchedLines(
  content: string,
  fallback: TheaterBeat[],
): Array<{ id: string, text: string }> | null {
  const jsonMatch = content.match(/\[[\s\S]*\]/)
  if (jsonMatch) {
    try {
      const arr = JSON.parse(jsonMatch[0]) as Array<{ id?: string, text?: string }>
      if (Array.isArray(arr) && arr.every(r => r.id && typeof r.text === 'string')) {
        return arr.map(r => ({ id: r.id!, text: r.text! }))
      }
    }
    catch { /* fall through */ }
  }
  const lines = content.split(/\r?\n/).map(l => l.trim()).filter(Boolean)
  if (lines.length >= fallback.length) {
    return fallback.map((b, i) => ({ id: b.id, text: lines[i] ?? b.text }))
  }
  return null
}

export async function probeOllamaAvailable(): Promise<boolean> {
  if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
    performance.mark(THEATER_POKE_PERF_MARKS.probeStart)
  }
  try {
    const res = await fetch(`${OLLAMA_URL.replace(/\/+$/, '')}/api/tags`, {
      signal: AbortSignal.timeout(2000),
    })
    const ok = res.ok
    if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
      performance.mark(THEATER_POKE_PERF_MARKS.probeEnd)
    }
    return ok
  }
  catch {
    if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
      performance.mark(THEATER_POKE_PERF_MARKS.probeEnd)
    }
    return false
  }
}
