import type { useMainShell } from '../composables/useMainShell'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope, reactive, ref } from 'vue'
import { useRoleWorkspace } from '../composables/useRoleWorkspace'

const api = vi.hoisted(() => ({ stats: vi.fn(), sessions: vi.fn(), caps: vi.fn() }))
vi.mock('@oclive/shared/api/chatStorage', () => ({ getChatStorageStats: api.stats, listChatSessions: api.sessions, getChatStorageCapabilities: api.caps }))
const scopes: ReturnType<typeof effectScope>[] = []

function start() {
  const state = reactive({ role: '', thread: 'initial' })
  const onSend = vi.fn()
  const select = vi.fn(async () => {
    state.thread = 'new-thread'
  })
  const shell = {
    roleStore: { get currentRoleId() { return state.role }, roles: [{ id: 'mumu', name: 'Mumu' }], roleInfo: { interactionMode: 'pure_chat' }, loadRoles: vi.fn() },
    uiStore: { sceneId: 'home' },
    chatStore: { isLoading: false, cancelPendingSend: vi.fn(), chatSessionKey: () => `${state.role}:${state.thread}`, chatThreadId: () => 'b1a27229-4f57-4456-b37f-fbc41b026871', selectChatSession: select },
    roleSwitching: ref(false),
    roleActivationError: ref(''),
    startupState: ref('loading'),
    onSwitchRole: vi.fn(async (id: string) => {
      state.role = id
      return true
    }),
    onSend,
    t: (key: string) => key,
  } as unknown as ReturnType<typeof useMainShell>
  const scope = effectScope()
  scopes.push(scope)
  return { workspace: scope.run(() => useRoleWorkspace(shell))!, shell, state, onSend, select }
}

describe('recoverable role-first user paths', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    api.stats.mockResolvedValue([])
    api.sessions.mockResolvedValue([])
    api.caps.mockResolvedValue({ supports_chat_threads: true })
  })
  afterEach(() => {
    scopes.splice(0).forEach(scope => scope.stop())
  })

  it('can recheck a temporarily unavailable capability and enter instead of leaving a dead button', async () => {
    api.caps.mockRejectedValueOnce(new Error('kernel temporarily offline'))
    const { workspace } = start()
    await workspace.refreshLibrary()
    expect(workspace.capabilityError.value).toBeTruthy()
    expect(await workspace.enterRole('mumu')).toBe(true)
    expect(workspace.capabilityError.value).toBeNull()
    expect(workspace.page.value).toBe('chat')
  })

  it('retains an active conversation when creating the next one fails', async () => {
    const { workspace, select } = start()
    await workspace.refreshLibrary()
    await workspace.enterRole('mumu')
    select.mockRejectedValueOnce(new Error('storage offline'))
    expect(await workspace.enterRole('mumu', undefined, true)).toBe(false)
    expect(workspace.page.value).toBe('chat')
    expect(workspace.error.value).toBeInstanceOf(Error)
  })

  it('preserves storage diagnostics when opening recent records fails and permits retry', async () => {
    const failure = new Error('[DB_ERROR] isolated history unavailable')
    api.sessions.mockRejectedValueOnce(failure)
    const { workspace } = start()
    expect(await workspace.enterRole('mumu')).toBe(false)
    expect(workspace.error.value).toBe(failure)
    expect(workspace.recentErrors.mumu).toBe(failure)
    expect(await workspace.enterRole('mumu')).toBe(true)
    expect(workspace.recentErrors.mumu).toBeUndefined()
  })

  it('restores failed input without overwriting text already typed for the next message', async () => {
    const { workspace, onSend } = start()
    onSend.mockResolvedValue({ status: 'failed' })
    await workspace.submitMessage({ content: 'my unsent words' })
    expect(workspace.draft.value).toBe('my unsent words')
    workspace.draft.value = 'newer draft'
    await workspace.submitMessage({ content: 'older message' })
    expect(workspace.draft.value).toBe('newer draft')
  })

  it('never restores failed input into another role or thread', async () => {
    const { workspace, onSend, state } = start()
    let finish!: (value: { status: string }) => void
    onSend.mockImplementation(() => new Promise((resolve) => {
      finish = resolve
    }))
    const sending = workspace.submitMessage({ content: 'belongs to original thread' })
    state.thread = 'other-thread'
    finish({ status: 'failed' })
    await sending
    expect(workspace.draft.value).toBe('')
  })

  it('reports library refresh failure after import without an unhandled rejection', async () => {
    const { workspace, shell } = start()
    vi.mocked(shell.roleStore.loadRoles).mockRejectedValueOnce(new Error('role list failed'))
    await expect(workspace.onImported('mumu')).resolves.toBeUndefined()
    expect(workspace.page.value).toBe('roles')
    expect(workspace.error.value).toBeInstanceOf(Error)
  })
})
