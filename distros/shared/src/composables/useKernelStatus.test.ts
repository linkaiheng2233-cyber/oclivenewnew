// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { useRoleSnapshotPoll } from './useKernelStatus'

const state = vi.hoisted(() => ({
  role: { currentRoleId: 'one', minimalRoleActive: false, roleInfo: { favorability: 0, currentEmotion: 'neutral', relationState: '', currentScene: 'home', userPresenceScene: 'home' } },
  chat: { isLoading: false },
  kernel: { status: { healthy: true } },
  revision: 0,
  fetch: vi.fn(),
}))
vi.mock('@oclive/shared/api/kernel', () => ({ fetchRoleSnapshot: state.fetch }))
vi.mock('@oclive/shared/stores/roleStore', () => ({ useRoleStore: () => state.role }))
vi.mock('@oclive/shared/stores/chatStore', () => ({ useChatStore: () => state.chat }))
vi.mock('@oclive/shared/stores/kernelConnectionStore', () => ({ useKernelConnectionStore: () => state.kernel }))
vi.mock('@oclive/shared/stores/chatStoreSend', () => ({ getChatSendRevision: () => state.revision }))

const snapshot = (favor: number) => ({ role_id: 'one', current_favorability: favor, current_emotion: 'happy', relation_state: 'friend', portrait_emotion: 'happy', current_scene: 'home', user_presence_scene: 'home' })

describe('role snapshot freshness', () => {
  let wrapper: ReturnType<typeof mount> | undefined
  let poll: ReturnType<typeof useRoleSnapshotPoll>
  beforeEach(() => {
    state.role.currentRoleId = 'one'
    state.role.minimalRoleActive = false
    Object.assign(state.role.roleInfo, { favorability: 0, currentEmotion: 'neutral', currentScene: 'home', userPresenceScene: 'home' })
    state.chat.isLoading = false
    state.revision = 0
    state.fetch.mockReset().mockResolvedValue(snapshot(40))
    wrapper = mount(defineComponent({
      setup() {
        poll = useRoleSnapshotPoll()
        return () => null
      },
    }))
  })
  afterEach(() => {
    wrapper?.unmount()
  })

  it('applies a current snapshot', async () => {
    await poll.tick()
    expect(state.role.roleInfo.favorability).toBe(40)
  })

  it.each(['completed-turn', 'inflight-turn', 'role', 'scene', 'unmount'])('rejects a pending snapshot after %s', async (change) => {
    let finish!: (value: ReturnType<typeof snapshot>) => void
    state.fetch.mockImplementationOnce(() => new Promise((resolve) => {
      finish = resolve
    }))
    const pending = poll.tick()
    if (change === 'completed-turn') {
      state.revision += 1
      state.role.roleInfo.favorability = 90
    }
    if (change === 'inflight-turn')
      state.chat.isLoading = true
    if (change === 'role')
      state.role.currentRoleId = 'two'
    if (change === 'scene')
      state.role.roleInfo.userPresenceScene = 'library'
    if (change === 'unmount') {
      wrapper?.unmount()
      wrapper = undefined
    }
    finish(snapshot(10))
    await pending
    expect(state.role.roleInfo.favorability).toBe(change === 'completed-turn' ? 90 : 0)
  })

  it('keeps the newest poll when responses finish in reverse order', async () => {
    let finish!: (value: ReturnType<typeof snapshot>) => void
    state.fetch.mockImplementationOnce(() => new Promise((resolve) => {
      finish = resolve
    }))
    const old = poll.tick()
    await poll.tick()
    finish(snapshot(1))
    await old
    expect(state.role.roleInfo.favorability).toBe(40)
  })
})
