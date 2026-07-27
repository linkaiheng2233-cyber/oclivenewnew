// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import HelpHint from './HelpHint.vue'

function mountHint(text = '第一段\n\n第二段') {
  const i18n = createI18n({
    legacy: false,
    locale: 'zh-CN',
    messages: {
      'zh-CN': {
        app: {
          helpHintAria: '查看说明',
          helpHintCloseAria: '关闭说明',
        },
      },
    },
  })

  return mount(HelpHint, {
    attachTo: document.body,
    props: { text },
    global: { plugins: [i18n] },
  })
}

describe('help hint', () => {
  beforeEach(() => {
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      x: 20,
      y: 20,
      width: 200,
      height: 40,
      top: 20,
      right: 220,
      bottom: 60,
      left: 20,
      toJSON: () => ({}),
    })
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      callback(0)
      return 1
    })
    vi.stubGlobal('cancelAnimationFrame', vi.fn())
    vi.stubGlobal('ResizeObserver', class {
      observe() {}
      disconnect() {}
    })
  })

  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  it('teleports paragraph content to the document body', async () => {
    const wrapper = mountHint()

    await wrapper.get('button').trigger('click')
    await nextTick()

    const popover = document.body.querySelector<HTMLElement>('.help-pop')
    expect(popover?.parentElement).toBe(document.body)
    expect(popover?.querySelectorAll('p')).toHaveLength(2)
    expect(wrapper.get('button').attributes('aria-expanded')).toBe('true')

    wrapper.unmount()
  })

  it('uses Escape for the help layer first and restores trigger focus', async () => {
    const wrapper = mountHint()
    const trigger = wrapper.get('button').element

    await wrapper.get('button').trigger('click')
    await nextTick()

    const event = new KeyboardEvent('keydown', {
      key: 'Escape',
      bubbles: true,
      cancelable: true,
    })
    document.dispatchEvent(event)
    await nextTick()

    expect(event.defaultPrevented).toBe(true)
    expect(document.body.querySelector('.help-pop')).toBeNull()
    expect(document.activeElement).toBe(trigger)
    expect(wrapper.get('button').attributes('aria-expanded')).toBe('false')

    wrapper.unmount()
  })

  it('closes the previous hint when another help button is clicked', async () => {
    const first = mountHint('第一条说明')
    const second = mountHint('第二条说明')

    await first.get('button').trigger('click')
    await nextTick()
    await second.get('button').trigger('pointerdown')
    await second.get('button').trigger('click')
    await nextTick()

    const popovers = document.body.querySelectorAll<HTMLElement>('.help-pop')
    expect(popovers).toHaveLength(1)
    expect(popovers[0]?.textContent).toContain('第二条说明')
    expect(first.get('button').attributes('aria-expanded')).toBe('false')
    expect(second.get('button').attributes('aria-expanded')).toBe('true')

    first.unmount()
    second.unmount()
  })
})
