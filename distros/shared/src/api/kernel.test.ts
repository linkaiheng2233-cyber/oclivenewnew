import type { RoleInfo } from './role'
import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { invoke } from '@tauri-apps/api/core'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setErrorReporter } from './helpers'
import { fetchRoleSnapshot } from './kernel'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@oclive/shared/composables/useDiagnosticHistory', () => ({ recordDiagnostic: vi.fn() }))

const info = {
  role_id: 'one',
  current_favorability: 37,
  current_emotion: 'happy',
  relation_state: 'friend',
  display_metrics: { favor: 38, traits: [], relation_summary: 'close' },
  personality_source: 'vector',
  current_scene: 'home',
  user_presence_scene: 'home',
} as RoleInfo

describe('authenticated background role snapshot', () => {
  beforeEach(async () => {
    vi.mocked(invoke).mockResolvedValue(info)
    await fetchRoleSnapshot('one') // Reset failure suppression through a successful projection.
    vi.clearAllMocks()
  })
  afterEach(() => {
    setErrorReporter(null)
    vi.restoreAllMocks()
  })

  it('uses the existing native command without exposing a token or fetching HTTP', async () => {
    const fetch = vi.spyOn(globalThis, 'fetch')
    const result = await fetchRoleSnapshot('one', 'home')
    expect(invoke).toHaveBeenCalledWith('get_role_info', { req: { role_id: 'one', session_id: null } })
    expect(fetch).not.toHaveBeenCalled()
    expect(result).toMatchObject({ role_id: 'one', portrait_emotion: 'happy', display_metrics: info.display_metrics })
  })

  it('records a redacted failure once until recovery, without foreground error toasts', async () => {
    const reporter = vi.fn()
    setErrorReporter(reporter)
    vi.mocked(invoke).mockRejectedValue('network failed\nAuthorization: Bearer private-fixture-value')
    expect(await fetchRoleSnapshot('one')).toBeNull()
    expect(await fetchRoleSnapshot('one')).toBeNull()
    expect(recordDiagnostic).toHaveBeenCalledOnce()
    expect(vi.mocked(recordDiagnostic).mock.calls[0]?.[0].details).not.toContain('private-fixture-value')
    expect(reporter).not.toHaveBeenCalled()
    vi.mocked(invoke).mockResolvedValueOnce(info)
    await fetchRoleSnapshot('one')
    await fetchRoleSnapshot('one')
    expect(recordDiagnostic).toHaveBeenCalledTimes(2)
  })
})
