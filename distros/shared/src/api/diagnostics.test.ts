import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readSupportLogs, runEnvironmentDiagnostics, runEnvironmentRepair } from './diagnostics'
import { invokeWithFriendlyError } from './helpers'

vi.mock('./helpers', () => ({
  invokeWithFriendlyError: vi.fn(),
}))

const invokeMock = vi.mocked(invokeWithFriendlyError)

describe('api/diagnostics IPC contracts', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue(null)
  })

  it('keeps the environment check read-only command separate', async () => {
    await runEnvironmentDiagnostics()

    expect(invokeMock).toHaveBeenCalledWith('run_environment_diagnostics')
  })

  it('invokes the explicit repair command without hidden arguments', async () => {
    await runEnvironmentRepair()

    expect(invokeMock).toHaveBeenCalledWith('run_environment_repair')
  })

  it('reads a bounded log by file name without accepting a root path', async () => {
    await readSupportLogs()
    expect(invokeMock).toHaveBeenLastCalledWith('read_support_logs', { fileName: null })
    await readSupportLogs('oclive.2026-10-26.log')
    expect(invokeMock).toHaveBeenLastCalledWith('read_support_logs', { fileName: 'oclive.2026-10-26.log' })
  })
})
