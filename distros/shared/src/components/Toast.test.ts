// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { useAppToast } from '../composables/useAppToast'
import { i18n } from '../i18n'
import Toast from './Toast.vue'

describe('notification severity and dismissal', () => {
  afterEach(() => useAppToast().dismissToast())

  it('keeps warning diagnostics separate from error alerts', () => {
    const wrapper = mount(Toast, {
      props: { show: true, type: 'warning', message: 'Your reply arrived', details: 'status refresh unavailable' },
      global: { plugins: [i18n] },
    })
    expect(wrapper.find('[role=alert]').exists()).toBe(false)
    expect(wrapper.get('.error-notice--warning').attributes('role')).toBe('status')
    expect(wrapper.get('.error-notice-heading p').text()).toBe('Your reply arrived')
    wrapper.unmount()
  })

  it('lets a short success notice be dismissed without waiting for its timer', async () => {
    const { toast, showToast } = useAppToast()
    showToast('success', 'Saved')
    const wrapper = mount(Toast, {
      props: { show: true, type: 'success', message: 'Saved' },
      global: { plugins: [i18n] },
    })
    expect(wrapper.get('[role=status]').text()).toBe('Saved')
    await wrapper.get('button').trigger('click')
    expect(toast.value.show).toBe(false)
    wrapper.unmount()
  })
})
