// @vitest-environment jsdom

import { OCLIVE_DEFAULT_IDENTITY_SENTINEL } from '@oclive/shared/api'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, reactive } from 'vue'
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

const DefaultHarness = defineComponent({
  setup() {
    const state = useUserIdentityState()
    return { ...state, restoreDefault: () => state.setIdentity(OCLIVE_DEFAULT_IDENTITY_SENTINEL) }
  },
  template: '<span>{{ currentIdentityLabel }}</span><button type="button" @click="restoreDefault">restore</button>',
})

function effectiveState(id: string, followingDefault: boolean) {
  return {
    role_id: 'role',
    identities: [
      { id: 'pack', display_name: 'Pack identity' },
      { id: 'host', display_name: 'Distro identity' },
      { id: 'choice', display_name: 'Explicit identity' },
    ],
    default_identity_id: 'host',
    current_identity_id: id,
    use_manifest_default: followingDefault,
    effective_relation_key: `relation-${id}`,
  }
}

describe('user identity catalog adult lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.roleStore = reactive(mocks.roleStore)
    mocks.uiStore = reactive(mocks.uiStore)
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
    wrapper.unmount()
  })

  it('labels the effective distro default instead of the first pack catalog entry', async () => {
    mocks.getIdentity.mockResolvedValue(effectiveState('host', true))
    const wrapper = mount(DefaultHarness)
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('Distro identity'))
    expect(wrapper.vm.identitySelectValue).toBe(OCLIVE_DEFAULT_IDENTITY_SENTINEL)
    expect(mocks.getIdentity).toHaveBeenCalledWith('role', 'home')
    expect(mocks.setSceneIdentity).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('keeps the explicit label and restores the effective default through the existing sentinel', async () => {
    mocks.roleStore.roleInfo.identityBinding = 'global'
    mocks.getIdentity.mockResolvedValue(effectiveState('choice', false))
    mocks.setIdentity.mockResolvedValue(effectiveState('host', true))
    const wrapper = mount(DefaultHarness)
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('Explicit identity'))
    expect(wrapper.vm.identitySelectValue).toBe('choice')
    await wrapper.get('button').trigger('click')
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('Distro identity'))
    expect(mocks.getIdentity).toHaveBeenCalledWith('role', null)
    expect(mocks.setIdentity).toHaveBeenCalledTimes(1)
    expect(mocks.setIdentity).toHaveBeenCalledWith('role', OCLIVE_DEFAULT_IDENTITY_SENTINEL)
    expect(mocks.setSceneIdentity).not.toHaveBeenCalled()
    expect(wrapper.vm.identitySelectValue).toBe(OCLIVE_DEFAULT_IDENTITY_SENTINEL)
    wrapper.unmount()
  })

  it('continues following roles after the first identity consumer was unmounted', async () => {
    const first = mount(DefaultHarness)
    await vi.waitFor(() => expect(first.get('span').text()).toBe('本人'))
    first.unmount()
    const current = mount(DefaultHarness)
    await vi.waitFor(() => expect(current.get('span').text()).toBe('本人'))
    mocks.getIdentity.mockResolvedValue({ ...identityState('partner'), role_id: 'other-role' })
    mocks.roleStore.currentRoleId = 'other-role'
    await vi.waitFor(() => expect(current.get('span').text()).toBe('伴侣'))
    expect(mocks.getIdentity).toHaveBeenLastCalledWith('other-role', 'home')
    mocks.getIdentity.mockResolvedValue(identityState('self'))
    mocks.roleStore.currentRoleId = 'role'
    await vi.waitFor(() => expect(current.get('span').text()).toBe('本人'))
    current.unmount()
  })

  it('discards an earlier role response after a rapid A-B-A switch', async () => {
    const wrapper = mount(DefaultHarness)
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('本人'))
    let finishOther!: (value: ReturnType<typeof identityState>) => void
    mocks.getIdentity.mockImplementationOnce(() => new Promise((resolve) => {
      finishOther = resolve
    }))
    mocks.roleStore.currentRoleId = 'other-role'
    expect(wrapper.vm.identityState).toBeNull()
    mocks.getIdentity.mockResolvedValue(identityState('self'))
    mocks.roleStore.currentRoleId = 'role'
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('本人'))
    finishOther({ ...identityState('partner'), role_id: 'other-role' })
    await flushPromises()
    expect(wrapper.get('span').text()).toBe('本人')
    wrapper.unmount()
  })

  it('does not apply a late explicit identity change to a newly selected role', async () => {
    mocks.roleStore.roleInfo.identityBinding = 'global'
    mocks.getIdentity.mockResolvedValue(effectiveState('choice', false))
    const wrapper = mount(DefaultHarness)
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('Explicit identity'))
    let finishChange!: (value: ReturnType<typeof effectiveState>) => void
    mocks.setIdentity.mockImplementationOnce(() => new Promise((resolve) => {
      finishChange = resolve
    }))
    await wrapper.get('button').trigger('click')
    await vi.waitFor(() => expect(mocks.setIdentity).toHaveBeenCalled())
    mocks.getIdentity.mockResolvedValue({ ...identityState('partner'), role_id: 'other-role' })
    mocks.roleStore.currentRoleId = 'other-role'
    await vi.waitFor(() => expect(wrapper.get('span').text()).toBe('伴侣'))
    finishChange(effectiveState('host', true))
    await flushPromises()
    expect(wrapper.get('span').text()).toBe('伴侣')
    expect(mocks.setIdentity).toHaveBeenCalledWith('role', OCLIVE_DEFAULT_IDENTITY_SENTINEL)
    expect(mocks.roleStore.refreshRoleInfo).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
