// @vitest-environment jsdom
import { usePluginManagerWindow } from '@oclive/shared/composables/usePluginManagerWindow'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, ref } from 'vue'
import TopBarMorePanel from './TopBarMorePanel.vue'

const mocks = vi.hoisted(() => ({
  clockMounted: vi.fn(),
  openMarket: vi.fn(),
  closeMarket: vi.fn(),
  roleStore: { interactionImmersive: true, currentRoleId: 'fixture', relationSelectValue: 'default', roles: [], roleInfo: {} },
}))
vi.mock('@oclive/shared/stores/roleStore', () => ({ useRoleStore: () => mocks.roleStore }))
vi.mock('@oclive/shared/stores/chatStore', () => ({ useChatStore: () => ({ isLoading: false }) }))
vi.mock('@oclive/shared/stores/debugStore', () => ({ useDebugStore: () => ({ toggle: vi.fn() }) }))
vi.mock('@oclive/shared/stores/pluginMarketStore', () => ({ usePluginMarketStore: () => ({ openMarketPanel: mocks.openMarket, closeMarketPanel: mocks.closeMarket }) }))
vi.mock('@oclive/shared/composables/useSceneDestination', () => ({ useSceneDestination: () => ({ characterSceneLabel: () => 'fixture' }) }))
vi.mock('@oclive/shared/composables/useAppToast', () => ({ useAppToast: () => ({ showToast: vi.fn() }) }))
vi.mock('@oclive/shared/composables/useOcliveShell', () => ({ resolveOcliveShell: () => 'fluent' }))
vi.mock('@oclive/shared/i18n/loadPluginWorkbench', () => ({ ensurePluginWorkbenchI18n: vi.fn() }))
vi.mock('vue-i18n', async importOriginal => ({
  ...await importOriginal<typeof import('vue-i18n')>(),
  useI18n: () => ({ t: (key: string) => key }),
}))

const wrappers: Array<ReturnType<typeof mount>> = []
describe('simple chat controls', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })
  afterEach(() => wrappers.splice(0).forEach(wrapper => wrapper.unmount()))

  it('keeps core settings and identity but mounts the virtual clock only in an open advanced menu', async () => {
    const ui = useUiStore()
    const wrapper = mount(TopBarMorePanel, {
      props: { modelValue: true, allSceneOptions: [], relationOptions: [] },
      global: { stubs: {
        VirtualTimeBar: defineComponent({ setup: mocks.clockMounted, template: '<div data-clock />' }),
        RoleSelector: true,
        HelpHint: true,
      } },
    })
    wrappers.push(wrapper)
    expect(wrapper.text()).toContain('app.more.openSettings')
    expect(wrapper.text()).toContain('app.more.modelManager')
    expect(wrapper.text()).toContain('app.more.identity')
    expect(mocks.clockMounted).not.toHaveBeenCalled()
    ui.setAdvancedFeaturesEnabled(true)
    await flushPromises()
    expect(wrapper.find('[data-clock]').exists()).toBe(true)
    await wrapper.setProps({ modelValue: false })
    expect(wrapper.find('[data-clock]').exists()).toBe(false)
    expect(mocks.clockMounted).toHaveBeenCalledTimes(1)
  })

  it('blocks direct plugin manager and market entry functions until advanced is enabled', () => {
    const enabled = ref(false)
    let windows!: ReturnType<typeof usePluginManagerWindow>
    const wrapper = mount(defineComponent({
      setup() {
        windows = usePluginManagerWindow({ closeMoreMenu: vi.fn(), enabled: () => enabled.value })
        return () => null
      },
    }))
    wrappers.push(wrapper)
    windows.openSimplePluginManager(true)
    windows.openPluginMarket()
    expect(windows.simplePluginManagerOpen.value).toBe(false)
    expect(mocks.openMarket).not.toHaveBeenCalled()
    enabled.value = true
    windows.openSimplePluginManager(true)
    expect(windows.simplePluginManagerOpen.value).toBe(true)
    enabled.value = false
    windows.openPluginMarket()
    expect(mocks.openMarket).not.toHaveBeenCalled()
  })
})
