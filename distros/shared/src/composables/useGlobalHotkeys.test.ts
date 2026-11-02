// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { useGlobalHotkeys } from './useGlobalHotkeys'

const mocks = vi.hoisted(() => ({ shell: 'fluent', emit: vi.fn() }))
vi.mock('./useOcliveShell', () => ({ resolveOcliveShell: () => mocks.shell }))
vi.mock('@oclive/shared/composables/useUnifiedKeybindings', () => ({ useUnifiedKeybindings: vi.fn() }))
vi.mock('@oclive/shared/lib/hostEventBus', () => ({ hostEventBus: { emit: mocks.emit } }))
vi.mock('@oclive/shared/lib/keybindings', () => ({ getPrimaryEffectiveAcceleratorForAction: vi.fn(), loadUnifiedBindingsFile: vi.fn() }))

const wrappers: Array<ReturnType<typeof mount>> = []
function setup() {
  const modelManagerOpen = ref(false)
  const marketPanelVisible = ref(false)
  const debugVisible = ref(false)
  const options = {
    simplePluginManagerOpen: ref(false),
    settingsViewOpen: ref(false),
    topMoreOpen: ref(false),
    modelManagerOpen,
    marketPanelVisible,
    debugVisible,
    pluginUiEnabled: ref(false),
    openPluginManagerPanel: vi.fn(),
    openModelManager: vi.fn(),
    closeModelManager: vi.fn(() => {
      modelManagerOpen.value = false
    }),
    closeMarketPanel: vi.fn(() => {
      marketPanelVisible.value = false
    }),
    toggleDebug: vi.fn(() => {
      debugVisible.value = !debugVisible.value
    }),
  }
  wrappers.push(mount(defineComponent({
    setup() {
      useGlobalHotkeys(options)
      return () => h('div')
    },
  })))
  return options
}

function escape(): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
  window.dispatchEvent(event)
  return event
}

describe('global Escape routing during dialog loading', () => {
  beforeEach(() => {
    mocks.shell = 'fluent'
    vi.clearAllMocks()
  })
  afterEach(() => wrappers.splice(0).forEach(wrapper => wrapper.unmount()))

  it.each(['model', 'settings'])('closes a requested %s window before its lazy view has mounted', (target) => {
    const options = setup()
    const visible = target === 'model' ? options.modelManagerOpen : options.settingsViewOpen
    visible.value = true
    expect(escape().defaultPrevented).toBe(true)
    expect(visible.value).toBe(false)
    expect(options.closeMarketPanel).not.toHaveBeenCalled()
  })

  it('closes the visible market first without closing a lower model window', () => {
    const options = setup()
    options.marketPanelVisible.value = true
    options.modelManagerOpen.value = true
    escape()
    expect(options.marketPanelVisible.value).toBe(false)
    expect(options.modelManagerOpen.value).toBe(true)
  })

  it('does nothing for an inactive debug panel and closes a visible debug panel', () => {
    const options = setup()
    expect(escape().defaultPrevented).toBe(false)
    expect(options.toggleDebug).not.toHaveBeenCalled()
    options.debugVisible.value = true
    expect(escape().defaultPrevented).toBe(true)
    expect(options.debugVisible.value).toBe(false)
  })

  it('keeps the Theatre Escape delegation unchanged', () => {
    mocks.shell = 'theater'
    const options = setup()
    escape()
    expect(mocks.emit).toHaveBeenCalledWith('theater:settings', { action: 'escape' })
    expect(options.closeMarketPanel).not.toHaveBeenCalled()
  })
})
