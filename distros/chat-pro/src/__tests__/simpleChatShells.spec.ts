// @vitest-environment jsdom
import { MAIN_SHELL_KEY } from '@oclive/shared/composables/mainShellKey'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { flushPromises, shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, reactive, ref } from 'vue'
import FluentShell from '../shells/fluent/FluentShell.vue'
import ToolShell from '../shells/tool/ToolShell.vue'

vi.mock('../composables/useMainShell', async () => {
  const { defineComponent } = await import('vue')
  return Object.fromEntries([
    'DebugPanel',
    'MarketView',
    'ModelManagerPanel',
    'RoleDetailView',
    'RoleplayAsidePanel',
    'SceneTravelBars',
    'SettingsView',
    'SimplePluginManagerPanel',
    'TopBarMorePanel',
    'AutonomousSceneNotice',
  ].map(name => [name, defineComponent({ name, template: '<div><slot name="leading" /></div>' })]))
})

const wrappers: Array<ReturnType<typeof shallowMount>> = []
function fixture() {
  const uiStore = useUiStore()
  uiStore.setScene('saved-scene')
  const roleStore = reactive({
    minimalRoleActive: false,
    interactionImmersive: true,
    currentRoleId: 'fixture',
    relationSelectValue: 'default',
    roles: [],
    roleInfo: { interactionMode: 'immersive', favorability: 1, currentLife: null },
  })
  return {
    t: (key: string) => key,
    uiStore,
    roleStore,
    storyUiEnabled: computed(() => uiStore.advancedFeaturesEnabled && roleStore.interactionImmersive),
    pluginStore: { bootstrapEpoch: 0 },
    pluginMarketStore: { marketPanelVisible: false },
    minimalRoleChatStore: { source: null, isLoading: false },
    chatStore: { isLoading: false },
    debugStore: { visible: false },
    toast: { show: false, type: 'info', message: '' },
    progressive: { showImmersiveUnlockBanner: true, immersiveIntroVisible: true },
    sceneTransition: { visible: false },
    autonomousSceneNotice: { visible: false },
    messages: [],
    relationOptions: [],
    allSceneOptions: [],
    sceneDestinationOptions: [],
    wideSplitLayout: ref(true),
    chatListLoading: ref(false),
    roleSwitching: ref(false),
    topMoreOpen: ref(false),
    settingsViewOpen: ref(false),
    modelManagerOpen: ref(false),
    simplePluginManagerOpen: ref(false),
    leftPaneRef: ref(null),
    chatListRef: ref(null),
    chatInputRef: ref(null),
    presetPickerOpen: ref(false),
    shortcutHelpOpen: ref(false),
    sidePanelOpen: ref(false),
    sidePanelTab: ref('settings'),
    pluginsPanelSubview: ref('list'),
    statusHeart: '',
    sceneLabelForId: (id: string) => id,
  }
}

const stubs = {
  RoleDetailView: true,
  TopBarMorePanel: { template: '<div><slot name="leading" /></div>' },
}

describe.each([['fluent', FluentShell], ['tool', ToolShell]] as const)('%s simple chat shell', (_name, component) => {
  beforeEach(() => setActivePinia(createPinia()))
  afterEach(() => wrappers.splice(0).forEach(wrapper => wrapper.unmount()))

  it('keeps the composer and character while optional hosts stay unmounted', async () => {
    const shell = fixture()
    const wrapper = shallowMount(component, { global: { provide: { [MAIN_SHELL_KEY as symbol]: shell }, stubs } })
    wrappers.push(wrapper)
    await vi.dynamicImportSettled()
    await flushPromises()
    expect(wrapper.find('chat-input-stub').exists()).toBe(true)
    expect(wrapper.find('role-detail-view-stub').exists(), wrapper.html()).toBe(true)
    for (const tag of ['advanced-feature-host', 'chat-plugin-toolbar-slots', 'plugin-sidebar-slots', 'plugin-chat-header-slots', 'immersive-unlock-banner', 'minimal-role-source-controls'])
      expect(wrapper.find(`${tag}-stub`).exists(), tag).toBe(false)
    shell.uiStore.setAdvancedFeaturesEnabled(true)
    await flushPromises()
    expect(wrapper.find('advanced-feature-host-stub').exists()).toBe(true)
    expect(wrapper.find('chat-plugin-toolbar-slots-stub').exists()).toBe(true)
    shell.uiStore.setAdvancedFeaturesEnabled(false)
    await flushPromises()
    expect(wrapper.find('advanced-feature-host-stub').exists()).toBe(false)
    expect(shell.uiStore.sceneId).toBe('saved-scene')
    expect(shell.roleStore.roleInfo.interactionMode).toBe('immersive')
  })

  it('keeps a selected minimal role return path without exposing new binding', async () => {
    const shell = fixture()
    shell.roleStore.minimalRoleActive = true
    const wrapper = shallowMount(component, { global: { provide: { [MAIN_SHELL_KEY as symbol]: shell }, stubs } })
    wrappers.push(wrapper)
    await vi.dynamicImportSettled()
    await flushPromises()
    const controls = wrapper.findComponent({ name: 'MinimalRoleSourceControls' })
    expect(controls.exists(), wrapper.html()).toBe(true)
    expect(controls.props('allowBinding')).toBe(false)
    expect(wrapper.find('advanced-feature-host-stub').exists()).toBe(false)
  })
})
