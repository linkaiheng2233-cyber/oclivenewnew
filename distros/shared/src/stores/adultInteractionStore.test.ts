import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useAdultInteractionStore } from './adultInteractionStore'

describe('adult interaction gates and session state', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('requires confirmation plus global and per-role gates', () => {
    const store = useAdultInteractionStore()
    expect(store.requestFor('mumu', 'home')).toBeUndefined()
    store.confirmAndEnableGlobal()
    expect(store.requestFor('mumu', 'home')).toBeUndefined()
    store.setRoleEnabled('mumu', true)
    expect(store.requestFor('mumu', 'home', 'continue')).toMatchObject({
      confirmed_adult: true,
      global_enabled: true,
      role_enabled: true,
      interaction_active: false,
      action: 'continue',
    })
  })

  it('global off clears active sessions but preserves role choices', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.updateSession('mumu', 'home', 'active')
    expect(store.sessionFor('mumu', 'home').active).toBe(true)

    store.setGlobalEnabled(false)

    expect(store.roleIsEnabled('mumu')).toBe(true)
    expect(store.sessionFor('mumu', 'home').active).toBe(false)
  })

  it('keeps voice text-only degradation scoped to one active interaction', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.updateSession('mumu', 'home', 'active')
    store.markVoiceTextOnly('mumu', 'home')
    expect(store.sessionFor('mumu', 'home').voiceTextOnly).toBe(true)

    store.updateSession('mumu', 'home', 'ended')
    store.updateSession('mumu', 'home', 'active')
    expect(store.sessionFor('mumu', 'home').voiceTextOnly).toBe(false)
  })

  it('keeps a staged generation id only while the interaction is active', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.updateSession('mumu', 'home', 'active')
    store.setSessionGeneration('mumu', 'home', 'generation-1')
    expect(store.sessionFor('mumu', 'home').generationId).toBe('generation-1')

    store.setSessionGeneration('mumu', 'home')
    expect(store.sessionFor('mumu', 'home').generationId).toBeUndefined()
    store.updateSession('mumu', 'home', 'ended')
    expect(store.sessionFor('mumu', 'home').active).toBe(false)
  })

  it('keeps an inactive cancellation tombstone until the generation is cleared', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.updateSession('mumu', 'home', 'active')
    store.setSessionGeneration('mumu', 'home', 'generation-1')

    store.setGlobalEnabled(false)

    expect(store.sessionFor('mumu', 'home')).toMatchObject({
      active: false,
      generationId: 'generation-1',
      roleId: 'mumu',
      sceneId: 'home',
    })

    store.setSessionGeneration('mumu', 'home')
    expect(store.sessions['mumu:home']).toBeUndefined()
  })

  it('normalizes the global staged queue cap to a positive integer', () => {
    const store = useAdultInteractionStore()
    store.setBackgroundQueue(true, 4, true)
    expect(store.backgroundQueueEnabled).toBe(true)
    expect(store.backgroundQueueCap).toBe(4)
    expect(store.backgroundQueueWarningAccepted).toBe(true)

    store.setBackgroundQueue(true, 0, true)
    expect(store.backgroundQueueCap).toBe(4)
  })
})
