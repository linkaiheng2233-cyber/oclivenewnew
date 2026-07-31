// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import RoleRuntimePanel from './RoleRuntimePanel.vue'

const mocks = vi.hoisted(() => ({
  sendAdult: vi.fn(),
  showToast: vi.fn(),
  adultStore: {
    sessionFor: vi.fn(() => ({
      active: true,
      voiceTextOnly: false,
      updatedAt: 1,
    })),
  },
  uiStore: { sceneId: 'home' },
  roleStore: {
    currentRoleId: 'role',
    relationSelectValue: 'friend',
    roleInfo: {
      description: '',
      version: '1',
      author: 'test',
      personalitySource: 'profile',
      eventImpactFactor: 1,
      identityBinding: 'global',
      defaultRelation: 'friend',
      userRelations: [
        {
          id: 'friend',
          name: 'Friend',
          prompt_hint: '',
          favor_multiplier: 1,
          initial_favorability: 0,
        },
        {
          id: 'partner',
          name: 'Partner',
          prompt_hint: '',
          favor_multiplier: 1,
          initial_favorability: 0,
        },
      ],
    },
    refreshRoleInfo: vi.fn(),
    setGlobalUserRelation: vi.fn(),
    setManifestDefaultRelation: vi.fn(),
    setSceneUserRelation: vi.fn(),
  },
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    setEvolutionFactor: vi.fn(),
  }
})

vi.mock('@oclive/shared/composables/useAppToast', () => ({
  useAppToast: () => ({ showToast: mocks.showToast }),
}))

vi.mock('@oclive/shared/stores/adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))

vi.mock('@oclive/shared/stores/chatStore', () => ({
  useChatStore: () => ({
    sendAdultAction: mocks.sendAdult,
  }),
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

vi.mock('@oclive/shared/stores/uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))

function mountPanel() {
  return mount(RoleRuntimePanel, {
    global: {
      plugins: [
        createI18n({
          legacy: false,
          locale: 'en',
          missingWarn: false,
          fallbackWarn: false,
          messages: { en: {} },
        }),
      ],
      stubs: {
        HelpHint: true,
        RoleIdentityControls: true,
      },
    },
  })
}

describe('role runtime adult lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.roleStore.currentRoleId = 'role'
    mocks.roleStore.relationSelectValue = 'friend'
    mocks.roleStore.roleInfo.identityBinding = 'global'
    mocks.uiStore.sceneId = 'home'
    mocks.sendAdult.mockResolvedValue(undefined)
    mocks.roleStore.setGlobalUserRelation.mockResolvedValue(undefined)
  })

  it('lets the store switch identity, then responds in the new identity', async () => {
    const wrapper = mountPanel()

    await wrapper.get('#rel-select').setValue('partner')

    expect(mocks.roleStore.setGlobalUserRelation)
      .toHaveBeenCalledWith('partner', 'home')
    expect(mocks.sendAdult)
      .toHaveBeenCalledWith(
        'exit',
        'home',
        expect.stringContaining('Partner'),
      )
    expect(mocks.roleStore.setGlobalUserRelation.mock.invocationCallOrder[0])
      .toBeLessThan(mocks.sendAdult.mock.invocationCallOrder[0]!)
  })
})
