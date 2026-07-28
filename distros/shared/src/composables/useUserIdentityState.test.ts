// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { useUserIdentityState } from './useUserIdentityState'

const mocks = vi.hoisted(() => ({
  getIdentity: vi.fn(),
  setSceneIdentity: vi.fn(),
  setIdentity: vi.fn(),
  clearAdult: vi.fn(),
  sendAdult: vi.fn(),
  roleStore: {
    currentRoleId: 'role',
    roleInfo: {
      identityBinding: 'per_scene',
    },
    refreshRoleInfo: vi.fn(),
  },
  adultStore: {
    sessionFor: vi.fn(() => ({
      active: true,
      voiceTextOnly: false,
      updatedAt: 1,
    })),
  },
  uiStore: {
    sceneId: 'home',
  },
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    getUserIdentityState: mocks.getIdentity,
    setSceneUserIdentity: mocks.setSceneIdentity,
    setUserIdentity: mocks.setIdentity,
  }
})

vi.mock('@oclive/shared/stores/adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))

vi.mock('@oclive/shared/stores/chatStore', () => ({
  useChatStore: () => ({
    clearAdultInteractionForContextChange: mocks.clearAdult,
    sendAdultAction: mocks.sendAdult,
  }),
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

vi.mock('@oclive/shared/stores/uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))

function identityState(currentIdentityId: string) {
  return {
    role_id: 'role',
    identities: [
      { id: 'self', display_name: '本人' },
      { id: 'partner', display_name: '伴侣' },
    ],
    default_identity_id: 'self',
    current_identity_id: currentIdentityId,
    use_manifest_default: currentIdentityId === 'self',
    effective_relation_key: currentIdentityId,
  }
}

const Harness = defineComponent({
  setup() {
    const identities = useUserIdentityState()
    return {
      switchIdentity: () => identities.setIdentity('partner'),
    }
  },
  template: '<button type="button" @click="switchIdentity">switch</button>',
})

describe('user identity catalog adult lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.roleStore.currentRoleId = 'role'
    mocks.roleStore.roleInfo.identityBinding = 'per_scene'
    mocks.uiStore.sceneId = 'home'
    mocks.getIdentity.mockResolvedValue(identityState('self'))
    mocks.setSceneIdentity.mockResolvedValue(identityState('partner'))
    mocks.clearAdult.mockResolvedValue(true)
    mocks.sendAdult.mockResolvedValue(undefined)
    mocks.roleStore.refreshRoleInfo.mockResolvedValue(undefined)
  })

  it('clears the old interaction before switching and replies in the new identity', async () => {
    const wrapper = mount(Harness)
    await vi.waitFor(() => expect(mocks.getIdentity).toHaveBeenCalled())

    await wrapper.get('button').trigger('click')
    await vi.waitFor(() => expect(mocks.sendAdult).toHaveBeenCalled())

    expect(mocks.clearAdult).toHaveBeenCalledWith('role', 'home')
    expect(mocks.setSceneIdentity).toHaveBeenCalledWith(
      'role',
      'home',
      'partner',
    )
    expect(mocks.sendAdult).toHaveBeenCalledWith(
      'exit',
      'home',
      expect.stringContaining('伴侣'),
    )
    expect(mocks.clearAdult.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.setSceneIdentity.mock.invocationCallOrder[0]!,
    )
    expect(mocks.setSceneIdentity.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.sendAdult.mock.invocationCallOrder[0]!,
    )
  })
})
