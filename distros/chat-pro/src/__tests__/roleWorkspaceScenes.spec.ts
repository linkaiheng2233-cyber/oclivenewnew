import type { useMainShell } from '../composables/useMainShell'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope, ref } from 'vue'
import { useRoleWorkspace } from '../composables/useRoleWorkspace'

const mocks = vi.hoisted(() => ({ stats: vi.fn(), sessions: vi.fn(), caps: vi.fn() }))
vi.mock('@oclive/shared/api/chatStorage', () => ({
  getChatStorageStats: mocks.stats,
  listChatSessions: mocks.sessions,
  getChatStorageCapabilities: mocks.caps,
}))

describe('role library scene ownership', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    mocks.stats.mockResolvedValue([{ role_id: 'doro', scenes: [{ scene_id: 'attic', last_active: '2026-10-25' }] }])
    mocks.sessions.mockResolvedValue([{ role_id: 'doro', scene_id: 'attic', session_id: 'doro', message_count: 2 }])
    mocks.caps.mockResolvedValue({ supports_chat_threads: true })
  })

  it('shows a role’s actual latest scene instead of assuming every role uses home', async () => {
    const scope = effectScope()
    const shell = {
      roleStore: { currentRoleId: '', roles: [{ id: 'doro', name: 'Doro' }], roleInfo: { interactionMode: 'immersive' } },
      uiStore: { sceneId: 'home' },
      chatStore: { isLoading: false, chatSessionKey: () => 'doro' },
      startupState: ref('loading'),
    } as unknown as ReturnType<typeof useMainShell>
    const workspace = scope.run(() => useRoleWorkspace(shell))!
    await workspace.refreshLibrary()
    expect(mocks.sessions).toHaveBeenCalledWith('doro', 'attic', 50, 0)
    expect(workspace.recent.doro).toHaveLength(1)
    workspace.openLibrary()
    await vi.waitFor(() => expect(mocks.sessions).toHaveBeenCalledTimes(2))
    expect(mocks.sessions.mock.calls.every(call => call[1] === 'attic')).toBe(true)
    scope.stop()
  })
})
