import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { invoke } from '@tauri-apps/api/core'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiInvokeError, invokeWithFriendlyError, setErrorReporter } from './helpers'
import { listOllamaModels } from './llmSettings'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@oclive/shared/composables/useDiagnosticHistory', () => ({ recordDiagnostic: vi.fn() }))

describe('ollama list error ownership', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  afterEach(() => {
    setErrorReporter(null)
    vi.restoreAllMocks()
  })

  it('retains a contextual, redacted diagnostic while the model form owns the notification', async () => {
    const reporter = vi.fn()
    setErrorReporter(reporter)
    vi.mocked(invoke).mockRejectedValueOnce(JSON.stringify({ code: 'LLM_ERROR', message: 'Failed to list models api_key=private-list-key' }))
    await expect(listOllamaModels('http://local.test', { errorSummary: 'Could not load this model list' })).rejects.toBeInstanceOf(ApiInvokeError)
    expect(recordDiagnostic).toHaveBeenCalledWith(expect.objectContaining({ source: 'list_ollama_models', code: 'LLM_ERROR', summary: 'Could not load this model list' }))
    expect(vi.mocked(recordDiagnostic).mock.calls[0][0].details).not.toContain('private-list-key')
    expect(reporter).not.toHaveBeenCalled()
  })

  it('does not report an abandoned list request through the global API error channel', async () => {
    const reporter = vi.fn()
    setErrorReporter(reporter)
    let reject!: (error: string) => void
    vi.mocked(invoke).mockReturnValueOnce(new Promise((_resolve, rejectPromise) => {
      reject = rejectPromise
    }))
    const controller = new AbortController()
    const raw = JSON.stringify({ code: 'LLM_ERROR', message: 'late failed request' })
    const pending = listOllamaModels('http://old.test', { signal: controller.signal })
    const assertion = expect(pending).rejects.toBe(raw)
    controller.abort()
    reject(raw)
    await assertion
    expect(recordDiagnostic).not.toHaveBeenCalled()
    expect(reporter).not.toHaveBeenCalled()
  })

  it('preserves the default global reporting behavior of other API calls', async () => {
    const reporter = vi.fn()
    setErrorReporter(reporter)
    vi.mocked(invoke).mockRejectedValueOnce(JSON.stringify({ code: 'LLM_ERROR', message: 'other action failed' }))
    await expect(invokeWithFriendlyError('other_action')).rejects.toBeInstanceOf(ApiInvokeError)
    expect(reporter).toHaveBeenCalledWith(expect.objectContaining({ code: 'LLM_ERROR' }))
    expect(recordDiagnostic).toHaveBeenCalledWith(expect.objectContaining({ source: 'other_action' }))
  })
})
