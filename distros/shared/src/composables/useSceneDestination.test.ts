// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { createI18n } from 'vue-i18n'
import { useSceneDestination } from './useSceneDestination'

const mocks = vi.hoisted(() => ({
  setPresenceScene: vi.fn(),
  clearAdult: vi.fn(),
  sendAdult: vi.fn(),
  applySceneChange: vi.fn(),
  loadDebug: vi.fn(),
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
    interactionImmersive: true,
    roleInfo: {
      currentScene: 'home',
      userPresenceScene: 'home',
      sceneLabels: [
        { id: 'home', label: 'Home' },
        { id: 'garden', label: 'Garden' },
      ],
    },
    applyRoleInfo: vi.fn(),
  },
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    setUserPresenceScene: mocks.setPresenceScene,
    switchScene: vi.fn(),
  }
})

vi.mock('@oclive/shared/stores/adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))

vi.mock('@oclive/shared/stores/chatStore', () => ({
  useChatStore: () => ({
    applySceneChange: mocks.applySceneChange,
    addSystemMessage: vi.fn(),
    clearAdultInteractionForContextChange: mocks.clearAdult,
    sendAdultAction: mocks.sendAdult,
  }),
}))

vi.mock('@oclive/shared/stores/debugStore', () => ({
  useDebugStore: () => ({ loadDebugData: mocks.loadDebug }),
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

vi.mock('@oclive/shared/stores/uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))

const Harness = defineComponent({
  setup() {
    const destination = useSceneDestination(mocks.showToast)
    return {
      go: () => destination.applySceneDestination('garden', false),
    }
  },
  template: '<button type="button" @click="go">go</button>',
})

describe('scene destination adult lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.uiStore.sceneId = 'home'
    mocks.roleStore.currentRoleId = 'role'
    mocks.roleStore.roleInfo.currentScene = 'home'
    mocks.roleStore.roleInfo.userPresenceScene = 'home'
    mocks.clearAdult.mockResolvedValue(true)
    mocks.applySceneChange.mockResolvedValue(undefined)
    mocks.sendAdult.mockResolvedValue(undefined)
    mocks.loadDebug.mockResolvedValue(undefined)
    mocks.setPresenceScene.mockResolvedValue({
      role_id: 'role',
      current_scene: 'home',
      user_presence_scene: 'garden',
    })
    mocks.roleStore.applyRoleInfo.mockImplementation((info: {
      current_scene?: string
      user_presence_scene?: string
    }) => {
      mocks.roleStore.roleInfo.currentScene
        = info.current_scene ?? mocks.roleStore.roleInfo.currentScene
      mocks.roleStore.roleInfo.userPresenceScene
        = info.user_presence_scene ?? mocks.roleStore.roleInfo.userPresenceScene
    })
  })

  it('clears the old interaction before switching and responds only in the new scene', async () => {
    const wrapper = mount(Harness, {
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
      },
    })

    await wrapper.get('button').trigger('click')

    expect(mocks.clearAdult).toHaveBeenCalledWith('role', 'home')
    expect(mocks.setPresenceScene).toHaveBeenCalledWith('role', 'garden')
    expect(mocks.applySceneChange).toHaveBeenCalledWith('garden')
    expect(mocks.sendAdult).toHaveBeenCalledWith(
      'exit',
      'garden',
      expect.stringContaining('Garden'),
    )
    expect(mocks.clearAdult.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.setPresenceScene.mock.invocationCallOrder[0]!,
    )
    expect(mocks.applySceneChange.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.sendAdult.mock.invocationCallOrder[0]!,
    )
  })
})
