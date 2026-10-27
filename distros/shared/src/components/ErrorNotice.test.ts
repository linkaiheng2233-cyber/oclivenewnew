// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { i18n } from '../i18n'
import ErrorNotice from './ErrorNotice.vue'

describe('two-level error display', () => {
  afterEach(() => {
    Reflect.deleteProperty(navigator, 'clipboard')
  })
  it('renders the plain-language summary before collapsed, selectable technical details', async () => {
    const wrapper = mount(ErrorNotice, {
      props: { error: JSON.stringify({ code: 'LLM_ERROR', message: 'Ollama: connect ECONNREFUSED' }), dismissible: true },
      global: { plugins: [i18n] },
    })
    expect(wrapper.get('.error-notice-heading p').text()).not.toContain('ECONNREFUSED')
    expect(wrapper.get('details').attributes('open')).toBeUndefined()
    expect(wrapper.get('pre').text()).toContain('ECONNREFUSED')
    expect(wrapper.html().indexOf('error-notice-heading')).toBeLessThan(wrapper.html().indexOf('<details'))
    await wrapper.get('button').trigger('click')
    expect(wrapper.emitted('dismiss')).toHaveLength(1)
    wrapper.unmount()
  })

  it('redacts explicitly supplied diagnostics before displaying and copying them', async () => {
    const basicCredential = btoa('review-user:synthetic-password')
    const copy = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: copy } })
    const wrapper = mount(ErrorNotice, {
      props: { summary: 'Model unavailable', details: `LLM_ERROR: api_key=private-copy-fixture; Authorization: Bearer private-bearer; Proxy-Authorization: Basic ${basicCredential}` },
      global: { plugins: [i18n] },
    })
    await wrapper.get('.error-notice-details button').trigger('click')
    await flushPromises()
    expect(copy).toHaveBeenCalledTimes(1)
    expect(copy.mock.calls[0]![0]).toContain('Model unavailable')
    expect(copy.mock.calls[0]![0]).toContain('LLM_ERROR')
    expect(copy.mock.calls[0]![0]).not.toContain('private-copy-fixture')
    expect(copy.mock.calls[0]![0]).not.toContain('private-bearer')
    expect(copy.mock.calls[0]![0]).not.toContain(basicCredential)
    expect(wrapper.text()).not.toContain('private-copy-fixture')
    expect(wrapper.text()).not.toContain(basicCredential)
    expect(wrapper.get('.error-notice-copy-status').text()).toBe(i18n.global.t('errorUi.copied'))
    wrapper.unmount()
  })

  it('keeps diagnostics selectable when clipboard access fails', async () => {
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: vi.fn().mockRejectedValue(new Error('denied')) } })
    const wrapper = mount(ErrorNotice, {
      props: { summary: 'Not saved', details: 'DB_ERROR: file locked', tone: 'warning' },
      global: { plugins: [i18n] },
    })
    await wrapper.get('.error-notice-details button').trigger('click')
    await flushPromises()
    expect(wrapper.attributes('role')).toBe('status')
    expect(wrapper.get('.error-notice-copy-status').text()).toBe(i18n.global.t('errorUi.copyFailed'))
    expect(wrapper.get('pre').attributes('tabindex')).toBe('0')
    expect(wrapper.get('pre').text()).toContain('DB_ERROR')
    expect(wrapper.get('button').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('ignores a copy result from the preceding notice', async () => {
    let complete!: () => void
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: vi.fn(() => new Promise<void>((resolve) => {
      complete = resolve
    })) } })
    const wrapper = mount(ErrorNotice, {
      props: { summary: 'First failure', details: 'old details' },
      global: { plugins: [i18n] },
    })
    await wrapper.get('.error-notice-details button').trigger('click')
    await wrapper.setProps({ summary: 'Second failure', details: 'new details' })
    complete()
    await flushPromises()
    expect(wrapper.find('.error-notice-copy-status').exists()).toBe(false)
    expect(wrapper.get('pre').text()).toBe('new details')
    expect(wrapper.get('details').attributes('open')).toBeUndefined()
    wrapper.unmount()
  })
})
