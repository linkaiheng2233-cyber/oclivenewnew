import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { invoke } from '@tauri-apps/api/core'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setErrorReporter } from './helpers'
import { getGlobalOllamaModel, getLlmUserSettings } from './llmSettings'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@oclive/shared/composables/useDiagnosticHistory', () => ({ recordDiagnostic: vi.fn() }))

describe.each([
  { command: 'get_llm_user_settings', read: (signal: AbortSignal) => getLlmUserSettings('role-a', undefined, { signal }) },
  { command: 'get_global_ollama_model', read: (signal: AbortSignal) => getGlobalOllamaModel({ signal }) },
])('model read reporting ownership: $command', ({ read }) => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })
  afterEach(() => {
    setErrorReporter(null)
    vi.restoreAllMocks()
  })

  it.each([true, false])('reports failures only while the originating read is active (abandoned=$0)', async (abandoned) => {
    let fail!: (error: unknown) => void
    vi.mocked(invoke).mockReturnValueOnce(new Promise((_resolve, reject) => {
      fail = reject
    }))
    const reporter = vi.fn()
    setErrorReporter(reporter)
    const owner = new AbortController()
    const pending = read(owner.signal).catch(() => {})
    if (abandoned)
      owner.abort()
    fail(JSON.stringify({ code: 'IO_ERROR', message: 'late settings read failure' }))
    await pending
    expect(reporter).toHaveBeenCalledTimes(abandoned ? 0 : 1)
    expect(recordDiagnostic).toHaveBeenCalledTimes(abandoned ? 0 : 1)
  })
})
