// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { i18n } from '../i18n'
import { useOcliveAppearance } from './useOcliveAppearance'

const mocks = vi.hoisted(() => ({ emit: vi.fn() }))
vi.mock('@oclive/shared/lib/hostEventBus', () => ({ hostEventBus: { emitBuiltin: mocks.emit } }))

const wrappers: Array<ReturnType<typeof mount>> = []
let systemDark = false
let schemeListener: (() => void) | undefined
const removeListener = vi.fn()
const addListener = vi.fn((_event: string, listener: () => void) => {
  schemeListener = listener
})

function render() {
  let appearance!: ReturnType<typeof useOcliveAppearance>
  const wrapper = mount(defineComponent({
    setup() {
      appearance = useOcliveAppearance()
      return () => null
    },
  }), { global: { plugins: [i18n] } })
  wrappers.push(wrapper)
  return { appearance, wrapper }
}

describe('shared appearance preferences', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    systemDark = false
    schemeListener = undefined
    vi.stubGlobal('matchMedia', vi.fn(() => ({
      matches: systemDark,
      addEventListener: addListener,
      removeEventListener: removeListener,
    })))
  })
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    vi.unstubAllGlobals()
  })

  it('shares a quick theme change with mounted settings without adding watchers or rehydrating stale state', () => {
    const first = render()
    first.appearance.toggleDarkTheme()
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(localStorage.getItem('oclive-runtime-theme')).toBe('dark')
    const events = mocks.emit.mock.calls.length
    const settings = render()
    expect(settings.appearance.themePreference.value).toBe('dark')
    expect(addListener).toHaveBeenCalledTimes(1)
    expect(mocks.emit.mock.calls).toHaveLength(events)
    settings.appearance.cycleTheme()
    expect(first.appearance.themePreference.value).toBe('system')
    expect(first.appearance.effectiveTheme.value).toBe('light')
    expect(mocks.emit.mock.calls).toHaveLength(events + 1)
    first.wrapper.unmount()
    settings.appearance.toggleDarkTheme()
    expect(localStorage.getItem('oclive-runtime-theme')).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(removeListener).not.toHaveBeenCalled()
  })

  it('follows system changes only while the system preference is selected', () => {
    const { appearance } = render()
    systemDark = true
    schemeListener?.()
    expect(appearance.effectiveTheme.value).toBe('dark')
    expect(appearance.themePreference.value).toBe('system')
    appearance.toggleDarkTheme()
    expect(appearance.themePreference.value).toBe('light')
    schemeListener?.()
    expect(appearance.effectiveTheme.value).toBe('light')
    appearance.cycleTheme()
    appearance.cycleTheme()
    expect(appearance.themePreference.value).toBe('system')
    expect(appearance.effectiveTheme.value).toBe('dark')
  })

  it('shares scale changes and removes its listener after the final consumer unmounts', () => {
    localStorage.setItem('oclive-runtime-ui-scale', '1.16')
    const first = render()
    const second = render()
    first.appearance.bumpScale(1)
    expect(second.appearance.scaleLabel.value).toBe('124%')
    expect(document.documentElement.style.getPropertyValue('--oclive-ui-scale')).toBe('1.24')
    first.wrapper.unmount()
    second.wrapper.unmount()
    expect(removeListener).toHaveBeenCalledTimes(1)
    const events = mocks.emit.mock.calls.length
    second.appearance.themePreference.value = 'light'
    expect(mocks.emit.mock.calls).toHaveLength(events)
    localStorage.setItem('oclive-runtime-theme', 'dark')
    const replacement = render()
    expect(replacement.appearance.effectiveTheme.value).toBe('dark')
    expect(addListener).toHaveBeenCalledTimes(2)
  })

  it('still changes the visible theme if local persistence is unavailable', () => {
    const { appearance } = render()
    const write = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('unavailable')
    })
    appearance.toggleDarkTheme()
    expect(appearance.effectiveTheme.value).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    write.mockRestore()
  })
})
