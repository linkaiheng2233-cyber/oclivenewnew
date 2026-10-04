// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import { useAdultInteractionStore } from '../../stores/adultInteractionStore'
import ChatInput from './ChatInput.vue'

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => ({
    currentRoleId: 'mumu',
    roleInfo: {
      name: 'Mumu',
      interactionMode: 'immersive',
    },
  }),
}))

function mountInput(loading: boolean) {
  return mount(ChatInput, {
    props: { loading },
    global: {
      plugins: [
        createI18n({
          legacy: false,
          locale: 'en',
          messages: {
            en: {
              common: {
                chatPlaceholder: 'Message {name}',
                chatInputLabel: 'Message',
                send: 'Send',
              },
              app: { defaultRoleName: 'Role' },
              chat: { adultExit: 'Exit' },
            },
          },
        }),
        createPinia(),
      ],
    },
  })
}

describe('chat input generation overlap', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('keeps text entry and send available while the previous reply is loading', async () => {
    const wrapper = mountInput(true)
    const textarea = wrapper.get<HTMLTextAreaElement>('textarea')

    expect(textarea.attributes('disabled')).toBeUndefined()
    await textarea.setValue('补充一句')
    const send = wrapper.get<HTMLButtonElement>('button.send')
    expect(send.attributes('disabled')).toBeUndefined()
    await send.trigger('click')

    expect(wrapper.emitted('send')).toEqual([[{ content: '补充一句' }]])
    wrapper.unmount()
  })

  it('uses the explicit minimal label and hides a retained active adult session', async () => {
    const wrapper = mountInput(false)
    useAdultInteractionStore().updateSession('mumu', 'default', 'active')
    await wrapper.setProps({ textOnly: true, roleName: 'Minimal' })
    expect(wrapper.get('textarea').attributes('placeholder')).toBe('Message Minimal')
    expect(wrapper.find('button.adult-exit').exists()).toBe(false)
    await wrapper.get('textarea').setValue('ordinary text')
    await wrapper.get('button.send').trigger('click')
    expect(wrapper.emitted('adultAction')).toBeUndefined()
    expect(wrapper.emitted('send')).toEqual([[{ content: 'ordinary text' }]])
    wrapper.unmount()
  })
})
