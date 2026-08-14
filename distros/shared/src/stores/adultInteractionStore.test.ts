// @vitest-environment jsdom

import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAdultInteractionStore } from './adultInteractionStore'

describe('adult interaction gates and session state', () => {
  beforeEach(() => {
    localStorage.clear()
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

  it('rejects new pacing and queue values outside the hard bounds', () => {
    const store = useAdultInteractionStore()
    expect(store.setPacingOverride(true, 499)).toBe(false)
    expect(store.pacingOverrideEnabled).toBe(false)
    expect(store.pacingIntervalMs).toBe(4_000)

    expect(store.setPacingOverride(true, 500)).toBe(true)
    expect(store.setPacingOverride(true, 60_000)).toBe(true)
    expect(store.setPacingOverride(true, 60_001)).toBe(false)
    expect(store.pacingIntervalMs).toBe(60_000)

    expect(store.setBackgroundQueue(true, 8, true)).toBe(true)
    expect(store.backgroundQueueEnabled).toBe(true)
    expect(store.backgroundQueueCap).toBe(8)
    expect(store.backgroundQueueWarningAccepted).toBe(true)

    expect(store.setBackgroundQueue(true, 9, true)).toBe(false)
    expect(store.backgroundQueueCap).toBe(8)
  })

  it('clamps legacy persisted pacing and queue values to the hard bounds', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    localStorage.setItem('oclive-chat-pro-adult-settings-v1', JSON.stringify({
      pacingIntervalMs: 100,
      backgroundQueueCap: 99,
    }))
    setActivePinia(createPinia())

    const store = useAdultInteractionStore()

    expect(store.pacingIntervalMs).toBe(500)
    expect(store.backgroundQueueCap).toBe(8)
    expect(warn).toHaveBeenCalledTimes(2)
  })

  it('resets the adult confirmation and all local R18 settings', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.setPacingOverride(true, 5_000)
    store.setBackgroundQueue(true, 4, true)
    store.updateSession('mumu', 'home', 'active')

    store.resetAdultSettings()

    expect(store.$state).toMatchObject({
      confirmedAdult: false,
      globalEnabled: false,
      roleEnabled: {},
      pacingOverrideEnabled: false,
      pacingIntervalMs: 4_000,
      backgroundQueueEnabled: false,
      backgroundQueueCap: 2,
      backgroundQueueWarningAccepted: false,
      sessions: {},
    })
    expect(JSON.parse(
      localStorage.getItem('oclive-chat-pro-adult-settings-v1') ?? '{}',
    )).toMatchObject({
      confirmedAdult: false,
      globalEnabled: false,
      roleEnabled: {},
    })
  })

  it('keeps only a failed-cancellation tombstone when resetting local settings', () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('mumu', true)
    store.updateSession('mumu', 'home', 'active')
    store.setSessionGeneration('mumu', 'home', 'generation-1')

    store.resetAdultSettings()

    expect(store.sessionFor('mumu', 'home')).toMatchObject({
      active: false,
      voiceTextOnly: false,
      generationId: 'generation-1',
    })
    expect(store.confirmedAdult).toBe(false)
    expect(store.roleEnabled).toEqual({})
  })
})
