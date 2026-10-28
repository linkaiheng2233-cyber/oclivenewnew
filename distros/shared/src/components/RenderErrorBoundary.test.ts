// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import { useDiagnosticHistory } from '../composables/useDiagnosticHistory'
import { i18n } from '../i18n'
import RenderErrorBoundary from './RenderErrorBoundary.vue'

describe('recoverable render region', () => {
  it.each([false, '', null])('also recovers from a falsy thrown value %s', async (value) => {
    const child = defineComponent({
      render() {
        throw value
      },
    })
    const wrapper = mount(RenderErrorBoundary, {
      slots: { default: () => h(child) },
      global: { plugins: [i18n] },
    })
    await flushPromises()
    expect(wrapper.find('.render-recovery').exists()).toBe(true)
    wrapper.unmount()
  })
  it('keeps surrounding controls, records redacted diagnostics and retries without replaying actions', async () => {
    let broken = true
    let actions = 0
    const child = defineComponent({
      render() {
        if (broken)
          throw new Error('fixture-render\nAuthorization: Bearer private-render-fixture')
        return h('button', {
          onClick: () => {
            actions += 1
          },
        }, 'explicit action')
      },
    })
    const wrapper = mount(defineComponent({ render: () => h('div', [
      h('nav', { 'data-test': 'help' }, 'Help'),
      h(RenderErrorBoundary, {}, { default: () => h(child) }),
    ]) }), { attachTo: document.body, global: { plugins: [i18n] } })
    await flushPromises()
    expect(wrapper.get('[data-test="help"]').text()).toBe('Help')
    expect(wrapper.find('.render-recovery').exists()).toBe(true)
    expect(wrapper.html()).not.toContain('private-render-fixture')
    expect(JSON.stringify(useDiagnosticHistory().entries.value)).not.toContain('private-render-fixture')
    broken = false
    await wrapper.get('.render-recovery > button').trigger('click')
    await flushPromises()
    expect(wrapper.find('.render-recovery').exists()).toBe(false)
    expect(actions).toBe(0)
    await wrapper.get('button').trigger('click')
    expect(actions).toBe(1)
    wrapper.unmount()
  })
})
