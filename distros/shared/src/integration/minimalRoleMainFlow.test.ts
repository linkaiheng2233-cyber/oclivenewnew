// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMainShellChat } from '../../../chat-pro/src/composables/useMainShellChat'
import ChatInput from '../components/chat/ChatInput.vue'
import ChatMessageList from '../components/chat/ChatMessageList.vue'
import MinimalRoleSourceControls from '../components/role/MinimalRoleSourceControls.vue'
import { useMinimalRoleSelection } from '../composables/useMinimalRoleSelection'
import { i18n } from '../i18n'
import { hostEventBus } from '../lib/hostEventBus'
import { useChatStore } from '../stores/chatStore'
import { useMinimalRoleChatStore } from '../stores/minimalRoleChatStore'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'
import { effectiveChatSceneId } from '../utils/pureChatScene'

const ipc = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke: ipc }))

const wrappers: ReturnType<typeof mount>[] = []
function renderMainFlow() {
  const showToast = vi.fn()
  const contextChanging = ref(false)
  const Harness = defineComponent({
    setup() {
      const role = useRoleStore()
      const minimal = useMinimalRoleChatStore()
      const selection = useMinimalRoleSelection()
      const { t } = useI18n()
      const flow = useMainShellChat({
        roleStore: role,
        uiStore: useUiStore(),
        showToast,
        t,
        clearSceneBarsBeforeSend: vi.fn(),
        offerSceneBarsAfterReply: vi.fn(),
        onTurnRecorded: vi.fn(),
        isContextChanging: () => contextChanging.value,
      })
      return () => h('main', [
        h(MinimalRoleSourceControls, {
          source: minimal.source,
          loading: minimal.isLoading,
          onBind: selection.bindSource,
          onReturn: selection.returnToRichRole,
          onCancel: minimal.cancelPendingSend,
        }),
        h(ChatMessageList, {
          key: flow.activeChatKey.value,
          messages: flow.messages.value,
          loading: flow.chatListLoading.value,
          historySplitIndex: flow.sceneHistorySplitIndex.value,
          roleSwitching: false,
        }),
        h(ChatInput, {
          loading: flow.chatListLoading.value,
          textOnly: role.minimalRoleActive,
          roleName: role.minimalRoleActive ? 'Minimal' : undefined,
          onSend: flow.onSend,
        }),
      ])
    },
  })
  const wrapper = mount(Harness, { global: { plugins: [i18n] } })
  wrappers.push(wrapper)
  return { wrapper, showToast, contextChanging }
}

async function settle() {
  for (let i = 0; i < 8; i++)
    await new Promise<void>(resolve => setTimeout(resolve, 0))
}

async function selectSource(wrapper: ReturnType<typeof mount>) {
  await wrapper.get('input[name="asset-root"]').setValue('E:\\synthetic-minimal')
  await wrapper.get('input[name="definition-reference"]').setValue('converted/persona.json')
  await wrapper.get('form').trigger('submit')
  await settle()
}

