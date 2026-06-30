import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useChatStore } from './chatStore'
import { useRoleStore } from './roleStore'
import { useUiStore } from './uiStore'

describe('applySceneChange bucket sync', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('loads the resolved scene bucket even when the scene id is unchanged but unloaded', () => {
    const chat = useChatStore()
    const ui = useUiStore()
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    // uiStore still points at the persisted immersive scene, but hydrate only loaded `home`,
    // so the `company` bucket was never loaded — regression guard against history vanishing.
    ui.setScene('company')
    const spy = vi
      .spyOn(chat, 'loadMessagesForRoleScene')
      .mockResolvedValue([])

    chat.applySceneChange('company')

    expect(spy).toHaveBeenCalledWith('mumu', 'company')
  })

  it('does not reload when the scene is unchanged and its bucket is already loaded', () => {
    const chat = useChatStore()
    const ui = useUiStore()
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    ui.setScene('company')
    chat.messageMap = { mumu: { company: [] } }
    const spy = vi
      .spyOn(chat, 'loadMessagesForRoleScene')
      .mockResolvedValue([])

    chat.applySceneChange('company')

    expect(spy).not.toHaveBeenCalled()
  })
})
