// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
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

function mountInput(loading: boolean, attachTo?: HTMLElement) {
  return mount(ChatInput, {
    attachTo,
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

  it.each([
    { isComposing: true, keyCode: 13 },
    { isComposing: false, keyCode: 229 },
  ])('keeps IME confirmation in the editor ($isComposing, $keyCode), then allows a normal Enter to send', async (keys) => {
    const wrapper = mountInput(false)
    const textarea = wrapper.get<HTMLTextAreaElement>('textarea')
    await textarea.setValue('正在选择中文')
    const confirmation = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true, ...keys })
    textarea.element.dispatchEvent(confirmation)
    expect(confirmation.defaultPrevented).toBe(false)
    expect(wrapper.emitted('send')).toBeUndefined()
    expect(textarea.element.value).toBe('正在选择中文')
    await textarea.trigger('keydown', { key: 'Enter' })
    expect(wrapper.emitted('send')).toEqual([[{ content: '正在选择中文' }]])
    wrapper.unmount()
  })

  it('does not move the keyboard out of a dialog when an async reply requests input focus', async () => {
    const container = document.createElement('div')
    const dialog = document.createElement('div')
    dialog.setAttribute('role', 'dialog')
    dialog.setAttribute('aria-modal', 'true')
    const control = document.createElement('button')
    dialog.append(control)
    document.body.append(container, dialog)
    const wrapper = mountInput(false, container)
    try {
      control.focus()
      wrapper.vm.focusInput()
      await nextTick()
      expect(document.activeElement).toBe(control)
      dialog.append(container)
      wrapper.vm.focusInput()
      await nextTick()
      expect(document.activeElement).toBe(wrapper.get('textarea').element)
    }
    finally {
      wrapper.unmount()
      container.remove()
      dialog.remove()
    }
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
