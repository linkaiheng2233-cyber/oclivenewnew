// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { useModalFocusRestore } from './useModalFocusRestore'

const wrappers: Array<ReturnType<typeof mount>> = []
let opener: HTMLButtonElement
function render(initial = true, trapFocus = true) {
  const open = ref(initial)
  const root = ref<HTMLElement | null>(null)
  const wrapper = mount(defineComponent({
    setup() {
      useModalFocusRestore(open, root, { trapFocus })
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
})
