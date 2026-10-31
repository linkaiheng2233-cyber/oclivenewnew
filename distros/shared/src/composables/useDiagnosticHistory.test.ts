import { describe, expect, it, vi } from 'vitest'

describe('session diagnostic history', () => {
  it('redacts credentials, deduplicates the same surfaced failure, and bounds retention', async () => {
    vi.resetModules()
    const { recordDiagnostic, useDiagnosticHistory } = await import('./useDiagnosticHistory')
    const event = { level: 'error' as const, source: 'send_message', code: 'LLM_ERROR', summary: 'Model failed', details: 'api_key=secret-123 Bearer token-abc https://user:password@example.com/log' }
    recordDiagnostic(event)
    recordDiagnostic({ ...event, source: 'interface' })
    const { entries } = useDiagnosticHistory()
    expect(entries.value).toHaveLength(1)
    expect(entries.value[0].code).toBe('LLM_ERROR')
    expect(entries.value[0].details).not.toMatch(/secret-123|token-abc|password/)
    expect(entries.value[0].source).toBe('send_message')
    for (let i = 0; i < 80; i++)
      recordDiagnostic({ ...event, details: `diagnostic ${i} ${'x'.repeat(15000)}` })
    expect(entries.value).toHaveLength(60)
    expect(entries.value.every(entry => entry.details.length <= 12000)).toBe(true)
    expect(entries.value[0].details).toContain('diagnostic 79')
  })
})
