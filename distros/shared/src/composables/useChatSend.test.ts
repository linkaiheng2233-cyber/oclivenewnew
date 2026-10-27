import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import { useChatSend } from './useChatSend'

const mocks = vi.hoisted(() => ({
  send: vi.fn(),
  exit: vi.fn(),
  refresh: vi.fn(),
  debug: vi.fn(),
  toast: vi.fn(),
  recorded: vi.fn(),
}))
vi.mock('@oclive/shared/stores/chatStore', () => ({ useChatStore: () => ({ sendMessage: mocks.send, sendAdultAction: mocks.exit }) }))
vi.mock('@oclive/shared/stores/debugStore', () => ({ useDebugStore: () => ({ loadDebugData: mocks.debug }) }))
vi.mock('@oclive/shared/stores/minimalRoleChatStore', () => ({ useMinimalRoleChatStore: () => ({}) }))
vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => ({ minimalRoleActive: false, roleInfo: { interactionMode: 'pure_chat' }, refreshRoleInfo: mocks.refresh }),
}))
vi.mock('@oclive/shared/stores/uiStore', () => ({ useUiStore: () => ({ sceneId: 'home' }) }))
vi.mock('@oclive/shared/composables/useNarrativeScene', () => ({ useNarrativeScene: () => ({ applyResolvedNarrativeScene: vi.fn() }) }))

describe('chat without optional debug refresh', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.send.mockResolvedValue({ reply: 'hello' })
    mocks.exit.mockResolvedValue({ reply: 'done' })
    mocks.refresh.mockResolvedValue(undefined)
    mocks.debug.mockRejectedValue(new Error('optional debug unavailable'))
  })

  it.each(['send', 'exit'])('keeps %s successful when hidden debug services are unavailable', async (action) => {
    const send = useChatSend({
      showToast: mocks.toast,
      t: ((key: string) => key) as never,
      chatInputRef: ref(null),
      clearSceneBarsBeforeSend: vi.fn(),
      offerSceneBarsAfterReply: vi.fn(),
      onTurnRecorded: mocks.recorded,
      debugDataEnabled: () => false,
    })
    if (action === 'send')
      await send.onSend({ content: 'fixture' })
    else
      await send.onAdultAction({ action: 'exit' })
    expect(mocks.refresh).toHaveBeenCalledOnce()
    expect(mocks.debug).not.toHaveBeenCalled()
    expect(mocks.toast).not.toHaveBeenCalled()
    if (action === 'send')
      expect(mocks.recorded).toHaveBeenCalledWith('fixture')
  })

  it('does not report an already completed reply as a failed send when refresh fails', async () => {
    mocks.refresh.mockRejectedValueOnce(new Error('role metadata offline'))
    const send = useChatSend({
      showToast: mocks.toast,
      t: ((key: string) => key) as never,
      chatInputRef: ref(null),
      clearSceneBarsBeforeSend: vi.fn(),
      offerSceneBarsAfterReply: vi.fn(),
      debugDataEnabled: () => false,
      onTurnRecorded: mocks.recorded,
    })
    expect(await send.onSend({ content: 'keep the delivered reply' })).toEqual({ status: 'sent' })
    expect(mocks.send).toHaveBeenCalledOnce()
    expect(mocks.toast).toHaveBeenCalledWith('warning', 'errorUi.refreshAfterReply', expect.stringContaining('role metadata offline'))
    expect(mocks.recorded).toHaveBeenCalledOnce()
  })

  it('returns a real send failure without retrying the request', async () => {
    mocks.send.mockRejectedValueOnce(new Error('[LLM_ERROR] no local model'))
    const send = useChatSend({
      showToast: mocks.toast,
      t: ((key: string) => key) as never,
      chatInputRef: ref(null),
      clearSceneBarsBeforeSend: vi.fn(),
      offerSceneBarsAfterReply: vi.fn(),
    })
    expect((await send.onSend({ content: 'retain this draft' }))?.status).toBe('failed')
    expect(mocks.send).toHaveBeenCalledOnce()
    expect(mocks.refresh).not.toHaveBeenCalled()
    expect(mocks.toast).toHaveBeenCalledWith('error', expect.any(String), expect.stringContaining('LLM_ERROR'))
  })
})
