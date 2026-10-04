// @vitest-environment jsdom

import type { RoleSnapshot } from '../api/kernel'
import type { useUnifiedKeybindings } from './useUnifiedKeybindings'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, defineComponent, ref } from 'vue'
import { useMainShellHotkeys } from '../../../chat-pro/src/composables/useMainShellHotkeys'
import { i18n } from '../i18n'
import { hostEventBus } from '../lib/hostEventBus'
import { VOICE_ASR_SUBMIT_EVENT } from '../lib/voiceAsrEvents'
import { useKernelConnectionStore } from '../stores/kernelConnectionStore'
import { useMinimalRoleChatStore } from '../stores/minimalRoleChatStore'
import { useRoleStore } from '../stores/roleStore'
import { useRoleSnapshotPoll } from './useKernelStatus'
import { usePluginEvents } from './usePluginEvents'
import { usePackUiTheme } from './useTheme'

const mocks = vi.hoisted(() => ({ snapshot: vi.fn(), ipc: vi.fn(), bindings: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: mocks.ipc }))
vi.mock('@oclive/shared/api/kernel', async original => ({
  ...await original<typeof import('../api/kernel')>(),
  fetchRoleSnapshot: mocks.snapshot,
}))
// Capture only keyboard registration; the actual MainShell/Global action guards run.
vi.mock('@oclive/shared/composables/useUnifiedKeybindings', () => ({ useUnifiedKeybindings: mocks.bindings }))

const wrappers: ReturnType<typeof mount>[] = []
function mountSetup(setup: () => void) {
  const wrapper = mount(defineComponent({
    setup() {
      setup()
      return () => null
    },
  }), { global: { plugins: [i18n] } })
  wrappers.push(wrapper)
}
function bindMinimal() {
  useMinimalRoleChatStore().bindSource({ role_id: 'minimal-id', asset_root: 'E:\\fixture', definition_reference: 'persona.json' })
}

