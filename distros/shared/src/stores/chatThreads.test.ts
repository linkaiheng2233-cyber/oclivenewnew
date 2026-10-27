// @vitest-environment jsdom
import type { SessionMeta } from '@oclive/shared/api/chatStorage'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useChatStore } from './chatStore'
import { useRoleStore } from './roleStore'

const mocks = vi.hoisted(() => ({ fetch: vi.fn(), caps: vi.fn() }))
vi.mock('@oclive/shared/api/chatStorage', async original => ({
  ...await original<typeof import('@oclive/shared/api/chatStorage')>(),
  fetchChatMessages: mocks.fetch,
  getChatStorageCapabilities: mocks.caps,
}))

function session(role = 'mumu', id = role): SessionMeta {
  return { session_id: id, role_id: role, scene_id: 'home', created_at: '', updated_at: '', message_count: 2, last_message_snippet: 'hello' }
}
const threadId = 'b1a27229-4f57-4456-b37f-fbc41b026871'

describe('explicit transcript selection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    useRoleStore().currentRoleId = 'mumu'
    mocks.caps.mockResolvedValue({ supports_chat_threads: true })
    mocks.fetch.mockResolvedValue([{ id: 'a', session_id: 'mumu', sender: 'assistant', content: 'old transcript', created_at: '' }])
  })

  it('separates a new transcript without changing role memory or relationship state', async () => {
    const chat = useChatStore()
    const role = useRoleStore()
    role.roleInfo.favorability = 42
    role.roleInfo.currentUserRelation = 'family'
    await chat.selectChatSession('mumu', 'home', session())
    expect(chat.messagesForRoleScene('mumu', 'home')[0]?.content).toBe('old transcript')
    await chat.selectChatSession('mumu', 'home', null)
    expect(chat.messagesForRoleScene('mumu', 'home')).toEqual([])
    expect(chat.chatThreadId('mumu', 'home')).toMatch(/^[0-9a-f-]{36}$/)
    expect(role.currentRoleId).toBe('mumu')
    expect(role.roleInfo.favorability).toBe(42)
    expect(role.roleInfo.currentUserRelation).toBe('family')
    // The original record is fetched again, never replaced with the empty new transcript.
    await chat.selectChatSession('mumu', 'home', session())
    expect(chat.chatThreadId('mumu', 'home')).toBeNull()
    expect(chat.messagesForRoleScene('mumu', 'home')[0]?.content).toBe('old transcript')
  })

  it('rejects cross-role and cross-scene records before reading them', async () => {
    const chat = useChatStore()
    await expect(chat.selectChatSession('mumu', 'home', session('other'))).rejects.toThrow()
    await expect(chat.selectChatSession('mumu', 'school', session())).rejects.toThrow()
    expect(mocks.fetch).not.toHaveBeenCalled()
  })

  it('does not display another transcript when its read fails', async () => {
    const chat = useChatStore()
    await chat.selectChatSession('mumu', 'home', session())
    mocks.fetch.mockRejectedValueOnce(new Error('storage unavailable'))
    await expect(chat.selectChatSession('mumu', 'home', session('mumu', `${threadId}:chat:mumu`))).rejects.toThrow('storage unavailable')
    expect(chat.chatThreadId('mumu', 'home')).toBeNull()
    expect(chat.messagesForRoleScene('mumu', 'home')[0]?.content).toBe('old transcript')
  })

  it('ignores a delayed read after another transcript has been selected', async () => {
    let finish!: (value: unknown[]) => void
    mocks.fetch.mockImplementationOnce(() => new Promise((resolve) => {
      finish = resolve
    }))
    const chat = useChatStore()
    const pending = chat.selectChatSession('mumu', 'home', session())
    await vi.waitFor(() => expect(finish).toBeDefined())
    await chat.selectChatSession('mumu', 'home', null)
    const selected = chat.chatThreadId('mumu', 'home')
    finish([{ id: 'late', sender: 'assistant', content: 'late old reply' }])
    await pending
    expect(chat.chatThreadId('mumu', 'home')).toBe(selected)
    expect(chat.messagesForRoleScene('mumu', 'home')).toEqual([])
  })

  it('requires host support for new threads while retaining legacy record access', async () => {
    mocks.caps.mockResolvedValue({})
    const chat = useChatStore()
    await expect(chat.selectChatSession('mumu', 'home', null)).rejects.toThrow()
    expect(chat.chatThreadId('mumu', 'home')).toBeUndefined()
    await chat.selectChatSession('mumu', 'home', session())
    expect(chat.chatThreadId('mumu', 'home')).toBeNull()
  })

  it('does not let a delayed capability probe replace a newer selection', async () => {
    let finishCaps!: (value: { supports_chat_threads: boolean }) => void
    mocks.caps.mockImplementationOnce(() => new Promise((resolve) => {
      finishCaps = resolve
    }))
    const chat = useChatStore()
    const pending = chat.selectChatSession('mumu', 'home', session())
    await vi.waitFor(() => expect(finishCaps).toBeDefined())
    await chat.selectChatSession('mumu', 'home', null)
    const latest = chat.chatThreadId('mumu', 'home')
    finishCaps({ supports_chat_threads: true })
    await pending
    expect(chat.chatThreadId('mumu', 'home')).toBe(latest)
    expect(mocks.fetch).not.toHaveBeenCalled()
  })
})
