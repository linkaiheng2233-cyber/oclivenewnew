// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import UiCloseButton from './UiCloseButton.vue'

vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: () => 'Close' }) }))

describe('shared close control', () => {
  it('shows only an icon while keeping a translated accessible name and tooltip', () => {
    const wrapper = mount(UiCloseButton)
    expect(wrapper.get('button').attributes('type')).toBe('button')
    expect(wrapper.get('button').attributes('aria-label')).toBe('Close')
    expect(wrapper.get('button').attributes('title')).toBe('Close')
    expect(wrapper.text()).toBe('')
    expect(wrapper.get('svg').attributes('aria-hidden')).toBe('true')
    wrapper.unmount()
  })

  it('keeps the surface-specific label and forwards one click without submitting a form', async () => {
    const onClick = vi.fn()
    const wrapper = mount(UiCloseButton, { props: { label: 'Dismiss notification' }, attrs: { onClick } })
    await wrapper.get('button').trigger('click')
    expect(onClick).toHaveBeenCalledTimes(1)
    expect(wrapper.get('button').attributes('aria-label')).toBe('Dismiss notification')
    wrapper.unmount()
  })

  it('does not invoke the action while disabled', async () => {
    const onClick = vi.fn()
    const wrapper = mount(UiCloseButton, { props: { disabled: true }, attrs: { onClick } })
    await wrapper.get('button').trigger('click')
    expect(onClick).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
