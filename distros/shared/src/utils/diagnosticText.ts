/** Display/export-only redaction. Original errors and persisted logs remain untouched. */
export function redactDiagnosticText(value: string): string {
  return value
    .replace(/\u001B\[[\d;]*m/g, '')
    .replace(/(\bBearer\s+)[\w.+/=-]+/gi, '$1[redacted]')
    .replace(/((?:api[_-]?key|token|authorization)["']?\s*[:=]\s*["']?)[^\s"',;&}]+/gi, '$1[redacted]')
    .replace(/\bsk-[\w-]{12,}/gi, '[redacted]')
    .replace(/(https?:\/\/)[^\s/@][^\s/:@]*:[^\s/@]+@/gi, '$1[redacted]@')
}
