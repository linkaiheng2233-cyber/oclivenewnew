// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { i18n } from '../i18n'
import ErrorNotice from './ErrorNotice.vue'

describe('two-level error display', () => {
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
})