describe('minimal role in the main chat flow', () => {
  beforeEach(() => {
    vi.stubGlobal('ResizeObserver', class {
      observe() {}
      unobserve() {}
      disconnect() {}
    })
    setActivePinia(createPinia())
    ipc.mockReset()
    useRoleStore().currentRoleId = 'retained-rich-role'
  })
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    hostEventBus.all.clear()
    vi.unstubAllGlobals()
  })

  it('binds an explicit source and sends from the existing composer to the actual main list', async () => {
    const sent = vi.fn()
    hostEventBus.on('message:sent', sent)
    const { wrapper } = renderMainFlow()
    await selectSource(wrapper)
    const source = useMinimalRoleChatStore().source!
    expect(source.role_id).toMatch(/^minimal-[\da-f-]{36}$/)
    expect(useRoleStore().currentRoleId).toBe('retained-rich-role')
    expect(useRoleStore().interactionImmersive).toBe(false)
    ipc.mockResolvedValue({ role_id: source.role_id, reply: '  authoritative\ntext  ', product_extensions: 'unavailable' })
    await wrapper.get('textarea').setValue('hello')
    await wrapper.get('button.send').trigger('click')
    await settle()
    expect(ipc.mock.calls).toEqual([['send_minimal_message', {
      req: { source, message: { user_message: 'hello', requirements: '' } },
    }]])
    expect(wrapper.findComponent(ChatMessageList).props('messages').map((message: { content: string }) => message.content))
      .toEqual(['hello', '  authoritative\ntext  '])
    expect(wrapper.text()).toContain('authoritative')
    expect(wrapper.text()).toContain('hello')
    expect(sent).toHaveBeenCalledOnce()
    expect(sent.mock.calls[0][0]).toMatchObject({ reply: '  authoritative\ntext  ', skip_auto_tts: true })
    expect(useChatStore().messageMap).toEqual({})
    useChatStore().messageMap['retained-rich-role'] = {
      [effectiveChatSceneId(useRoleStore().roleInfo.interactionMode, useUiStore().sceneId)]: [{ id: 'existing-row-id', role: 'assistant', content: 'retained rich reply', timestamp: 1 }],
    }
    await wrapper.get('button[data-action="return-rich"]').trigger('click')
    expect(useRoleStore().minimalRoleActive).toBe(false)
    expect(useRoleStore().currentRoleId).toBe('retained-rich-role')
    expect(useMinimalRoleChatStore().messages).toEqual([])
    expect(wrapper.text()).toContain('retained rich reply')
  })

  it('keeps completed bubbles visible and supplies only this temporary conversation to the Host', async () => {
    const { wrapper } = renderMainFlow()
    await selectSource(wrapper)
    const source = useMinimalRoleChatStore().source!
    ipc.mockResolvedValue({ role_id: source.role_id, reply: 'first reply', product_extensions: 'unavailable' })
    await wrapper.get('textarea').setValue('first user')
    await wrapper.get('button.send').trigger('click')
    await settle()
    ipc.mockResolvedValue({ role_id: source.role_id, reply: 'second reply', product_extensions: 'unavailable' })
    await wrapper.get('textarea').setValue('second user')
    await wrapper.get('button.send').trigger('click')
    await settle()
    for (const text of ['first user', 'first reply', 'second user', 'second reply'])
      expect(wrapper.text()).toContain(text)
    expect(ipc.mock.calls[1][1]).toEqual({ req: {
      source,
      message: { user_message: 'second user', requirements: '' },
      conversation: [{ user_message: 'first user', reply: 'first reply' }],
    } })
  })

  it('stops waiting without accepting a late reply or sending a recovery request', async () => {
    let resolve!: (value: unknown) => void
    ipc.mockImplementation(() => new Promise(r => resolve = r))
    const sent = vi.fn()
    hostEventBus.on('message:sent', sent)
    const { wrapper } = renderMainFlow()
    await selectSource(wrapper)
    const source = useMinimalRoleChatStore().source!
    await wrapper.get('textarea').setValue('wait')
    await wrapper.get('button.send').trigger('click')
    await settle()
    expect(useMinimalRoleChatStore().isLoading).toBe(true)
    await wrapper.get('button[data-action="cancel-minimal"]').trigger('click')
    resolve({ role_id: source.role_id, reply: 'late', product_extensions: 'unavailable' })
    await settle()
    expect(useMinimalRoleChatStore().messages).toEqual([])
    expect(sent).not.toHaveBeenCalled()
    expect(ipc.mock.calls.map(call => call[0])).toEqual(['send_minimal_message'])
  })

  it('surfaces an original transport error and removes the pending bubble', async () => {
    ipc.mockRejectedValue(new Error('synthetic model unavailable'))
    const { wrapper, showToast } = renderMainFlow()
    await selectSource(wrapper)
    await wrapper.get('textarea').setValue('hello')
    await wrapper.get('button.send').trigger('click')
    await settle()
    expect(showToast).toHaveBeenCalledWith('error', 'synthetic model unavailable')
    expect(useMinimalRoleChatStore().messages).toEqual([])
    expect(ipc).toHaveBeenCalledOnce()
  })

  it('validates before changing the rich context and refuses binding if queue cancellation fails', async () => {
    const selection = useMinimalRoleSelection()
    const chat = useChatStore()
    const cancel = vi.spyOn(chat, 'clearAdultInteractionForContextChange')
    await expect(selection.bindSource({ role_id: 'id', asset_root: '', definition_reference: 'x' })).rejects.toThrow()
    expect(cancel).not.toHaveBeenCalled()
    cancel.mockRejectedValueOnce(new Error('queue cancellation failed'))
    await expect(selection.bindSource({ role_id: 'id', asset_root: 'E:\\fixture', definition_reference: 'x' }))
      .rejects
      .toThrow('queue cancellation failed')
    expect(useMinimalRoleChatStore().source).toBeNull()
    expect(useRoleStore().currentRoleId).toBe('retained-rich-role')
  })

  it('does not start a rich or minimal send during a context transition', async () => {
    const { wrapper, contextChanging } = renderMainFlow()
    contextChanging.value = true
    await wrapper.get('textarea').setValue('during rich transition')
    await wrapper.get('button.send').trigger('click')
    await selectSource(wrapper)
    await wrapper.get('textarea').setValue('during minimal transition')
    await wrapper.get('button.send').trigger('click')
    await settle()
    expect(ipc).not.toHaveBeenCalled()
    expect(useMinimalRoleChatStore().messages).toEqual([])
  })

  it('does not rebind after returning while adult queue cancellation is pending', async () => {
    let release!: () => void
    vi.spyOn(useChatStore(), 'clearAdultInteractionForContextChange').mockImplementationOnce(() => new Promise(resolve => release = () => resolve(false)))
    const selection = useMinimalRoleSelection()
    const pending = selection.bindSource({ role_id: 'id', asset_root: 'E:\\fixture', definition_reference: 'x' })
    selection.returnToRichRole()
    release()
    await pending
    expect(useMinimalRoleChatStore().source).toBeNull()
    expect(ipc).not.toHaveBeenCalled()
  })
})
