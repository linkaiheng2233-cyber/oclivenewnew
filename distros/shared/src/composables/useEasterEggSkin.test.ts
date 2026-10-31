// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { useEasterEggSkin } from './useEasterEggSkin'

const mocks = vi.hoisted(() => ({ toast: vi.fn(), emit: vi.fn(), decorations: vi.fn() }))
vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: (key: string) => key }) }))
vi.mock('@oclive/shared/composables/useAppToast', () => ({ useAppToast: () => ({ showToast: mocks.toast }) }))
vi.mock('@oclive/shared/lib/hostAppearance', () => ({ readHostAppearance: () => ({ theme: 'dark', shell: 'fluent', uiScale: 1.25 }) }))
vi.mock('@oclive/shared/lib/hostEventBus', () => ({ hostEventBus: { emitBuiltin: mocks.emit } }))
vi.mock('@tauri-apps/api/webviewWindow', () => ({ getCurrentWebviewWindow: () => ({ setDecorations: mocks.decorations }) }))

const sequence = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'KeyB', 'KeyA']
const wrappers: Array<ReturnType<typeof mount>> = []
function render() {
  let skin!: ReturnType<typeof useEasterEggSkin>
  const wrapper = mount(defineComponent({
    setup() {
      skin = useEasterEggSkin()
      return () => null
    },
  }))
  wrappers.push(wrapper)
  return { skin, wrapper }
}
function press(codes = sequence) {
  for (const code of codes)
    window.dispatchEvent(new KeyboardEvent('keydown', { code }))
}

describe('win98 discovery and restoration lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    document.documentElement.removeAttribute('data-skin')
    mocks.decorations.mockResolvedValue(undefined)
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
  })
  afterEach(async () => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    await flushPromises()
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('requires a complete sequence and unlocks without changing other appearance axes', () => {
    localStorage.setItem('oclive-runtime-theme', 'dark')
    const { skin } = render()
    press(['ArrowUp', 'ArrowDown', 'KeyB', 'KeyA'])
    expect(skin.skinUnlocked.value).toBe(false)
    press()
    expect(skin.skinUnlocked.value).toBe(true)
    expect(skin.win98Enabled.value).toBe(true)
    expect(localStorage.getItem('oclive-easteregg-unlocked')).toBe('1')
    expect(localStorage.getItem('oclive-runtime-skin')).toBe('win98')
    expect(localStorage.getItem('oclive-runtime-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-skin')).toBe('win98')
    expect(mocks.emit).toHaveBeenLastCalledWith('appearance:changed', { theme: 'dark', shell: 'fluent', uiScale: 1.25, skin: 'win98' })
    expect(mocks.toast).toHaveBeenCalledTimes(1)
  })

  it('keeps an undiscovered switch inactive and restores the native frame when disabled', async () => {
    const { skin } = render()
    skin.toggleWin98()
    expect(skin.win98Enabled.value).toBe(false)
    press()
    await flushPromises()
    expect(mocks.decorations).toHaveBeenLastCalledWith(false)
    skin.toggleWin98()
    await flushPromises()
    expect(mocks.decorations).toHaveBeenLastCalledWith(true)
    expect(document.documentElement.getAttribute('data-skin')).toBe('default')
    expect(localStorage.getItem('oclive-runtime-skin')).toBeNull()
    expect(localStorage.getItem('oclive-easteregg-unlocked')).toBe('1')
    expect(mocks.emit).toHaveBeenLastCalledWith('appearance:changed', expect.objectContaining({ skin: 'default' }))
  })

  it('restores an enabled skin after remount without requiring discovery again', async () => {
    localStorage.setItem('oclive-easteregg-unlocked', '1')
    localStorage.setItem('oclive-runtime-skin', 'win98')
    const { skin } = render()
    await flushPromises()
    expect(skin.skinUnlocked.value).toBe(true)
    expect(skin.win98Enabled.value).toBe(true)
    expect(mocks.decorations).toHaveBeenLastCalledWith(false)
    expect(mocks.toast).not.toHaveBeenCalled()
  })

  it('remembers discovery while disabled and allows the sequence to re-enable without another toast', () => {
    localStorage.setItem('oclive-easteregg-unlocked', '1')
    const { skin } = render()
    expect(skin.win98Enabled.value).toBe(false)
    press()
    expect(skin.win98Enabled.value).toBe(true)
    expect(mocks.toast).not.toHaveBeenCalled()
  })

  it('shares one listener across consumers and removes it when the final consumer closes', () => {
    const first = render()
    const second = render()
    first.wrapper.unmount()
    press()
    expect(second.skin.win98Enabled.value).toBe(true)
    expect(mocks.toast).toHaveBeenCalledTimes(1)
    second.skin.toggleWin98()
    second.wrapper.unmount()
    press()
    expect(localStorage.getItem('oclive-runtime-skin')).toBeNull()
    expect(mocks.toast).toHaveBeenCalledTimes(1)
  })

  it('resets a partial sequence when the application remounts', () => {
    const first = render()
    press(sequence.slice(0, 6))
    first.wrapper.unmount()
    const second = render()
    press(sequence.slice(6))
    expect(second.skin.skinUnlocked.value).toBe(false)
    press()
    expect(second.skin.skinUnlocked.value).toBe(true)
  })

  it('keeps switching usable in a browser and when native decoration permission is unavailable', async () => {
    mocks.decorations.mockRejectedValue(new Error('permission unavailable'))
    const { skin } = render()
    press()
    await flushPromises()
    expect(skin.win98Enabled.value).toBe(true)
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    skin.toggleWin98()
    await flushPromises()
    expect(skin.win98Enabled.value).toBe(false)
    expect(document.documentElement.getAttribute('data-skin')).toBe('default')
  })

  it('waits for an in-flight native frame change before restoring the latest choice', async () => {
    const { skin } = render()
    await flushPromises()
    let finish!: () => void
    mocks.decorations.mockImplementationOnce(() => new Promise<void>((resolve) => {
      finish = resolve
    }))
    press()
    await flushPromises()
    expect(mocks.decorations).toHaveBeenLastCalledWith(false)
    skin.toggleWin98()
    await flushPromises()
    expect(mocks.decorations).toHaveBeenLastCalledWith(false)
    finish()
    await flushPromises()
    expect(mocks.decorations).toHaveBeenLastCalledWith(true)
    expect(document.documentElement.getAttribute('data-skin')).toBe('default')
  })
})
