import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useRoleStore } from './roleStore'

const mocks = vi.hoisted(() => ({
  cancelQueue: vi.fn(),
  clearSession: vi.fn(),
  setSceneRelation: vi.fn(),
  clearSceneRelation: vi.fn(),
  setUserRelation: vi.fn(),
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    clearSceneUserRelation: mocks.clearSceneRelation,
    setSceneUserRelation: mocks.setSceneRelation,
    setUserRelation: mocks.setUserRelation,
  }
})

vi.mock('@oclive/shared/lib/adultBeatQueue', () => ({
  cancelAdultBeatQueue: mocks.cancelQueue,
}))

vi.mock('./adultInteractionStore', () => ({
  useAdultInteractionStore: () => ({
    clearSession: mocks.clearSession,
  }),
}))

function roleInfo() {
  return {
    role_id: 'role',
    role_name: 'Role',
    current_favorability: 0,
    current_emotion: 'neutral',
    relation_state: 'Friend',
    event_impact_factor: 1,
    personality_source: 'profile',
    identity_binding: 'global',
  }
}

describe('role store relation lifecycle ownership', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mocks.cancelQueue.mockResolvedValue(undefined)
    mocks.setSceneRelation.mockResolvedValue(roleInfo())
    mocks.clearSceneRelation.mockResolvedValue(roleInfo())
    mocks.setUserRelation.mockResolvedValue(roleInfo())
  })

  it('cancels and clears before a global relation mutation', async () => {
    const store = useRoleStore()
    store.currentRoleId = 'role'

    await store.setGlobalUserRelation('partner', 'home')

    expect(mocks.cancelQueue).toHaveBeenCalledWith('role', 'home')
    expect(mocks.clearSession).toHaveBeenCalledWith('role', 'home')
    expect(mocks.setUserRelation).toHaveBeenCalledWith('role', 'partner')
    expect(mocks.cancelQueue.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.clearSession.mock.invocationCallOrder[0]!,
    )
    expect(mocks.clearSession.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.setUserRelation.mock.invocationCallOrder[0]!,
    )
  })

  it('cancels and clears before a per-scene relation mutation', async () => {
    const store = useRoleStore()
    store.currentRoleId = 'role'

    await store.setSceneUserRelation('garden', 'partner')

    expect(mocks.cancelQueue).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.clearSession).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.setSceneRelation).toHaveBeenCalledWith(
      'role',
      'garden',
      'partner',
    )
    expect(mocks.clearSession.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.setSceneRelation.mock.invocationCallOrder[0]!,
    )
  })

  it('cancels and clears before restoring the manifest default', async () => {
    const store = useRoleStore()
    store.currentRoleId = 'role'

    await store.setManifestDefaultRelation('garden')

    expect(mocks.cancelQueue).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.clearSession).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.clearSceneRelation).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.setUserRelation).toHaveBeenCalledWith(
      'role',
      '__oclive_default__',
    )
    expect(mocks.clearSession.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.clearSceneRelation.mock.invocationCallOrder[0]!,
    )
    expect(mocks.clearSceneRelation.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.setUserRelation.mock.invocationCallOrder[0]!,
    )
  })
})
