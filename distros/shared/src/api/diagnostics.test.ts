import { beforeEach, describe, expect, it, vi } from 'vitest'

import { runEnvironmentDiagnostics, runEnvironmentRepair } from './diagnostics'
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
})
