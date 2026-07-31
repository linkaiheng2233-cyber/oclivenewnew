// @vitest-environment jsdom
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterEach, describe, expect, it, vi } from 'vitest'

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const slots = path.join(repo, 'distros/chat-pro/plugins/com.oclive.voice.asr/slots')
const readSlot = (name: string) => readFileSync(path.join(slots, name), 'utf8')

function mountHtml(name: string): void {
  const parsed = new DOMParser().parseFromString(readSlot(name), 'text/html')
  document.head.innerHTML = parsed.head.innerHTML
  document.body.innerHTML = parsed.body.innerHTML
}

afterEach(() => {
  document.head.innerHTML = ''
  document.body.innerHTML = ''
  delete (window as Window & { OclivePluginBridge?: unknown }).OclivePluginBridge
})

describe('official Voice HTML fallbacks', () => {
  it('ships a functional isolated toolbar instead of a placeholder', () => {
    const html = readSlot('toolbar.html')
    const script = readSlot('voice-toolbar.js')

    expect(html).toContain('voice-toolbar.js')
    expect(html).toContain('id="record"')
    expect(script).toContain('rpc(\'voice.transcribe\'')
    expect(script).toContain('bridge.audioCapture.start()')
    expect(script).toContain('bridge.audioCapture.stop()')
    expect(script).not.toContain('navigator.mediaDevices.getUserMedia')
    expect(script).toContain('bridge.emit(submitEvent')
    expect(script).toContain('bridge.listen(holdEvent')
    expect(html).not.toContain('iframe fallback')
  })

  it('ships settings, probe, import, warm and persistence controls', () => {
    const html = readSlot('settings.html')
    const script = readSlot('voice-settings.js')

    for (const id of ['asr-profile', 'tts-profile', 'save', 'warm', 'import-asr', 'import-tts'])
      expect(html).toContain(`id="${id}"`)
    for (const command of [
      'get_plugin_settings_ui',
      'set_plugin_settings_config',
      'voice.list_profiles',
      'voice.probe',
      'voice.import_model',
      'voice.import_tts_adapter',
      'voice.warm',
    ])
      expect(script).toContain(command)
    expect(script).toContain('bridge.emit(configEvent')
    expect(html).not.toContain('iframe fallback')
  })

  it('initializes the toolbar through the isolated bridge', async () => {
    mountHtml('toolbar.html')
    const invoke = vi.fn(async (command: string, params?: Record<string, unknown>) => {
      if (command === 'get_plugin_settings_ui')
        return { config: { submit_mode: 'fill', asr_profile: 'test-asr' } }
      if (command === 'plugin_rpc_invoke' && params?.method === 'voice.probe')
        return { ok: true, message: 'ready through broker' }
      return {}
    })
    const listen = vi.fn().mockResolvedValue(() => {})
    ;(window as Window & { OclivePluginBridge?: unknown }).OclivePluginBridge = {
      invoke,
      emit: vi.fn(),
      listen,
      audioCapture: {
        start: vi.fn().mockResolvedValue({ mimeType: 'audio/webm' }),
        stop: vi.fn(),
        cancel: vi.fn().mockResolvedValue(null),
      },
    }

    // The fixture is shipped as a classic browser script; executing that exact artifact is the test subject.
    // eslint-disable-next-line no-eval
    window.eval(readSlot('voice-toolbar.js'))

    await vi.waitFor(() => expect(listen).toHaveBeenCalledWith(
      'com.oclive.voice.asr:hold',
      expect.any(Function),
    ))
    expect(invoke).toHaveBeenCalledWith('plugin_rpc_invoke', {
      method: 'voice.probe',
      params: { profile: 'test-asr' },
    })
    expect(document.querySelector('#status')?.textContent).toBe('ready through broker')
    expect(document.querySelector<HTMLButtonElement>('#record')?.disabled).toBe(false)
  })

  it('persists settings and emits the bound config event', async () => {
    mountHtml('settings.html')
    const invoke = vi.fn(async (command: string, params?: Record<string, unknown>) => {
      if (command === 'get_plugin_settings_ui')
        return { config: { asr_profile: 'asr-a', tts_profile: 'tts-a' } }
      if (command === 'set_plugin_settings_config')
        return { ok: true }
      if (command !== 'plugin_rpc_invoke')
        return {}
      const method = params?.method
      if (method === 'voice.list_profiles') {
        return { profiles: [
          { id: 'asr-a', label: 'ASR A', engine: 'test', kind: 'asr' },
          { id: 'tts-a', label: 'TTS A', engine: 'test', kind: 'tts' },
          { id: 'director-a', label: 'Director A', engine: 'test', kind: 'director' },
        ] }
      }
      if (method === 'voice.list_model_packs')
        return { packs: [] }
      if (method === 'voice.list_tts_adapters')
        return { adapters: [] }
      if (method === 'voice.probe')
        return { ok: true }
      if (method === 'config_updated')
        return { ok: true }
      return {}
    })
    const emit = vi.fn().mockResolvedValue(null)
    ;(window as Window & { OclivePluginBridge?: unknown }).OclivePluginBridge = {
      invoke,
      emit,
      listen: vi.fn(),
    }

    // The fixture is shipped as a classic browser script; executing that exact artifact is the test subject.
    // eslint-disable-next-line no-eval
    window.eval(readSlot('voice-settings.js'))
    await vi.waitFor(() => expect(document.querySelector('#asr-profile option')?.textContent).toBe('ASR A'))
    document.querySelector<HTMLButtonElement>('#save')?.click()

    await vi.waitFor(() => expect(emit).toHaveBeenCalledWith(
      'com.oclive.voice.asr:config-updated',
      {},
    ))
    expect(invoke).toHaveBeenCalledWith(
      'set_plugin_settings_config',
      expect.objectContaining({ pluginId: 'com.oclive.voice.asr' }),
    )
  })
})
