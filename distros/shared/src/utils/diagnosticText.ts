/** Display/export-only redaction. Original errors and persisted logs remain untouched. */
export function redactDiagnosticText(value: string): string {
  return value
    .replace(/\u001B\[[\d;]*m/g, '')
    // Quoted values may contain spaces/escaped quotes; bare Basic/Bearer values
    // end at the credential. Never let a blank header consume the next log line.
    .replace(/(\b(?:proxy-)?authorization["']?[ \t]*[:=][ \t]*)("(?:\\.|[^"\\\r\n])*"|'(?:\\.|[^'\\\r\n])*'|(?:Basic|Bearer)[ \t]+[^\s"',;&}]+|\[redacted\]|[^ \t\r\n;}][^\r\n;}]*)/gi, (_match, prefix: string, credential: string) => {
      const quote = /^["']/.test(credential) ? credential[0] : ''
      return `${prefix}${quote}[redacted]${quote}`
    })
    .replace(/(\bBearer\s+)[\w.+/=-]+/gi, '$1[redacted]')
    .replace(/((?:api[_-]?key|token)["']?\s*[:=]\s*["']?)[^\s"',;&}]+/gi, '$1[redacted]')
    .replace(/\bsk-[\w-]{12,}/gi, '[redacted]')
    .replace(/(https?:\/\/)[^\s/@][^\s/:@]*:[^\s/@]+@/gi, '$1[redacted]@')
}
