// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
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
  })
})
