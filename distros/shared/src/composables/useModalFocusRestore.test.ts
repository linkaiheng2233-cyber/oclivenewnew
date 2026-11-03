// @vitest-environment jsdom

import type { Ref } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { useModalFocusRestore } from './useModalFocusRestore'

const wrappers: Array<ReturnType<typeof mount>> = []
let opener: HTMLButtonElement
function render(initial = true, trapFocus = true, explicitOpener?: Ref<HTMLElement | null>, ownsEscape = false) {
  const open = ref(initial)
  const root = ref<HTMLElement | null>(null)
  const wrapper = mount(defineComponent({
    setup() {
      useModalFocusRestore(open, root, {
        trapFocus,
        opener: explicitOpener,
        onEscape: ownsEscape
          ? () => {
              open.value = false
            }
          : undefined,
      })
      return () => open.value
        ? h('section', { ref: root, tabindex: -1 }, [
            h('button', { id: 'first-control' }, 'First'),
            h('input', { disabled: true }),
            h('button', { hidden: true }, 'Hidden'),
            h('button', { id: 'last-control' }, 'Last'),
          ])
        : null
    },
  }), { attachTo: document.body })
  wrappers.push(wrapper)
  return { wrapper, open, root }
}
function tab(target: HTMLElement, shiftKey = false) {
  const event = new KeyboardEvent('keydown', { key: 'Tab', shiftKey, bubbles: true, cancelable: true })
  target.dispatchEvent(event)
  return event
}

describe('modal focus lifecycle', () => {
  beforeEach(() => {
    opener = document.createElement('button')
    document.body.append(opener)
    opener.focus()
    vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(function (this: HTMLElement) {
      return (this.hidden ? [] : [new DOMRect(0, 0, 20, 20)]) as unknown as DOMRectList
    })
  })
  afterEach(async () => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    await flushPromises()
    opener.remove()
    vi.restoreAllMocks()
  })

  it('focuses a dialog that was already open when conditionally mounted', async () => {
    const { root } = render()
    await flushPromises()
    expect(document.activeElement).toBe(root.value)
  })

  it('wraps Tab and Shift+Tab without landing on disabled or hidden controls', async () => {
    const { root } = render()
    await flushPromises()
    expect(tab(root.value!).defaultPrevented).toBe(true)
    expect(document.activeElement?.id).toBe('first-control')
    tab(document.activeElement as HTMLElement, true)
    expect(document.activeElement?.id).toBe('last-control')
    tab(document.activeElement as HTMLElement)
    expect(document.activeElement?.id).toBe('first-control')
  })

  it('restores the opener when the controlled dialog closes', async () => {
    const { open } = render()
    await flushPromises()
    open.value = false
    await flushPromises()
    expect(document.activeElement).toBe(opener)
  })

  it('returns to an explicit trigger that lost focus before an async dialog opened', async () => {
    const returnTo = ref<HTMLElement | null>(opener)
    opener.blur()
    expect(document.activeElement).not.toBe(opener)
    const { open } = render(true, true, returnTo)
    await flushPromises()
    open.value = false
    await flushPromises()
    expect(document.activeElement).toBe(opener)
  })

  it('restores the opener when a v-if removes the whole dialog component', async () => {
    const { wrapper } = render()
    await flushPromises()
    wrapper.unmount()
    await flushPromises()
    expect(document.activeElement).toBe(opener)
  })

  it('does not focus a disposed dialog after a queued opening', async () => {
    const { wrapper } = render()
    wrapper.unmount()
    await flushPromises()
    expect(document.activeElement).toBe(opener)
  })

  it('does not capture Tab for nonmodal consumers that did not opt into trapping', async () => {
    const { root } = render(true, false)
    await flushPromises()
    expect(tab(root.value!).defaultPrevented).toBe(false)
  })

  it('closes the recovery dialog before global handlers even if native focus left its subtree', async () => {
    const lowerWindow = vi.fn()
    window.addEventListener('keydown', lowerWindow)
    const { open } = render(true, true, undefined, true)
    await flushPromises()
    opener.focus()
    const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
    opener.dispatchEvent(event)
    expect(event.defaultPrevented).toBe(true)
    expect(open.value).toBe(false)
    expect(lowerWindow).not.toHaveBeenCalled()
    await flushPromises()
    expect(document.activeElement).toBe(opener)
    opener.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(lowerWindow).toHaveBeenCalledTimes(1)
    window.removeEventListener('keydown', lowerWindow)
  })

  it('closes only the top dialog when another modal opens above recovery', async () => {
    const lower = render(true, true, undefined, true)
    await flushPromises()
    const upper = render(true, true, undefined, true)
    await flushPromises()
    opener.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    expect(upper.open.value).toBe(false)
    expect(lower.open.value).toBe(true)
    await flushPromises()
    opener.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    expect(lower.open.value).toBe(false)
  })
})
