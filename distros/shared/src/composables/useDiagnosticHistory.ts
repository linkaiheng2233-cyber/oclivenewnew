import { redactDiagnosticText } from '@oclive/shared/utils/diagnosticText'
import { readonly, ref } from 'vue'

export interface DiagnosticEntry {
  id: number
  at: number
  level: 'error' | 'warning'
  source: string
  code?: string
  summary: string
  details: string
}

const entries = ref<DiagnosticEntry[]>([])
let nextId = 0
const MAX_ENTRIES = 60
const MAX_DETAIL_CHARS = 12000

/** Bounded session-only diagnostics. Redacts common credentials and never auto-saves history. */
export function recordDiagnostic(event: Omit<DiagnosticEntry, 'id' | 'at'>): void {
  const at = Date.now()
  const summary = redactDiagnosticText(event.summary).slice(0, 500)
  const details = redactDiagnosticText(event.details).slice(0, MAX_DETAIL_CHARS)
  const previous = entries.value[0]
  // The API reporter and an action catch may surface the same failure in the same tick.
  if (previous && at - previous.at < 1000
    && (details ? previous.details === details : previous.summary === summary)) {
    return
  }
  entries.value.unshift({ ...event, summary, details, at, id: ++nextId })
  entries.value.length = Math.min(entries.value.length, MAX_ENTRIES)
}

export function useDiagnosticHistory() {
  return { entries: readonly(entries) }
}