describe('minimal main-chat capability scope', () => {
  beforeEach(() => {
    vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener() {}, removeEventListener() {} }))
    setActivePinia(createPinia())
    vi.clearAllMocks()
    const role = useRoleStore()
    role.currentRoleId = 'rich'
    role.roleInfo.interactionMode = 'immersive'
    useKernelConnectionStore().status = {
      mode: 'attached',
      baseUrl: 'http://unused.invalid',
      port: 1,
      binaryPath: null,
      kernelTier: null,
      healthy: true,
    }
  })
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    hostEventBus.all.clear()
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('does not poll for the retained rich role or apply an in-flight rich snapshot in minimal mode', async () => {
    let resolve!: (value: RoleSnapshot) => void
    mocks.snapshot.mockImplementationOnce(() => new Promise(r => resolve = r))
    let poll!: ReturnType<typeof useRoleSnapshotPoll>
    mountSetup(() => poll = useRoleSnapshotPoll())
    const before = useRoleStore().roleInfo.favorability
    const inFlight = poll.tick()
    bindMinimal()
    resolve({ role_id: 'rich', current_favorability: 99, current_emotion: 'happy', portrait_emotion: 'happy', relation_state: 'close', personality_source: 'vector', current_scene: null, user_presence_scene: null })
    await inFlight
    await poll.tick()
    await useRoleStore().refreshRoleInfo()
    expect(useRoleStore().roleInfo.favorability).toBe(before)
    expect(mocks.snapshot).toHaveBeenCalledOnce()
    expect(mocks.ipc).not.toHaveBeenCalled()
    useMinimalRoleChatStore().unbindSource()
    mocks.snapshot.mockResolvedValue(null)
    await poll.tick()
    expect(mocks.snapshot).toHaveBeenCalledTimes(2)
  })

  it('refuses role-mutating plugin events and ASR submissions, then restores the rich event path', () => {
    const asr = vi.fn()
    const travel = vi.fn()
    mountSetup(() => usePluginEvents({ showToast: vi.fn(), onPureChatMode: vi.fn(), onVoiceAsrSubmit: asr, onQuickActionTravel: travel }))
    bindMinimal()
    hostEventBus.emit('com.oclive.mumu.settings-panel:set_interaction_mode', { mode: 'immersive' })
    hostEventBus.emit('com.oclive.mumu.settings-panel:set_remote_life', { enabled: true })
    hostEventBus.emit('com.oclive.mumu.settings-panel:request_reset_layout', {})
    hostEventBus.emit('com.oclive.mumu.quick-actions:travel', {})
    hostEventBus.emit(VOICE_ASR_SUBMIT_EVENT, { text: 'synthetic', mode: 'submit', submissionId: 'scope-1' })
    expect(mocks.ipc).not.toHaveBeenCalled()
    expect(asr).not.toHaveBeenCalled()
    expect(travel).not.toHaveBeenCalled()
    useMinimalRoleChatStore().unbindSource()
    hostEventBus.emit(VOICE_ASR_SUBMIT_EVENT, { text: 'synthetic', mode: 'submit', submissionId: 'scope-1' })
    hostEventBus.emit('com.oclive.mumu.quick-actions:travel', {})
    expect(asr).toHaveBeenCalledOnce()
    expect(travel).toHaveBeenCalledOnce()
  })

  it('guards settings and hold-to-talk while keeping Host model management available', () => {
    const settingsOpen = ref(false)
    const models = vi.fn()
    const hold = vi.fn()
    hostEventBus.on('com.oclive.voice.asr:hold', hold)
    let keys!: ReturnType<typeof useMainShellHotkeys>
    mountSetup(() => {
      const role = useRoleStore()
      keys = useMainShellHotkeys({
        simplePluginManagerOpen: ref(false),
        settingsViewOpen: settingsOpen,
        topMoreOpen: ref(false),
        marketPanelVisible: computed(() => false),
        modelManagerOpen: ref(false),
        debugVisible: computed(() => false),
        pluginUiEnabled: computed(() => role.interactionImmersive),
        debugUiEnabled: computed(() => role.interactionImmersive),
        settingsUiEnabled: computed(() => !role.minimalRoleActive),
        voiceInputEnabled: computed(() => !role.minimalRoleActive),
        openPluginManagerPanel: vi.fn(),
        openModelManager: models,
        toggleDebug: vi.fn(),
        closeMarketPanel: vi.fn(),
        closeModelManager: vi.fn(),
        settingsFocusTab: ref(null),
      })
    })
    const actions = mocks.bindings.mock.calls[0][0] as Parameters<typeof useUnifiedKeybindings>[0]
    const settings = actions.appActions.find(action => action.actionId === 'app.openSettings')!
    const model = actions.appActions.find(action => action.actionId === 'app.openModelManager')!
    const voice = actions.holdActions![0]
    voice.onStart()
    expect(hold).toHaveBeenCalledWith({ phase: 'start' })
    bindMinimal()
    expect(hold).toHaveBeenLastCalledWith({ phase: 'stop' })
    hold.mockClear()
    expect(settings.enabled.value).toBe(false)
    expect(voice.enabled.value).toBe(false)
    keys.openSettingsView()
    settings.run()
    voice.onStart()
    expect(settingsOpen.value).toBe(false)
    expect(hold).not.toHaveBeenCalled()
    model.run()
    expect(models).toHaveBeenCalledOnce()
    useMinimalRoleChatStore().unbindSource()
    settings.run()
    voice.onStart()
    expect(settingsOpen.value).toBe(true)
    expect(hold).toHaveBeenCalledWith({ phase: 'start' })
  })

  it('clears the retained rich pack theme and restores it only when returning', async () => {
    useRoleStore().roleInfo.packUiConfig.theme = { primaryColor: '#123456' }
    mountSetup(usePackUiTheme)
    expect(document.documentElement.style.getPropertyValue('--focus-ring-color')).toBe('#123456')
    bindMinimal()
    await Promise.resolve()
    expect(document.documentElement.style.getPropertyValue('--focus-ring-color')).toBe('')
    useMinimalRoleChatStore().unbindSource()
    await Promise.resolve()
    expect(document.documentElement.style.getPropertyValue('--focus-ring-color')).toBe('#123456')
  })

  it('releases the actual keyboard registry hold when disabled and allows a new hold after returning', async () => {
    const actual = await vi.importActual<typeof import('./useUnifiedKeybindings')>('./useUnifiedKeybindings')
    vi.spyOn(document, 'hasFocus').mockReturnValue(true)
    const start = vi.fn()
    const stop = vi.fn()
    mountSetup(() => {
      actual.useUnifiedKeybindings({
        appActions: [],
        holdActions: [{
          actionId: 'voice.holdToTalk',
          enabled: computed(() => !useRoleStore().minimalRoleActive),
          onStart: start,
          onStop: stop,
        }],
      })
    })
    const keydown = () => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'v', code: 'KeyV' }))
    const keyup = () => window.dispatchEvent(new KeyboardEvent('keyup', { key: 'v', code: 'KeyV' }))
    keydown()
    expect(start).toHaveBeenCalledOnce()
    bindMinimal()
    expect(stop).toHaveBeenCalledOnce()
    keyup()
    keydown()
    expect(start).toHaveBeenCalledOnce()
    useMinimalRoleChatStore().unbindSource()
    keydown()
    expect(start).toHaveBeenCalledTimes(2)
    keyup()
    expect(stop).toHaveBeenCalledTimes(2)
  })
})
